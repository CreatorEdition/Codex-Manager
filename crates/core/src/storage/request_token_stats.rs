use rusqlite::{params, params_from_iter, Result, Row};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicI64, Ordering};

use super::{
    now_ts, sqlite_placeholders, sqlite_text_params, ApiKeyModelTokenUsageSummary,
    ApiKeyTokenUsageSummary, DailyTokenUsageRollup, RequestLogTodaySummary, RequestTokenStat,
    RequestTokenStatDailyRollup, SourceTokenUsageRanking, SourceTokenUsageRollup, Storage,
    TokenUsageRollup, TokenUsageSummary, UserTokenUsageRanking, UserTokenUsageRollup,
};

const DEFAULT_REQUEST_TOKEN_STATS_RETAIN_DAYS: i64 = 14;
const DEFAULT_OBSERVABILITY_MAINTENANCE_INTERVAL_SECS: i64 = 900;
const DEFAULT_OBSERVABILITY_MAINTENANCE_BATCH_LIMIT: usize = 5_000;
const REQUEST_TOKEN_STATS_RETAIN_DAYS_ENV: &str = "CODEXMANAGER_REQUEST_TOKEN_STATS_RETENTION_DAYS";
const OBSERVABILITY_MAINTENANCE_INTERVAL_SECS_ENV: &str =
    "CODEXMANAGER_OBSERVABILITY_MAINTENANCE_INTERVAL_SECS";
const OBSERVABILITY_MAINTENANCE_BATCH_LIMIT_ENV: &str =
    "CODEXMANAGER_OBSERVABILITY_MAINTENANCE_BATCH_LIMIT";

static LAST_OBSERVABILITY_MAINTENANCE_AT: AtomicI64 = AtomicI64::new(0);

pub(super) fn request_token_stats_retain_days() -> i64 {
    std::env::var(REQUEST_TOKEN_STATS_RETAIN_DAYS_ENV)
        .ok()
        .and_then(|raw| raw.trim().parse::<i64>().ok())
        .unwrap_or(DEFAULT_REQUEST_TOKEN_STATS_RETAIN_DAYS)
}

fn observability_maintenance_interval_secs() -> i64 {
    std::env::var(OBSERVABILITY_MAINTENANCE_INTERVAL_SECS_ENV)
        .ok()
        .and_then(|raw| raw.trim().parse::<i64>().ok())
        .unwrap_or(DEFAULT_OBSERVABILITY_MAINTENANCE_INTERVAL_SECS)
}

pub fn observability_maintenance_batch_limit() -> usize {
    std::env::var(OBSERVABILITY_MAINTENANCE_BATCH_LIMIT_ENV)
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .map(|limit| limit.clamp(1, 100_000))
        .unwrap_or(DEFAULT_OBSERVABILITY_MAINTENANCE_BATCH_LIMIT)
}

pub(super) fn retention_cutoff(now: i64, days: i64) -> Option<i64> {
    (days > 0).then(|| now.saturating_sub(days.saturating_mul(86_400)))
}

fn token_total_sql_expr() -> &'static str {
    "CASE
        WHEN total_tokens IS NOT NULL THEN
            CASE WHEN total_tokens > 0 THEN total_tokens ELSE 0 END
        ELSE
            CASE
                WHEN IFNULL(input_tokens, 0) - IFNULL(cached_input_tokens, 0) + IFNULL(output_tokens, 0) > 0
                    THEN IFNULL(input_tokens, 0) - IFNULL(cached_input_tokens, 0) + IFNULL(output_tokens, 0)
                ELSE 0
            END
     END"
}

const TOKEN_ROLLUP_COLUMNS: &str = "
    IFNULL(SUM(IFNULL(t.input_tokens, 0)), 0) AS input_tokens,
    IFNULL(SUM(IFNULL(t.cached_input_tokens, 0)), 0) AS cached_input_tokens,
    IFNULL(SUM(IFNULL(t.output_tokens, 0)), 0) AS output_tokens,
    IFNULL(SUM(IFNULL(t.reasoning_output_tokens, 0)), 0) AS reasoning_output_tokens,
    IFNULL(
        SUM(
            CASE
                WHEN t.total_tokens IS NOT NULL THEN
                    CASE WHEN t.total_tokens > 0 THEN t.total_tokens ELSE 0 END
                ELSE
                    CASE
                        WHEN IFNULL(t.input_tokens, 0) - IFNULL(t.cached_input_tokens, 0) + IFNULL(t.output_tokens, 0) > 0
                            THEN IFNULL(t.input_tokens, 0) - IFNULL(t.cached_input_tokens, 0) + IFNULL(t.output_tokens, 0)
                        ELSE 0
                    END
            END
        ),
        0
    ) AS total_tokens,
    IFNULL(SUM(IFNULL(t.estimated_cost_usd, 0.0)), 0.0) AS estimated_cost_usd,
    COUNT(DISTINCT r.id) AS request_count,
    COUNT(DISTINCT CASE WHEN r.status_code >= 200 AND r.status_code <= 299 THEN r.id END) AS success_count,
    COUNT(DISTINCT CASE WHEN IFNULL(r.status_code, 0) >= 400 OR TRIM(IFNULL(r.error, '')) <> '' THEN r.id END) AS error_count";

const USER_OWNER_EXPR: &str =
    "COALESCE(NULLIF(TRIM(charge.owner_id), ''), NULLIF(TRIM(owner.owner_user_id), ''))";

// User attribution prefers the request_charge wallet owner. The api_key_owners
// fallback is current-owner based, so old uncharged logs are approximate.
const USER_OWNER_JOINS: &str = "
    LEFT JOIN (
        SELECT l.request_log_id, MIN(w.owner_id) AS owner_id
        FROM app_wallet_ledger_entries l
        JOIN app_wallets w ON w.id = l.wallet_id
        WHERE l.entry_kind = 'request_charge'
          AND w.owner_kind = 'user'
        GROUP BY l.request_log_id
    ) charge ON charge.request_log_id = r.id
    LEFT JOIN api_key_owners owner ON owner.key_id = r.key_id AND owner.owner_kind = 'user'";

fn token_usage_rollup_from_row(row: &Row<'_>, offset: usize) -> Result<TokenUsageRollup> {
    Ok(TokenUsageRollup {
        input_tokens: row.get::<_, i64>(offset)?.max(0),
        cached_input_tokens: row.get::<_, i64>(offset + 1)?.max(0),
        output_tokens: row.get::<_, i64>(offset + 2)?.max(0),
        reasoning_output_tokens: row.get::<_, i64>(offset + 3)?.max(0),
        total_tokens: row.get::<_, i64>(offset + 4)?.max(0),
        estimated_cost_usd: row.get::<_, f64>(offset + 5)?.max(0.0),
        request_count: row.get::<_, i64>(offset + 6)?.max(0),
        success_count: row.get::<_, i64>(offset + 7)?.max(0),
        error_count: row.get::<_, i64>(offset + 8)?.max(0),
    })
}

const DAILY_TOKEN_ROLLUP_COLUMNS: &str = "
    IFNULL(SUM(IFNULL(input_tokens, 0)), 0) AS input_tokens,
    IFNULL(SUM(IFNULL(cached_input_tokens, 0)), 0) AS cached_input_tokens,
    IFNULL(SUM(IFNULL(output_tokens, 0)), 0) AS output_tokens,
    IFNULL(SUM(IFNULL(reasoning_output_tokens, 0)), 0) AS reasoning_output_tokens,
    IFNULL(SUM(IFNULL(total_tokens, 0)), 0) AS total_tokens,
    IFNULL(SUM(IFNULL(estimated_cost, 0.0)), 0.0) AS estimated_cost_usd,
    IFNULL(SUM(IFNULL(request_count, 0)), 0) AS request_count,
    IFNULL(SUM(IFNULL(success_count, 0)), 0) AS success_count,
    IFNULL(SUM(IFNULL(error_count, 0)), 0) AS error_count";

#[derive(Debug, Default)]
struct MixedTokenStatsRange {
    rollup_start_ts: Option<i64>,
    rollup_end_ts: i64,
    live_segments: Vec<(i64, i64)>,
}

fn token_usage_rollup_from_daily_row(row: &Row<'_>, offset: usize) -> Result<TokenUsageRollup> {
    Ok(TokenUsageRollup {
        input_tokens: row.get::<_, i64>(offset)?.max(0),
        cached_input_tokens: row.get::<_, i64>(offset + 1)?.max(0),
        output_tokens: row.get::<_, i64>(offset + 2)?.max(0),
        reasoning_output_tokens: row.get::<_, i64>(offset + 3)?.max(0),
        total_tokens: row.get::<_, i64>(offset + 4)?.max(0),
        estimated_cost_usd: row.get::<_, f64>(offset + 5)?.max(0.0),
        request_count: row.get::<_, i64>(offset + 6)?.max(0),
        success_count: row.get::<_, i64>(offset + 7)?.max(0),
        error_count: row.get::<_, i64>(offset + 8)?.max(0),
    })
}

fn add_token_usage_rollup(target: &mut TokenUsageRollup, usage: &TokenUsageRollup) {
    target.input_tokens = target
        .input_tokens
        .saturating_add(usage.input_tokens.max(0));
    target.cached_input_tokens = target
        .cached_input_tokens
        .saturating_add(usage.cached_input_tokens.max(0));
    target.output_tokens = target
        .output_tokens
        .saturating_add(usage.output_tokens.max(0));
    target.reasoning_output_tokens = target
        .reasoning_output_tokens
        .saturating_add(usage.reasoning_output_tokens.max(0));
    target.total_tokens = target
        .total_tokens
        .saturating_add(usage.total_tokens.max(0));
    target.estimated_cost_usd += usage.estimated_cost_usd.max(0.0);
    target.request_count = target
        .request_count
        .saturating_add(usage.request_count.max(0));
    target.success_count = target
        .success_count
        .saturating_add(usage.success_count.max(0));
    target.error_count = target.error_count.saturating_add(usage.error_count.max(0));
}

fn add_user_rollups_to_map(
    target: &mut HashMap<String, TokenUsageRollup>,
    items: Vec<UserTokenUsageRollup>,
) {
    for item in items {
        add_token_usage_rollup(target.entry(item.user_id).or_default(), &item.usage);
    }
}

fn add_source_rollups_to_map(
    target: &mut HashMap<String, TokenUsageRollup>,
    items: Vec<SourceTokenUsageRollup>,
) {
    for item in items {
        add_token_usage_rollup(target.entry(item.source_id).or_default(), &item.usage);
    }
}

fn aligned_bucket_start(ts: i64, anchor_ts: i64, bucket_seconds: i64) -> i64 {
    let bucket_seconds = bucket_seconds.max(1);
    ts.saturating_sub((ts - anchor_ts).rem_euclid(bucket_seconds))
}

fn split_mixed_token_stats_range(
    start_ts: i64,
    end_ts: i64,
    closed_before_ts: i64,
    bucket_seconds: i64,
) -> MixedTokenStatsRange {
    let mut range = MixedTokenStatsRange::default();
    if end_ts <= start_ts {
        return range;
    }

    let bucket_seconds = bucket_seconds.max(1);
    let mut cursor = aligned_bucket_start(start_ts, closed_before_ts, bucket_seconds);
    while cursor < end_ts {
        let next = cursor.saturating_add(bucket_seconds);
        if next <= cursor {
            range.live_segments.push((start_ts, end_ts));
            return range;
        }
        let segment_start = cursor.max(start_ts);
        let segment_end = next.min(end_ts);
        if segment_end > segment_start {
            let is_closed_full_bucket = cursor < closed_before_ts
                && next <= closed_before_ts
                && segment_start == cursor
                && segment_end == next;
            if is_closed_full_bucket {
                if range.rollup_start_ts.is_none() {
                    range.rollup_start_ts = Some(cursor);
                }
                range.rollup_end_ts = next;
            } else {
                range.live_segments.push((segment_start, segment_end));
            }
        }
        cursor = next;
    }
    range
}

fn live_segments_excluding_closed_ranges(
    start_ts: i64,
    end_ts: i64,
    closed_ranges: &[(i64, i64)],
) -> Vec<(i64, i64)> {
    let mut segments = Vec::new();
    let mut cursor = start_ts;
    for &(closed_start, closed_end) in closed_ranges {
        let closed_start = closed_start.max(start_ts);
        let closed_end = closed_end.min(end_ts);
        if closed_end <= closed_start || closed_end <= cursor {
            continue;
        }
        if closed_start > cursor {
            segments.push((cursor, closed_start));
        }
        cursor = cursor.max(closed_end);
        if cursor >= end_ts {
            break;
        }
    }
    if cursor < end_ts {
        segments.push((cursor, end_ts));
    }
    segments
}

fn ranked_usage_ids_from_maps(
    today_map: &HashMap<String, TokenUsageRollup>,
    range_map: &HashMap<String, TokenUsageRollup>,
    limit: usize,
) -> Vec<String> {
    let mut ids = today_map.keys().cloned().collect::<HashSet<_>>();
    ids.extend(range_map.keys().cloned());
    let mut ids = ids.into_iter().collect::<Vec<_>>();
    ids.sort_by(|a, b| {
        let today_a = today_map
            .get(a)
            .map(|usage| usage.total_tokens)
            .unwrap_or(0);
        let today_b = today_map
            .get(b)
            .map(|usage| usage.total_tokens)
            .unwrap_or(0);
        let range_a = range_map
            .get(a)
            .map(|usage| usage.total_tokens)
            .unwrap_or(0);
        let range_b = range_map
            .get(b)
            .map(|usage| usage.total_tokens)
            .unwrap_or(0);
        today_b
            .cmp(&today_a)
            .then_with(|| range_b.cmp(&range_a))
            .then_with(|| a.cmp(b))
    });
    ids.truncate(limit);
    ids
}
fn dual_usage_ranking_select_sql(entity_alias: &str, entity_column: &str) -> String {
    format!(
        "{entity_alias}.{entity_column},
         IFNULL(today.input_tokens, 0) AS today_input_tokens,
         IFNULL(today.cached_input_tokens, 0) AS today_cached_input_tokens,
         IFNULL(today.output_tokens, 0) AS today_output_tokens,
         IFNULL(today.reasoning_output_tokens, 0) AS today_reasoning_output_tokens,
         IFNULL(today.total_tokens, 0) AS today_total_tokens,
         IFNULL(today.estimated_cost_usd, 0.0) AS today_estimated_cost_usd,
         IFNULL(today.request_count, 0) AS today_request_count,
         IFNULL(today.success_count, 0) AS today_success_count,
         IFNULL(today.error_count, 0) AS today_error_count,
         IFNULL(range_usage.input_tokens, 0) AS range_input_tokens,
         IFNULL(range_usage.cached_input_tokens, 0) AS range_cached_input_tokens,
         IFNULL(range_usage.output_tokens, 0) AS range_output_tokens,
         IFNULL(range_usage.reasoning_output_tokens, 0) AS range_reasoning_output_tokens,
         IFNULL(range_usage.total_tokens, 0) AS range_total_tokens,
         IFNULL(range_usage.estimated_cost_usd, 0.0) AS range_estimated_cost_usd,
         IFNULL(range_usage.request_count, 0) AS range_request_count,
         IFNULL(range_usage.success_count, 0) AS range_success_count,
         IFNULL(range_usage.error_count, 0) AS range_error_count"
    )
}

fn source_id_expr(source_kind: &str) -> Option<&'static str> {
    match source_kind {
        "openai_account" => Some(
            // Prefer actual_source_* written by routing. Legacy account_id is only
            // used when actual source metadata was not captured.
            "CASE
                WHEN r.actual_source_kind = 'openai_account'
                    THEN COALESCE(NULLIF(TRIM(r.actual_source_id), ''), NULLIF(TRIM(r.account_id), ''))
                WHEN r.actual_source_kind IS NULL OR TRIM(r.actual_source_kind) = ''
                    THEN NULLIF(TRIM(r.account_id), '')
                ELSE NULL
             END",
        ),
        "aggregate_api" => Some(
            // Prefer actual_source_* written by routing. Legacy aggregate API
            // context is only used when actual source metadata was not captured.
            "CASE
                WHEN r.actual_source_kind = 'aggregate_api'
                    THEN COALESCE(NULLIF(TRIM(r.actual_source_id), ''), NULLIF(TRIM(r.initial_aggregate_api_id), ''))
                WHEN r.actual_source_kind IS NULL OR TRIM(r.actual_source_kind) = ''
                    THEN NULLIF(TRIM(r.initial_aggregate_api_id), '')
                ELSE NULL
             END",
        ),
        _ => None,
    }
}

fn repeated_sqlite_text_params(values: &[String], repeats: usize) -> Vec<rusqlite::types::Value> {
    let params = sqlite_text_params(values);
    let mut out = Vec::with_capacity(params.len().saturating_mul(repeats));
    for _ in 0..repeats {
        out.extend(params.iter().cloned());
    }
    out
}

impl Storage {
    /// 函数 `insert_request_token_stat`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - self: 参数 self
    /// - stat: 参数 stat
    ///
    /// # 返回
    /// 返回函数执行结果
    pub fn insert_request_token_stat(&self, stat: &RequestTokenStat) -> Result<()> {
        self.conn.execute(
            "INSERT INTO request_token_stats (
                request_log_id, key_id, account_id, model,
                input_tokens, cached_input_tokens, output_tokens, total_tokens, reasoning_output_tokens,
                estimated_cost_usd, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            (
                stat.request_log_id,
                &stat.key_id,
                &stat.account_id,
                &stat.model,
                stat.input_tokens,
                stat.cached_input_tokens,
                stat.output_tokens,
                stat.total_tokens,
                stat.reasoning_output_tokens,
                stat.estimated_cost_usd,
                stat.created_at,
            ),
        )?;
        Ok(())
    }

    pub fn maybe_run_observability_maintenance(&self, now: i64) -> Result<()> {
        let interval = observability_maintenance_interval_secs().max(60);
        let last = LAST_OBSERVABILITY_MAINTENANCE_AT.load(Ordering::Relaxed);
        if last != 0 && now.saturating_sub(last) < interval {
            return Ok(());
        }
        if LAST_OBSERVABILITY_MAINTENANCE_AT
            .compare_exchange(last, now, Ordering::SeqCst, Ordering::Relaxed)
            .is_err()
        {
            return Ok(());
        }

        if let Err(err) = self.prune_observability_history(now) {
            LAST_OBSERVABILITY_MAINTENANCE_AT.store(last, Ordering::Relaxed);
            return Err(err);
        }
        Ok(())
    }

    pub fn prune_observability_history(&self, now: i64) -> Result<()> {
        self.prune_observability_history_with_daily_rollup_anchor(now, now)
    }

    pub fn prune_observability_history_with_daily_rollup_anchor(
        &self,
        now: i64,
        daily_rollup_anchor_ts: i64,
    ) -> Result<()> {
        self.prune_observability_history_with_daily_rollup_ranges(
            now,
            &[(
                daily_rollup_anchor_ts.saturating_sub(86_400),
                daily_rollup_anchor_ts,
            )],
        )
    }

    /// 使用显式本地自然日边界执行观测数据维护。
    ///
    /// 每个区间必须按时间升序排列且互不重叠。显式边界允许调用方保留
    /// 夏令时切换日的 23/25 小时时长，而不把本地自然日强制压成 86400 秒。
    pub fn prune_observability_history_with_daily_rollup_ranges(
        &self,
        now: i64,
        daily_rollup_ranges: &[(i64, i64)],
    ) -> Result<()> {
        let batch_limit = observability_maintenance_batch_limit();
        let mut touched = 0_usize;
        let mut daily_rolled = 0_usize;
        for &(day_start, day_end) in daily_rollup_ranges {
            if day_end <= day_start || daily_rolled >= batch_limit {
                continue;
            }
            daily_rolled =
                daily_rolled.saturating_add(self.rollup_request_token_stats_daily_before_limited(
                    day_end,
                    day_end,
                    day_end.saturating_sub(day_start),
                    batch_limit.saturating_sub(daily_rolled),
                )?);
        }
        touched = touched.saturating_add(daily_rolled);
        let closed_before_ts = daily_rollup_ranges.last().map(|(_, end)| *end).unwrap_or(0);
        let mut defer_request_log_prune = daily_rolled >= batch_limit
            || self.has_pending_daily_rollup_before(closed_before_ts)?;
        if let Some(cutoff) = retention_cutoff(now, request_token_stats_retain_days()) {
            let rolled =
                self.rollup_daily_rolled_request_token_stats_before_limited(cutoff, batch_limit)?;
            touched = touched.saturating_add(rolled);

            defer_request_log_prune = defer_request_log_prune
                || rolled >= batch_limit
                || self.has_request_token_stats_before(cutoff)?;
        }
        if !defer_request_log_prune {
            touched = touched
                .saturating_add(self.prune_request_logs_by_retention_limited(now, batch_limit)?);
        }
        touched = touched
            .saturating_add(self.prune_duplicate_usage_refresh_failed_events_limited(batch_limit)?);
        touched = touched.saturating_add(self.prune_events_by_retention_limited(now, batch_limit)?);
        touched = touched.saturating_add(self.prune_usage_snapshots_all_accounts_limited(
            super::usage::usage_snapshots_retain_per_account(),
            batch_limit,
        )?);
        if touched > 0 {
            let _ = self.conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
        }
        Ok(())
    }

    /// 返回指定边界前最早尚未写入日级汇总的 token 明细时间。
    pub fn oldest_pending_daily_rollup_ts_before(&self, cutoff_ts: i64) -> Result<Option<i64>> {
        if cutoff_ts <= 0 {
            return Ok(None);
        }
        self.conn.query_row(
            "SELECT MIN(created_at)
             FROM request_token_stats
             WHERE created_at < ?1
               AND daily_rolled_at IS NULL",
            [cutoff_ts],
            |row| row.get(0),
        )
    }

    /// 按创建时间读取指定边界前尚未日级固化的明细时间戳。
    ///
    /// 返回行数受 limit 限制，供服务层只解析本批次真实存在数据的本地日期，
    /// 避免稀疏历史数据按日历跨度枚举大量空日期。
    pub fn pending_daily_rollup_timestamps_before_limited(
        &self,
        cutoff_ts: i64,
        limit: usize,
    ) -> Result<Vec<i64>> {
        if cutoff_ts <= 0 || limit == 0 {
            return Ok(Vec::new());
        }
        let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
        let mut stmt = self.conn.prepare(
            "SELECT created_at
             FROM request_token_stats
             WHERE created_at < ?1
               AND daily_rolled_at IS NULL
             ORDER BY created_at ASC, id ASC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map((cutoff_ts, limit_i64), |row| row.get(0))?;
        rows.collect()
    }

    fn has_pending_daily_rollup_before(&self, cutoff_ts: i64) -> Result<bool> {
        Ok(self
            .oldest_pending_daily_rollup_ts_before(cutoff_ts)?
            .is_some())
    }
    pub fn rollup_request_token_stats_daily_before_limited(
        &self,
        cutoff_ts: i64,
        bucket_anchor_ts: i64,
        bucket_seconds: i64,
        limit: usize,
    ) -> Result<usize> {
        if cutoff_ts <= 0 || limit == 0 {
            return Ok(0);
        }
        let bucket_seconds = bucket_seconds.max(1);
        let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
        let now = now_ts();
        let tx = self.conn.unchecked_transaction()?;
        let day_start_expr = "t.created_at - (((t.created_at - ?1) % ?2 + ?2) % ?2)";
        let source_kind_expr = "CASE
            WHEN r.actual_source_kind IN ('openai_account', 'aggregate_api') THEN r.actual_source_kind
            WHEN (r.actual_source_kind IS NULL OR TRIM(r.actual_source_kind) = '')
                 AND NULLIF(TRIM(r.initial_aggregate_api_id), '') IS NOT NULL THEN 'aggregate_api'
            WHEN (r.actual_source_kind IS NULL OR TRIM(r.actual_source_kind) = '')
                 AND COALESCE(NULLIF(TRIM(r.account_id), ''), NULLIF(TRIM(t.account_id), '')) IS NOT NULL THEN 'openai_account'
            ELSE ''
        END";
        let source_id_expr = "CASE
            WHEN r.actual_source_kind = 'aggregate_api'
                THEN COALESCE(NULLIF(TRIM(r.actual_source_id), ''), NULLIF(TRIM(r.initial_aggregate_api_id), ''))
            WHEN r.actual_source_kind = 'openai_account'
                THEN COALESCE(NULLIF(TRIM(r.actual_source_id), ''), NULLIF(TRIM(r.account_id), ''), NULLIF(TRIM(t.account_id), ''))
            WHEN (r.actual_source_kind IS NULL OR TRIM(r.actual_source_kind) = '')
                 AND NULLIF(TRIM(r.initial_aggregate_api_id), '') IS NOT NULL
                THEN NULLIF(TRIM(r.initial_aggregate_api_id), '')
            WHEN (r.actual_source_kind IS NULL OR TRIM(r.actual_source_kind) = '')
                THEN COALESCE(NULLIF(TRIM(r.account_id), ''), NULLIF(TRIM(t.account_id), ''))
            ELSE ''
        END";
        let status_bucket_expr = "CASE
            WHEN r.status_code >= 200 AND r.status_code <= 299 THEN '2xx'
            WHEN r.status_code >= 300 AND r.status_code <= 399 THEN '3xx'
            WHEN r.status_code >= 400 AND r.status_code <= 499 THEN '4xx'
            WHEN r.status_code >= 500 THEN '5xx'
            WHEN TRIM(IFNULL(r.error, '')) <> '' THEN 'error'
            ELSE 'other'
        END";

        tx.execute(
            &format!(
                "WITH batch AS (
                    SELECT id
                    FROM request_token_stats
                    WHERE created_at < ?5
                      AND daily_rolled_at IS NULL
                    ORDER BY created_at ASC, id ASC
                    LIMIT ?4
                 ), enriched AS (
                    SELECT
                        {day_start_expr} AS day_start,
                        COALESCE(NULLIF(TRIM(t.key_id), ''), '') AS key_id,
                        COALESCE(NULLIF(TRIM(t.account_id), ''), '') AS account_id,
                        COALESCE({source_kind_expr}, '') AS source_kind,
                        COALESCE({source_id_expr}, '') AS source_id,
                        COALESCE({USER_OWNER_EXPR}, '') AS user_id,
                        COALESCE(NULLIF(TRIM(t.model), ''), '') AS model,
                        {status_bucket_expr} AS status_bucket,
                        CASE WHEN t.input_tokens > 0 THEN t.input_tokens ELSE 0 END AS input_tokens,
                        CASE WHEN t.cached_input_tokens > 0 THEN t.cached_input_tokens ELSE 0 END AS cached_input_tokens,
                        CASE WHEN t.output_tokens > 0 THEN t.output_tokens ELSE 0 END AS output_tokens,
                        CASE WHEN t.reasoning_output_tokens > 0 THEN t.reasoning_output_tokens ELSE 0 END AS reasoning_output_tokens,
                        {token_total} AS total_tokens,
                        CASE WHEN t.estimated_cost_usd > 0 THEN t.estimated_cost_usd ELSE 0.0 END AS estimated_cost,
                        r.status_code AS status_code,
                        r.error AS error
                    FROM request_token_stats t
                    JOIN batch ON batch.id = t.id
                    LEFT JOIN request_logs r ON r.id = t.request_log_id
                    {USER_OWNER_JOINS}
                 )
                 INSERT INTO request_token_stat_daily_rollups (
                    day_start, key_id, account_id, source_kind, source_id, user_id, model, status_bucket,
                    input_tokens, cached_input_tokens, output_tokens, total_tokens, reasoning_output_tokens,
                    estimated_cost, request_count, success_count, error_count, source_rows, updated_at
                 )
                 SELECT
                    day_start,
                    key_id,
                    account_id,
                    source_kind,
                    source_id,
                    user_id,
                    model,
                    status_bucket,
                    IFNULL(SUM(input_tokens), 0),
                    IFNULL(SUM(cached_input_tokens), 0),
                    IFNULL(SUM(output_tokens), 0),
                    IFNULL(SUM(total_tokens), 0),
                    IFNULL(SUM(reasoning_output_tokens), 0),
                    IFNULL(SUM(estimated_cost), 0.0),
                    COUNT(1),
                    IFNULL(SUM(CASE WHEN status_code >= 200 AND status_code <= 299 THEN 1 ELSE 0 END), 0),
                    IFNULL(SUM(CASE WHEN IFNULL(status_code, 0) >= 400 OR TRIM(IFNULL(error, '')) <> '' THEN 1 ELSE 0 END), 0),
                    COUNT(1),
                    ?3
                 FROM enriched
                 GROUP BY day_start, key_id, account_id, source_kind, source_id, user_id, model, status_bucket
                 ON CONFLICT(day_start, key_id, account_id, source_kind, source_id, user_id, model, status_bucket) DO UPDATE SET
                    input_tokens = request_token_stat_daily_rollups.input_tokens + excluded.input_tokens,
                    cached_input_tokens = request_token_stat_daily_rollups.cached_input_tokens + excluded.cached_input_tokens,
                    output_tokens = request_token_stat_daily_rollups.output_tokens + excluded.output_tokens,
                    total_tokens = request_token_stat_daily_rollups.total_tokens + excluded.total_tokens,
                    reasoning_output_tokens = request_token_stat_daily_rollups.reasoning_output_tokens + excluded.reasoning_output_tokens,
                    estimated_cost = request_token_stat_daily_rollups.estimated_cost + excluded.estimated_cost,
                    request_count = request_token_stat_daily_rollups.request_count + excluded.request_count,
                    success_count = request_token_stat_daily_rollups.success_count + excluded.success_count,
                    error_count = request_token_stat_daily_rollups.error_count + excluded.error_count,
                    source_rows = request_token_stat_daily_rollups.source_rows + excluded.source_rows,
                    updated_at = excluded.updated_at",
                token_total = token_total_sql_expr(),
            ),
            (bucket_anchor_ts, bucket_seconds, now, limit_i64, cutoff_ts),
        )?;
        let updated = tx.execute(
            "UPDATE request_token_stats
             SET daily_rolled_at = ?2
             WHERE id IN (
                SELECT id
                FROM request_token_stats
                WHERE created_at < ?1
                  AND daily_rolled_at IS NULL
                ORDER BY created_at ASC, id ASC
                LIMIT ?3
             )",
            (cutoff_ts, now, limit_i64),
        )?;
        tx.commit()?;
        Ok(updated)
    }
    pub fn rollup_all_request_token_stats(&self) -> Result<usize> {
        self.rollup_request_token_stats_before(i64::MAX)
    }

    pub fn rollup_request_token_stats_before(&self, cutoff_ts: i64) -> Result<usize> {
        self.rollup_request_token_stats_before_limited(cutoff_ts, usize::MAX)
    }

    fn has_request_token_stats_before(&self, cutoff_ts: i64) -> Result<bool> {
        self.conn.query_row(
            "SELECT EXISTS (
                SELECT 1
                FROM request_token_stats
                WHERE created_at < ?1
                LIMIT 1
             )",
            (cutoff_ts,),
            |row| row.get::<_, i64>(0).map(|value| value != 0),
        )
    }

    fn rollup_daily_rolled_request_token_stats_before_limited(
        &self,
        cutoff_ts: i64,
        limit: usize,
    ) -> Result<usize> {
        self.rollup_request_token_stats_before_limited_filtered(cutoff_ts, limit, true)
    }

    pub fn rollup_request_token_stats_before_limited(
        &self,
        cutoff_ts: i64,
        limit: usize,
    ) -> Result<usize> {
        self.rollup_request_token_stats_before_limited_filtered(cutoff_ts, limit, false)
    }

    fn rollup_request_token_stats_before_limited_filtered(
        &self,
        cutoff_ts: i64,
        limit: usize,
        only_daily_rolled: bool,
    ) -> Result<usize> {
        if limit == 0 {
            return Ok(0);
        }
        let now = now_ts();
        let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
        let only_daily_rolled_flag = i64::from(only_daily_rolled);
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            &format!(
                "WITH batch AS (
                    SELECT id
                    FROM request_token_stats
                    WHERE created_at < ?1
                      AND (?4 = 0 OR daily_rolled_at IS NOT NULL)
                    ORDER BY created_at ASC, id ASC
                    LIMIT ?3
                 )
                 INSERT INTO request_token_stat_rollups (
                    key_id, account_id, model,
                    input_tokens, cached_input_tokens, output_tokens, total_tokens,
                    reasoning_output_tokens, estimated_cost_usd, source_rows, updated_at
                 )
                 SELECT
                    COALESCE(NULLIF(TRIM(key_id), ''), ''),
                    COALESCE(NULLIF(TRIM(account_id), ''), ''),
                    COALESCE(NULLIF(TRIM(model), ''), ''),
                    IFNULL(SUM(CASE WHEN input_tokens > 0 THEN input_tokens ELSE 0 END), 0),
                    IFNULL(SUM(CASE WHEN cached_input_tokens > 0 THEN cached_input_tokens ELSE 0 END), 0),
                    IFNULL(SUM(CASE WHEN output_tokens > 0 THEN output_tokens ELSE 0 END), 0),
                    IFNULL(SUM({token_total}), 0),
                    IFNULL(SUM(CASE WHEN reasoning_output_tokens > 0 THEN reasoning_output_tokens ELSE 0 END), 0),
                    IFNULL(SUM(CASE WHEN estimated_cost_usd > 0 THEN estimated_cost_usd ELSE 0 END), 0.0),
                    COUNT(1),
                    ?2
                 FROM request_token_stats t
                 JOIN batch ON batch.id = t.id
                 GROUP BY
                    COALESCE(NULLIF(TRIM(key_id), ''), ''),
                    COALESCE(NULLIF(TRIM(account_id), ''), ''),
                    COALESCE(NULLIF(TRIM(model), ''), '')
                 ON CONFLICT(key_id, account_id, model) DO UPDATE SET
                    input_tokens = request_token_stat_rollups.input_tokens + excluded.input_tokens,
                    cached_input_tokens = request_token_stat_rollups.cached_input_tokens + excluded.cached_input_tokens,
                    output_tokens = request_token_stat_rollups.output_tokens + excluded.output_tokens,
                    total_tokens = request_token_stat_rollups.total_tokens + excluded.total_tokens,
                    reasoning_output_tokens = request_token_stat_rollups.reasoning_output_tokens + excluded.reasoning_output_tokens,
                    estimated_cost_usd = request_token_stat_rollups.estimated_cost_usd + excluded.estimated_cost_usd,
                    source_rows = request_token_stat_rollups.source_rows + excluded.source_rows,
                    updated_at = excluded.updated_at",
                token_total = token_total_sql_expr(),
            ),
            (cutoff_ts, now, limit_i64, only_daily_rolled_flag),
        )?;
        let deleted = tx.execute(
            "DELETE FROM request_token_stats
             WHERE id IN (
                SELECT id
                FROM request_token_stats
                WHERE created_at < ?1
                  AND (?3 = 0 OR daily_rolled_at IS NOT NULL)
                ORDER BY created_at ASC, id ASC
                LIMIT ?2
             )",
            (cutoff_ts, limit_i64, only_daily_rolled_flag),
        )?;
        tx.commit()?;
        Ok(deleted)
    }
    pub fn summarize_request_token_stats_between(
        &self,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<RequestLogTodaySummary> {
        let mut stmt = self.conn.prepare(
            "SELECT
                IFNULL(SUM(input_tokens), 0),
                IFNULL(SUM(cached_input_tokens), 0),
                IFNULL(SUM(output_tokens), 0),
                IFNULL(SUM(reasoning_output_tokens), 0),
                IFNULL(SUM(estimated_cost_usd), 0.0)
             FROM request_token_stats
             WHERE created_at >= ?1 AND created_at < ?2",
        )?;
        let mut rows = stmt.query((start_ts, end_ts))?;
        if let Some(row) = rows.next()? {
            return Ok(RequestLogTodaySummary {
                input_tokens: row.get(0)?,
                cached_input_tokens: row.get(1)?,
                output_tokens: row.get(2)?,
                reasoning_output_tokens: row.get(3)?,
                estimated_cost_usd: row.get(4)?,
            });
        }
        Ok(RequestLogTodaySummary {
            input_tokens: 0,
            cached_input_tokens: 0,
            output_tokens: 0,
            reasoning_output_tokens: 0,
            estimated_cost_usd: 0.0,
        })
    }

    pub fn summarize_request_token_stats_by_key(&self) -> Result<Vec<ApiKeyTokenUsageSummary>> {
        let mut stmt = self.conn.prepare(&format!(
            "WITH all_stats AS (
                SELECT
                    key_id,
                    input_tokens,
                    cached_input_tokens,
                    output_tokens,
                    total_tokens,
                    estimated_cost_usd
                FROM request_token_stats
                UNION ALL
                SELECT
                    NULLIF(key_id, '') AS key_id,
                    input_tokens,
                    cached_input_tokens,
                    output_tokens,
                    total_tokens,
                    estimated_cost_usd
                FROM request_token_stat_rollups
             )
             SELECT
                key_id,
                IFNULL(SUM({token_total}), 0) AS total_tokens,
                IFNULL(SUM(estimated_cost_usd), 0.0) AS estimated_cost_usd
             FROM all_stats
             WHERE key_id IS NOT NULL AND TRIM(key_id) <> ''
             GROUP BY key_id
             ORDER BY total_tokens DESC, key_id ASC",
            token_total = token_total_sql_expr(),
        ))?;
        let mut rows = stmt.query([])?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(ApiKeyTokenUsageSummary {
                key_id: row.get(0)?,
                total_tokens: row.get(1)?,
                estimated_cost_usd: row.get(2)?,
            });
        }
        Ok(items)
    }

    pub fn summarize_request_token_stats_by_key_ids(
        &self,
        key_ids: &[String],
    ) -> Result<Vec<ApiKeyTokenUsageSummary>> {
        if key_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = sqlite_placeholders(key_ids.len());
        let mut stmt = self.conn.prepare(&format!(
            "WITH all_stats AS (
                 SELECT
                     key_id,
                     input_tokens,
                     cached_input_tokens,
                     output_tokens,
                     total_tokens,
                     estimated_cost_usd
                 FROM request_token_stats
                 WHERE key_id IN ({placeholders})
                 UNION ALL
                 SELECT
                     NULLIF(key_id, '') AS key_id,
                     input_tokens,
                     cached_input_tokens,
                     output_tokens,
                     total_tokens,
                     estimated_cost_usd
                 FROM request_token_stat_rollups
                 WHERE key_id IN ({placeholders})
              )
              SELECT
                 key_id,
                 IFNULL(SUM({token_total}), 0) AS total_tokens,
                 IFNULL(SUM(estimated_cost_usd), 0.0) AS estimated_cost_usd
              FROM all_stats
              WHERE key_id IS NOT NULL AND TRIM(key_id) <> ''
              GROUP BY key_id
              ORDER BY total_tokens DESC, key_id ASC",
            token_total = token_total_sql_expr(),
        ))?;
        let mut rows = stmt.query(params_from_iter(repeated_sqlite_text_params(key_ids, 2)))?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(ApiKeyTokenUsageSummary {
                key_id: row.get(0)?,
                total_tokens: row.get(1)?,
                estimated_cost_usd: row.get(2)?,
            });
        }
        Ok(items)
    }

    pub fn summarize_request_token_stats_by_model(
        &self,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
    ) -> Result<Vec<TokenUsageSummary>> {
        let include_rollups = start_ts.is_none() && end_ts.is_none();
        let sql = if include_rollups {
            format!(
                "WITH all_stats AS (
                    SELECT
                        model,
                        input_tokens,
                        cached_input_tokens,
                        output_tokens,
                        reasoning_output_tokens,
                        total_tokens,
                        estimated_cost_usd
                    FROM request_token_stats
                    UNION ALL
                    SELECT
                        NULLIF(model, '') AS model,
                        input_tokens,
                        cached_input_tokens,
                        output_tokens,
                        reasoning_output_tokens,
                        total_tokens,
                        estimated_cost_usd
                    FROM request_token_stat_rollups
                 )
                 SELECT
                    COALESCE(NULLIF(TRIM(model), ''), 'unknown') AS normalized_model,
                    IFNULL(SUM(input_tokens), 0) AS input_tokens,
                    IFNULL(SUM(cached_input_tokens), 0) AS cached_input_tokens,
                    IFNULL(SUM(output_tokens), 0) AS output_tokens,
                    IFNULL(SUM(reasoning_output_tokens), 0) AS reasoning_output_tokens,
                    IFNULL(SUM({token_total}), 0) AS total_tokens,
                    IFNULL(SUM(estimated_cost_usd), 0.0) AS estimated_cost_usd
                 FROM all_stats
                 GROUP BY normalized_model
                 ORDER BY total_tokens DESC, normalized_model ASC",
                token_total = token_total_sql_expr(),
            )
        } else {
            format!(
                "SELECT
                    COALESCE(NULLIF(TRIM(model), ''), 'unknown') AS normalized_model,
                    IFNULL(SUM(input_tokens), 0) AS input_tokens,
                    IFNULL(SUM(cached_input_tokens), 0) AS cached_input_tokens,
                    IFNULL(SUM(output_tokens), 0) AS output_tokens,
                    IFNULL(SUM(reasoning_output_tokens), 0) AS reasoning_output_tokens,
                    IFNULL(SUM({token_total}), 0) AS total_tokens,
                    IFNULL(SUM(estimated_cost_usd), 0.0) AS estimated_cost_usd
                 FROM request_token_stats
                 WHERE (?1 IS NULL OR created_at >= ?1)
                   AND (?2 IS NULL OR created_at < ?2)
                 GROUP BY normalized_model
                 ORDER BY total_tokens DESC, normalized_model ASC",
                token_total = token_total_sql_expr(),
            )
        };
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = if include_rollups {
            stmt.query([])?
        } else {
            stmt.query((start_ts, end_ts))?
        };
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(TokenUsageSummary {
                model: row.get(0)?,
                input_tokens: row.get::<_, i64>(1)?.max(0),
                cached_input_tokens: row.get::<_, i64>(2)?.max(0),
                output_tokens: row.get::<_, i64>(3)?.max(0),
                reasoning_output_tokens: row.get::<_, i64>(4)?.max(0),
                total_tokens: row.get::<_, i64>(5)?.max(0),
                estimated_cost_usd: row.get::<_, f64>(6)?.max(0.0),
            });
        }
        Ok(items)
    }

    pub fn summarize_request_token_stats_by_key_and_model(
        &self,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
    ) -> Result<Vec<ApiKeyModelTokenUsageSummary>> {
        let include_rollups = start_ts.is_none() && end_ts.is_none();
        let sql = if include_rollups {
            format!(
                "WITH all_stats AS (
                    SELECT
                        key_id,
                        model,
                        input_tokens,
                        cached_input_tokens,
                        output_tokens,
                        reasoning_output_tokens,
                        total_tokens,
                        estimated_cost_usd
                    FROM request_token_stats
                    UNION ALL
                    SELECT
                        NULLIF(key_id, '') AS key_id,
                        NULLIF(model, '') AS model,
                        input_tokens,
                        cached_input_tokens,
                        output_tokens,
                        reasoning_output_tokens,
                        total_tokens,
                        estimated_cost_usd
                    FROM request_token_stat_rollups
                 )
                 SELECT
                    key_id,
                    COALESCE(NULLIF(TRIM(model), ''), 'unknown') AS normalized_model,
                    IFNULL(SUM(input_tokens), 0) AS input_tokens,
                    IFNULL(SUM(cached_input_tokens), 0) AS cached_input_tokens,
                    IFNULL(SUM(output_tokens), 0) AS output_tokens,
                    IFNULL(SUM(reasoning_output_tokens), 0) AS reasoning_output_tokens,
                    IFNULL(SUM({token_total}), 0) AS total_tokens,
                    IFNULL(SUM(estimated_cost_usd), 0.0) AS estimated_cost_usd
                 FROM all_stats
                 WHERE key_id IS NOT NULL AND TRIM(key_id) <> ''
                 GROUP BY key_id, normalized_model
                 ORDER BY total_tokens DESC, key_id ASC, normalized_model ASC",
                token_total = token_total_sql_expr(),
            )
        } else {
            format!(
                "SELECT
                    key_id,
                    COALESCE(NULLIF(TRIM(model), ''), 'unknown') AS normalized_model,
                    IFNULL(SUM(input_tokens), 0) AS input_tokens,
                    IFNULL(SUM(cached_input_tokens), 0) AS cached_input_tokens,
                    IFNULL(SUM(output_tokens), 0) AS output_tokens,
                    IFNULL(SUM(reasoning_output_tokens), 0) AS reasoning_output_tokens,
                    IFNULL(SUM({token_total}), 0) AS total_tokens,
                    IFNULL(SUM(estimated_cost_usd), 0.0) AS estimated_cost_usd
                 FROM request_token_stats
                 WHERE key_id IS NOT NULL AND TRIM(key_id) <> ''
                   AND (?1 IS NULL OR created_at >= ?1)
                   AND (?2 IS NULL OR created_at < ?2)
                 GROUP BY key_id, normalized_model
                 ORDER BY total_tokens DESC, key_id ASC, normalized_model ASC",
                token_total = token_total_sql_expr(),
            )
        };
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = if include_rollups {
            stmt.query([])?
        } else {
            stmt.query((start_ts, end_ts))?
        };
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(ApiKeyModelTokenUsageSummary {
                key_id: row.get(0)?,
                model: row.get(1)?,
                input_tokens: row.get::<_, i64>(2)?.max(0),
                cached_input_tokens: row.get::<_, i64>(3)?.max(0),
                output_tokens: row.get::<_, i64>(4)?.max(0),
                reasoning_output_tokens: row.get::<_, i64>(5)?.max(0),
                total_tokens: row.get::<_, i64>(6)?.max(0),
                estimated_cost_usd: row.get::<_, f64>(7)?.max(0.0),
            });
        }
        Ok(items)
    }

    pub fn summarize_request_token_stats_by_key_ids_and_model(
        &self,
        key_ids: &[String],
        start_ts: Option<i64>,
        end_ts: Option<i64>,
    ) -> Result<Vec<ApiKeyModelTokenUsageSummary>> {
        if key_ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = sqlite_placeholders(key_ids.len());
        let include_rollups = start_ts.is_none() && end_ts.is_none();
        let sql = if include_rollups {
            format!(
                "WITH all_stats AS (
                    SELECT
                        key_id,
                        model,
                        input_tokens,
                        cached_input_tokens,
                        output_tokens,
                        reasoning_output_tokens,
                        total_tokens,
                        estimated_cost_usd
                    FROM request_token_stats
                    WHERE key_id IN ({placeholders})
                    UNION ALL
                    SELECT
                        NULLIF(key_id, '') AS key_id,
                        NULLIF(model, '') AS model,
                        input_tokens,
                        cached_input_tokens,
                        output_tokens,
                        reasoning_output_tokens,
                        total_tokens,
                        estimated_cost_usd
                    FROM request_token_stat_rollups
                    WHERE key_id IN ({placeholders})
                 )
                 SELECT
                    key_id,
                    COALESCE(NULLIF(TRIM(model), ''), 'unknown') AS normalized_model,
                    IFNULL(SUM(input_tokens), 0) AS input_tokens,
                    IFNULL(SUM(cached_input_tokens), 0) AS cached_input_tokens,
                    IFNULL(SUM(output_tokens), 0) AS output_tokens,
                    IFNULL(SUM(reasoning_output_tokens), 0) AS reasoning_output_tokens,
                    IFNULL(SUM({token_total}), 0) AS total_tokens,
                    IFNULL(SUM(estimated_cost_usd), 0.0) AS estimated_cost_usd
                 FROM all_stats
                 WHERE key_id IS NOT NULL AND TRIM(key_id) <> ''
                 GROUP BY key_id, normalized_model
                 ORDER BY total_tokens DESC, key_id ASC, normalized_model ASC",
                token_total = token_total_sql_expr(),
            )
        } else {
            format!(
                "SELECT
                    key_id,
                    COALESCE(NULLIF(TRIM(model), ''), 'unknown') AS normalized_model,
                    IFNULL(SUM(input_tokens), 0) AS input_tokens,
                    IFNULL(SUM(cached_input_tokens), 0) AS cached_input_tokens,
                    IFNULL(SUM(output_tokens), 0) AS output_tokens,
                    IFNULL(SUM(reasoning_output_tokens), 0) AS reasoning_output_tokens,
                    IFNULL(SUM({token_total}), 0) AS total_tokens,
                    IFNULL(SUM(estimated_cost_usd), 0.0) AS estimated_cost_usd
                 FROM request_token_stats
                 WHERE key_id IN ({placeholders})
                   AND (? IS NULL OR created_at >= ?)
                   AND (? IS NULL OR created_at < ?)
                 GROUP BY key_id, normalized_model
                 ORDER BY total_tokens DESC, key_id ASC, normalized_model ASC",
                token_total = token_total_sql_expr(),
            )
        };
        let mut params = if include_rollups {
            repeated_sqlite_text_params(key_ids, 2)
        } else {
            sqlite_text_params(key_ids)
        };
        if !include_rollups {
            params.push(start_ts.map_or(
                rusqlite::types::Value::Null,
                rusqlite::types::Value::Integer,
            ));
            params.push(start_ts.map_or(
                rusqlite::types::Value::Null,
                rusqlite::types::Value::Integer,
            ));
            params.push(end_ts.map_or(
                rusqlite::types::Value::Null,
                rusqlite::types::Value::Integer,
            ));
            params.push(end_ts.map_or(
                rusqlite::types::Value::Null,
                rusqlite::types::Value::Integer,
            ));
        }
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params_from_iter(params))?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(ApiKeyModelTokenUsageSummary {
                key_id: row.get(0)?,
                model: row.get(1)?,
                input_tokens: row.get::<_, i64>(2)?.max(0),
                cached_input_tokens: row.get::<_, i64>(3)?.max(0),
                output_tokens: row.get::<_, i64>(4)?.max(0),
                reasoning_output_tokens: row.get::<_, i64>(5)?.max(0),
                total_tokens: row.get::<_, i64>(6)?.max(0),
                estimated_cost_usd: row.get::<_, f64>(7)?.max(0.0),
            });
        }
        Ok(items)
    }

    pub fn summarize_request_token_stats_daily(
        &self,
        start_ts: i64,
        end_ts: i64,
        bucket_seconds: i64,
    ) -> Result<Vec<DailyTokenUsageRollup>> {
        if end_ts <= start_ts {
            return Ok(Vec::new());
        }
        let bucket_seconds = bucket_seconds.max(1);
        // 直接使用 t.created_at 以命中索引 idx_request_token_stats_created_at
        // request_token_stats.request_log_id 是 NOT NULL + UNIQUE，created_at 也是 NOT NULL
        // LEFT JOIN 保证所有 token_stats 记录都能被包含，无需 COALESCE
        let sql = format!(
            "SELECT
                ?1 + CAST((t.created_at - ?1) / ?3 AS INTEGER) * ?3 AS bucket_start,
                MIN(?1 + (CAST((t.created_at - ?1) / ?3 AS INTEGER) + 1) * ?3, ?2) AS bucket_end,
                {TOKEN_ROLLUP_COLUMNS}
             FROM request_token_stats t
             LEFT JOIN request_logs r ON r.id = t.request_log_id
             WHERE t.created_at >= ?1 AND t.created_at < ?2
             GROUP BY bucket_start
             ORDER BY bucket_start ASC"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![start_ts, end_ts, bucket_seconds])?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(DailyTokenUsageRollup {
                day_start_ts: row.get(0)?,
                day_end_ts: row.get(1)?,
                usage: token_usage_rollup_from_row(row, 2)?,
            });
        }
        Ok(items)
    }

    pub fn summarize_request_token_stats_by_user_between(
        &self,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<Vec<UserTokenUsageRollup>> {
        if end_ts <= start_ts {
            return Ok(Vec::new());
        }
        let sql = format!(
            "SELECT
                {USER_OWNER_EXPR} AS user_id,
                {TOKEN_ROLLUP_COLUMNS}
             FROM request_logs r
             LEFT JOIN request_token_stats t ON t.request_log_id = r.id
             {USER_OWNER_JOINS}
             WHERE r.created_at >= ?1 AND r.created_at < ?2
               AND {USER_OWNER_EXPR} IS NOT NULL
             GROUP BY user_id
             ORDER BY total_tokens DESC, user_id ASC"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![start_ts, end_ts])?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(UserTokenUsageRollup {
                user_id: row.get(0)?,
                usage: token_usage_rollup_from_row(row, 1)?,
            });
        }
        Ok(items)
    }

    pub fn summarize_request_token_stats_user_ranking_between(
        &self,
        today_start_ts: i64,
        today_end_ts: i64,
        range_start_ts: i64,
        range_end_ts: i64,
        limit: usize,
    ) -> Result<Vec<UserTokenUsageRanking>> {
        if limit == 0 || today_end_ts <= today_start_ts || range_end_ts <= range_start_ts {
            return Ok(Vec::new());
        }
        let sql = format!(
            "WITH today AS (
                SELECT
                    {USER_OWNER_EXPR} AS user_id,
                    {TOKEN_ROLLUP_COLUMNS}
                 FROM request_logs r
                 LEFT JOIN request_token_stats t ON t.request_log_id = r.id
                 {USER_OWNER_JOINS}
                 WHERE r.created_at >= ?1 AND r.created_at < ?2
                   AND {USER_OWNER_EXPR} IS NOT NULL
                 GROUP BY user_id
             ),
             range_usage AS (
                SELECT
                    {USER_OWNER_EXPR} AS user_id,
                    {TOKEN_ROLLUP_COLUMNS}
                 FROM request_logs r
                 LEFT JOIN request_token_stats t ON t.request_log_id = r.id
                 {USER_OWNER_JOINS}
                 WHERE r.created_at >= ?3 AND r.created_at < ?4
                   AND {USER_OWNER_EXPR} IS NOT NULL
                 GROUP BY user_id
             ),
             ranked_ids AS (
                SELECT user_id FROM today
                UNION
                SELECT user_id FROM range_usage
             )
             SELECT
                {select_columns}
             FROM ranked_ids ranked
             LEFT JOIN today ON today.user_id = ranked.user_id
             LEFT JOIN range_usage ON range_usage.user_id = ranked.user_id
             ORDER BY IFNULL(today.total_tokens, 0) DESC,
                      IFNULL(range_usage.total_tokens, 0) DESC,
                      ranked.user_id ASC
             LIMIT ?5",
            select_columns = dual_usage_ranking_select_sql("ranked", "user_id"),
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![
            today_start_ts,
            today_end_ts,
            range_start_ts,
            range_end_ts,
            limit as i64
        ])?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(UserTokenUsageRanking {
                user_id: row.get(0)?,
                today_usage: token_usage_rollup_from_row(row, 1)?,
                range_usage: token_usage_rollup_from_row(row, 10)?,
            });
        }
        Ok(items)
    }

    pub fn summarize_request_token_stats_for_user_between(
        &self,
        user_id: &str,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<TokenUsageRollup> {
        if end_ts <= start_ts || user_id.trim().is_empty() {
            return Ok(TokenUsageRollup::default());
        }
        let sql = format!(
            "SELECT
                {TOKEN_ROLLUP_COLUMNS}
             FROM request_logs r
             LEFT JOIN request_token_stats t ON t.request_log_id = r.id
             {USER_OWNER_JOINS}
             WHERE r.created_at >= ?1 AND r.created_at < ?2
               AND {USER_OWNER_EXPR} = ?3"
        );
        self.conn
            .query_row(&sql, params![start_ts, end_ts, user_id.trim()], |row| {
                token_usage_rollup_from_row(row, 0)
            })
    }

    pub fn summarize_request_token_stats_daily_for_user(
        &self,
        user_id: &str,
        start_ts: i64,
        end_ts: i64,
        bucket_seconds: i64,
    ) -> Result<Vec<DailyTokenUsageRollup>> {
        if end_ts <= start_ts || user_id.trim().is_empty() {
            return Ok(Vec::new());
        }
        let bucket_seconds = bucket_seconds.max(1);
        let sql = format!(
            "SELECT
                ?1 + CAST((r.created_at - ?1) / ?3 AS INTEGER) * ?3 AS bucket_start,
                MIN(?1 + (CAST((r.created_at - ?1) / ?3 AS INTEGER) + 1) * ?3, ?2) AS bucket_end,
                {TOKEN_ROLLUP_COLUMNS}
             FROM request_logs r
             LEFT JOIN request_token_stats t ON t.request_log_id = r.id
             {USER_OWNER_JOINS}
             WHERE r.created_at >= ?1 AND r.created_at < ?2
               AND {USER_OWNER_EXPR} = ?4
             GROUP BY bucket_start
             ORDER BY bucket_start ASC"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![start_ts, end_ts, bucket_seconds, user_id.trim()])?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(DailyTokenUsageRollup {
                day_start_ts: row.get(0)?,
                day_end_ts: row.get(1)?,
                usage: token_usage_rollup_from_row(row, 2)?,
            });
        }
        Ok(items)
    }

    pub fn summarize_request_token_stats_by_source_between(
        &self,
        source_kind: &str,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<Vec<SourceTokenUsageRollup>> {
        if end_ts <= start_ts {
            return Ok(Vec::new());
        }
        let Some(source_id_expr) = source_id_expr(source_kind) else {
            return Ok(Vec::new());
        };
        let sql = format!(
            "SELECT
                {source_id_expr} AS source_id,
                {TOKEN_ROLLUP_COLUMNS}
             FROM request_logs r
             LEFT JOIN request_token_stats t ON t.request_log_id = r.id
             WHERE r.created_at >= ?1 AND r.created_at < ?2
               AND {source_id_expr} IS NOT NULL
             GROUP BY source_id
             ORDER BY total_tokens DESC, source_id ASC"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![start_ts, end_ts])?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(SourceTokenUsageRollup {
                source_kind: source_kind.to_string(),
                source_id: row.get(0)?,
                usage: token_usage_rollup_from_row(row, 1)?,
            });
        }
        Ok(items)
    }

    pub fn summarize_request_token_stats_source_ranking_between(
        &self,
        source_kind: &str,
        today_start_ts: i64,
        today_end_ts: i64,
        range_start_ts: i64,
        range_end_ts: i64,
        limit: usize,
    ) -> Result<Vec<SourceTokenUsageRanking>> {
        if limit == 0 || today_end_ts <= today_start_ts || range_end_ts <= range_start_ts {
            return Ok(Vec::new());
        }
        let Some(source_id_expr) = source_id_expr(source_kind) else {
            return Ok(Vec::new());
        };
        let sql = format!(
            "WITH today AS (
                SELECT
                    {source_id_expr} AS source_id,
                    {TOKEN_ROLLUP_COLUMNS}
                 FROM request_logs r
                 LEFT JOIN request_token_stats t ON t.request_log_id = r.id
                 WHERE r.created_at >= ?1 AND r.created_at < ?2
                   AND {source_id_expr} IS NOT NULL
                 GROUP BY source_id
             ),
             range_usage AS (
                SELECT
                    {source_id_expr} AS source_id,
                    {TOKEN_ROLLUP_COLUMNS}
                 FROM request_logs r
                 LEFT JOIN request_token_stats t ON t.request_log_id = r.id
                 WHERE r.created_at >= ?3 AND r.created_at < ?4
                   AND {source_id_expr} IS NOT NULL
                 GROUP BY source_id
             ),
             ranked_ids AS (
                SELECT source_id FROM today
                UNION
                SELECT source_id FROM range_usage
             )
             SELECT
                {select_columns}
             FROM ranked_ids ranked
             LEFT JOIN today ON today.source_id = ranked.source_id
             LEFT JOIN range_usage ON range_usage.source_id = ranked.source_id
             ORDER BY IFNULL(today.total_tokens, 0) DESC,
                      IFNULL(range_usage.total_tokens, 0) DESC,
                      ranked.source_id ASC
             LIMIT ?5",
            select_columns = dual_usage_ranking_select_sql("ranked", "source_id"),
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![
            today_start_ts,
            today_end_ts,
            range_start_ts,
            range_end_ts,
            limit as i64
        ])?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(SourceTokenUsageRanking {
                source_kind: source_kind.to_string(),
                source_id: row.get(0)?,
                today_usage: token_usage_rollup_from_row(row, 1)?,
                range_usage: token_usage_rollup_from_row(row, 10)?,
            });
        }
        Ok(items)
    }

    fn query_request_token_stat_daily_rollup_usage_between(
        &self,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<TokenUsageRollup> {
        if end_ts <= start_ts {
            return Ok(TokenUsageRollup::default());
        }
        let sql = format!(
            "SELECT {DAILY_TOKEN_ROLLUP_COLUMNS}
             FROM request_token_stat_daily_rollups
             WHERE day_start >= ?1 AND day_start < ?2"
        );
        self.conn.query_row(&sql, params![start_ts, end_ts], |row| {
            token_usage_rollup_from_daily_row(row, 0)
        })
    }

    fn query_request_token_stat_daily_rollup_users_between(
        &self,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<Vec<UserTokenUsageRollup>> {
        if end_ts <= start_ts {
            return Ok(Vec::new());
        }
        let sql = format!(
            "SELECT user_id, {DAILY_TOKEN_ROLLUP_COLUMNS}
             FROM request_token_stat_daily_rollups
             WHERE day_start >= ?1 AND day_start < ?2
               AND TRIM(user_id) <> ''
             GROUP BY user_id
             ORDER BY total_tokens DESC, user_id ASC"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![start_ts, end_ts])?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(UserTokenUsageRollup {
                user_id: row.get(0)?,
                usage: token_usage_rollup_from_daily_row(row, 1)?,
            });
        }
        Ok(items)
    }

    fn query_request_token_stat_daily_rollup_sources_between(
        &self,
        source_kind: &str,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<Vec<SourceTokenUsageRollup>> {
        if end_ts <= start_ts || !matches!(source_kind, "openai_account" | "aggregate_api") {
            return Ok(Vec::new());
        }
        let sql = format!(
            "SELECT source_id, {DAILY_TOKEN_ROLLUP_COLUMNS}
             FROM request_token_stat_daily_rollups
             WHERE day_start >= ?1 AND day_start < ?2
               AND source_kind = ?3
               AND TRIM(source_id) <> ''
             GROUP BY source_id
             ORDER BY total_tokens DESC, source_id ASC"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![start_ts, end_ts, source_kind])?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(SourceTokenUsageRollup {
                source_kind: source_kind.to_string(),
                source_id: row.get(0)?,
                usage: token_usage_rollup_from_daily_row(row, 1)?,
            });
        }
        Ok(items)
    }

    fn summarize_request_token_stats_daily_unrolled(
        &self,
        start_ts: i64,
        end_ts: i64,
        bucket_seconds: i64,
    ) -> Result<Vec<DailyTokenUsageRollup>> {
        if end_ts <= start_ts {
            return Ok(Vec::new());
        }
        let bucket_seconds = bucket_seconds.max(1);
        let sql = format!(
            "SELECT
                ?1 + CAST((t.created_at - ?1) / ?3 AS INTEGER) * ?3 AS bucket_start,
                MIN(?1 + (CAST((t.created_at - ?1) / ?3 AS INTEGER) + 1) * ?3, ?2) AS bucket_end,
                {TOKEN_ROLLUP_COLUMNS}
             FROM request_token_stats t
             LEFT JOIN request_logs r ON r.id = t.request_log_id
             WHERE t.created_at >= ?1 AND t.created_at < ?2
               AND t.daily_rolled_at IS NULL
             GROUP BY bucket_start
             ORDER BY bucket_start ASC"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![start_ts, end_ts, bucket_seconds])?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(DailyTokenUsageRollup {
                day_start_ts: row.get(0)?,
                day_end_ts: row.get(1)?,
                usage: token_usage_rollup_from_row(row, 2)?,
            });
        }
        Ok(items)
    }

    fn summarize_request_token_stats_by_user_between_unrolled(
        &self,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<Vec<UserTokenUsageRollup>> {
        if end_ts <= start_ts {
            return Ok(Vec::new());
        }
        let sql = format!(
            "SELECT
                {USER_OWNER_EXPR} AS user_id,
                {TOKEN_ROLLUP_COLUMNS}
             FROM request_logs r
             LEFT JOIN request_token_stats t ON t.request_log_id = r.id
             {USER_OWNER_JOINS}
             WHERE r.created_at >= ?1 AND r.created_at < ?2
               AND (t.id IS NULL OR t.daily_rolled_at IS NULL)
               AND {USER_OWNER_EXPR} IS NOT NULL
             GROUP BY user_id
             ORDER BY total_tokens DESC, user_id ASC"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![start_ts, end_ts])?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(UserTokenUsageRollup {
                user_id: row.get(0)?,
                usage: token_usage_rollup_from_row(row, 1)?,
            });
        }
        Ok(items)
    }

    fn summarize_request_token_stats_by_source_between_unrolled(
        &self,
        source_kind: &str,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<Vec<SourceTokenUsageRollup>> {
        if end_ts <= start_ts {
            return Ok(Vec::new());
        }
        let Some(source_id_expr) = source_id_expr(source_kind) else {
            return Ok(Vec::new());
        };
        let sql = format!(
            "SELECT
                {source_id_expr} AS source_id,
                {TOKEN_ROLLUP_COLUMNS}
             FROM request_logs r
             LEFT JOIN request_token_stats t ON t.request_log_id = r.id
             WHERE r.created_at >= ?1 AND r.created_at < ?2
               AND (t.id IS NULL OR t.daily_rolled_at IS NULL)
               AND {source_id_expr} IS NOT NULL
             GROUP BY source_id
             ORDER BY total_tokens DESC, source_id ASC"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![start_ts, end_ts])?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(SourceTokenUsageRollup {
                source_kind: source_kind.to_string(),
                source_id: row.get(0)?,
                usage: token_usage_rollup_from_row(row, 1)?,
            });
        }
        Ok(items)
    }
    pub fn summarize_request_token_stats_daily_mixed(
        &self,
        start_ts: i64,
        end_ts: i64,
        bucket_seconds: i64,
        closed_before_ts: i64,
    ) -> Result<Vec<DailyTokenUsageRollup>> {
        if end_ts <= start_ts {
            return Ok(Vec::new());
        }
        let bucket_seconds = bucket_seconds.max(1);
        let mut items = Vec::new();
        let mut cursor = start_ts;
        while cursor < end_ts {
            let next = cursor.saturating_add(bucket_seconds).min(end_ts);
            if next <= cursor {
                break;
            }
            let range =
                split_mixed_token_stats_range(cursor, next, closed_before_ts, bucket_seconds);
            let mut usage = TokenUsageRollup::default();
            if range.rollup_start_ts == Some(cursor) && range.rollup_end_ts == next {
                let daily_usage =
                    self.query_request_token_stat_daily_rollup_usage_between(cursor, next)?;
                add_token_usage_rollup(&mut usage, &daily_usage);
                if let Some(unrolled_usage) = self
                    .summarize_request_token_stats_daily_unrolled(cursor, next, bucket_seconds)?
                    .into_iter()
                    .next()
                    .map(|item| item.usage)
                {
                    add_token_usage_rollup(&mut usage, &unrolled_usage);
                }
            } else if let Some(live_usage) = self
                .summarize_request_token_stats_daily(cursor, next, bucket_seconds)?
                .into_iter()
                .next()
                .map(|item| item.usage)
            {
                add_token_usage_rollup(&mut usage, &live_usage);
            }
            items.push(DailyTokenUsageRollup {
                day_start_ts: cursor,
                day_end_ts: next,
                usage,
            });
            cursor = next;
        }
        Ok(items)
    }

    /// 按显式本地日区间汇总 token 使用量，避免 DST 日被固定秒数切错。
    pub fn summarize_request_token_stats_daily_mixed_ranges(
        &self,
        day_ranges: &[(i64, i64)],
        closed_day_ranges: &[(i64, i64)],
    ) -> Result<Vec<DailyTokenUsageRollup>> {
        let closed_days = closed_day_ranges.iter().copied().collect::<HashSet<_>>();
        let mut items = Vec::with_capacity(day_ranges.len());
        for &(day_start, day_end) in day_ranges {
            if day_end <= day_start {
                continue;
            }
            let mut usage = TokenUsageRollup::default();
            if closed_days.contains(&(day_start, day_end)) {
                add_token_usage_rollup(
                    &mut usage,
                    &self
                        .query_request_token_stat_daily_rollup_usage_between(day_start, day_end)?,
                );
                if let Some(unrolled_usage) = self
                    .summarize_request_token_stats_daily_unrolled(
                        day_start,
                        day_end,
                        day_end.saturating_sub(day_start),
                    )?
                    .into_iter()
                    .next()
                    .map(|item| item.usage)
                {
                    add_token_usage_rollup(&mut usage, &unrolled_usage);
                }
            } else if let Some(live_usage) = self
                .summarize_request_token_stats_daily(
                    day_start,
                    day_end,
                    day_end.saturating_sub(day_start),
                )?
                .into_iter()
                .next()
                .map(|item| item.usage)
            {
                add_token_usage_rollup(&mut usage, &live_usage);
            }
            items.push(DailyTokenUsageRollup {
                day_start_ts: day_start,
                day_end_ts: day_end,
                usage,
            });
        }
        Ok(items)
    }

    /// 使用显式已关闭本地日边界合并用户维度的日级汇总与存量明细。
    pub fn summarize_request_token_stats_by_user_between_mixed_ranges(
        &self,
        start_ts: i64,
        end_ts: i64,
        closed_day_ranges: &[(i64, i64)],
    ) -> Result<Vec<UserTokenUsageRollup>> {
        if end_ts <= start_ts {
            return Ok(Vec::new());
        }
        let mut usage_by_user = HashMap::new();
        for &(day_start, day_end) in closed_day_ranges {
            if day_start < start_ts || day_end > end_ts || day_end <= day_start {
                continue;
            }
            add_user_rollups_to_map(
                &mut usage_by_user,
                self.query_request_token_stat_daily_rollup_users_between(day_start, day_end)?,
            );
            add_user_rollups_to_map(
                &mut usage_by_user,
                self.summarize_request_token_stats_by_user_between_unrolled(day_start, day_end)?,
            );
        }
        for (segment_start, segment_end) in
            live_segments_excluding_closed_ranges(start_ts, end_ts, closed_day_ranges)
        {
            add_user_rollups_to_map(
                &mut usage_by_user,
                self.summarize_request_token_stats_by_user_between(segment_start, segment_end)?,
            );
        }
        let mut items = usage_by_user
            .into_iter()
            .map(|(user_id, usage)| UserTokenUsageRollup { user_id, usage })
            .collect::<Vec<_>>();
        items.sort_by(|a, b| {
            b.usage
                .total_tokens
                .cmp(&a.usage.total_tokens)
                .then_with(|| a.user_id.cmp(&b.user_id))
        });
        Ok(items)
    }

    /// 使用显式已关闭本地日边界合并来源维度的日级汇总与存量明细。
    pub fn summarize_request_token_stats_by_source_between_mixed_ranges(
        &self,
        source_kind: &str,
        start_ts: i64,
        end_ts: i64,
        closed_day_ranges: &[(i64, i64)],
    ) -> Result<Vec<SourceTokenUsageRollup>> {
        if end_ts <= start_ts {
            return Ok(Vec::new());
        }
        let mut usage_by_source = HashMap::new();
        for &(day_start, day_end) in closed_day_ranges {
            if day_start < start_ts || day_end > end_ts || day_end <= day_start {
                continue;
            }
            add_source_rollups_to_map(
                &mut usage_by_source,
                self.query_request_token_stat_daily_rollup_sources_between(
                    source_kind,
                    day_start,
                    day_end,
                )?,
            );
            add_source_rollups_to_map(
                &mut usage_by_source,
                self.summarize_request_token_stats_by_source_between_unrolled(
                    source_kind,
                    day_start,
                    day_end,
                )?,
            );
        }
        for (segment_start, segment_end) in
            live_segments_excluding_closed_ranges(start_ts, end_ts, closed_day_ranges)
        {
            add_source_rollups_to_map(
                &mut usage_by_source,
                self.summarize_request_token_stats_by_source_between(
                    source_kind,
                    segment_start,
                    segment_end,
                )?,
            );
        }
        let mut items = usage_by_source
            .into_iter()
            .map(|(source_id, usage)| SourceTokenUsageRollup {
                source_kind: source_kind.to_string(),
                source_id,
                usage,
            })
            .collect::<Vec<_>>();
        items.sort_by(|a, b| {
            b.usage
                .total_tokens
                .cmp(&a.usage.total_tokens)
                .then_with(|| a.source_id.cmp(&b.source_id))
        });
        Ok(items)
    }

    pub fn summarize_request_token_stats_by_user_between_mixed(
        &self,
        start_ts: i64,
        end_ts: i64,
        closed_before_ts: i64,
    ) -> Result<Vec<UserTokenUsageRollup>> {
        if end_ts <= start_ts {
            return Ok(Vec::new());
        }
        let range = split_mixed_token_stats_range(start_ts, end_ts, closed_before_ts, 86_400);
        let mut usage_by_user = HashMap::new();
        if let Some(rollup_start) = range.rollup_start_ts {
            add_user_rollups_to_map(
                &mut usage_by_user,
                self.query_request_token_stat_daily_rollup_users_between(
                    rollup_start,
                    range.rollup_end_ts,
                )?,
            );
            add_user_rollups_to_map(
                &mut usage_by_user,
                self.summarize_request_token_stats_by_user_between_unrolled(
                    rollup_start,
                    range.rollup_end_ts,
                )?,
            );
        }
        for (segment_start, segment_end) in range.live_segments {
            add_user_rollups_to_map(
                &mut usage_by_user,
                self.summarize_request_token_stats_by_user_between(segment_start, segment_end)?,
            );
        }
        let mut items = usage_by_user
            .into_iter()
            .map(|(user_id, usage)| UserTokenUsageRollup { user_id, usage })
            .collect::<Vec<_>>();
        items.sort_by(|a, b| {
            b.usage
                .total_tokens
                .cmp(&a.usage.total_tokens)
                .then_with(|| a.user_id.cmp(&b.user_id))
        });
        Ok(items)
    }

    pub fn summarize_request_token_stats_user_ranking_between_mixed(
        &self,
        today_start_ts: i64,
        today_end_ts: i64,
        range_start_ts: i64,
        range_end_ts: i64,
        closed_before_ts: i64,
        limit: usize,
    ) -> Result<Vec<UserTokenUsageRanking>> {
        if limit == 0 || today_end_ts <= today_start_ts || range_end_ts <= range_start_ts {
            return Ok(Vec::new());
        }
        let mut today_map = self
            .summarize_request_token_stats_by_user_between_mixed(
                today_start_ts,
                today_end_ts,
                closed_before_ts,
            )?
            .into_iter()
            .map(|item| (item.user_id, item.usage))
            .collect::<HashMap<_, _>>();
        let mut range_map = self
            .summarize_request_token_stats_by_user_between_mixed(
                range_start_ts,
                range_end_ts,
                closed_before_ts,
            )?
            .into_iter()
            .map(|item| (item.user_id, item.usage))
            .collect::<HashMap<_, _>>();
        Ok(ranked_usage_ids_from_maps(&today_map, &range_map, limit)
            .into_iter()
            .map(|user_id| UserTokenUsageRanking {
                today_usage: today_map.remove(&user_id).unwrap_or_default(),
                range_usage: range_map.remove(&user_id).unwrap_or_default(),
                user_id,
            })
            .collect())
    }

    pub fn summarize_request_token_stats_by_source_between_mixed(
        &self,
        source_kind: &str,
        start_ts: i64,
        end_ts: i64,
        closed_before_ts: i64,
    ) -> Result<Vec<SourceTokenUsageRollup>> {
        if end_ts <= start_ts {
            return Ok(Vec::new());
        }
        let range = split_mixed_token_stats_range(start_ts, end_ts, closed_before_ts, 86_400);
        let mut usage_by_source = HashMap::new();
        if let Some(rollup_start) = range.rollup_start_ts {
            add_source_rollups_to_map(
                &mut usage_by_source,
                self.query_request_token_stat_daily_rollup_sources_between(
                    source_kind,
                    rollup_start,
                    range.rollup_end_ts,
                )?,
            );
            add_source_rollups_to_map(
                &mut usage_by_source,
                self.summarize_request_token_stats_by_source_between_unrolled(
                    source_kind,
                    rollup_start,
                    range.rollup_end_ts,
                )?,
            );
        }
        for (segment_start, segment_end) in range.live_segments {
            add_source_rollups_to_map(
                &mut usage_by_source,
                self.summarize_request_token_stats_by_source_between(
                    source_kind,
                    segment_start,
                    segment_end,
                )?,
            );
        }
        let mut items = usage_by_source
            .into_iter()
            .map(|(source_id, usage)| SourceTokenUsageRollup {
                source_kind: source_kind.to_string(),
                source_id,
                usage,
            })
            .collect::<Vec<_>>();
        items.sort_by(|a, b| {
            b.usage
                .total_tokens
                .cmp(&a.usage.total_tokens)
                .then_with(|| a.source_id.cmp(&b.source_id))
        });
        Ok(items)
    }

    pub fn summarize_request_token_stats_source_ranking_between_mixed(
        &self,
        source_kind: &str,
        today_start_ts: i64,
        today_end_ts: i64,
        range_start_ts: i64,
        range_end_ts: i64,
        closed_before_ts: i64,
        limit: usize,
    ) -> Result<Vec<SourceTokenUsageRanking>> {
        if limit == 0 || today_end_ts <= today_start_ts || range_end_ts <= range_start_ts {
            return Ok(Vec::new());
        }
        let mut today_map = self
            .summarize_request_token_stats_by_source_between_mixed(
                source_kind,
                today_start_ts,
                today_end_ts,
                closed_before_ts,
            )?
            .into_iter()
            .map(|item| (item.source_id, item.usage))
            .collect::<HashMap<_, _>>();
        let mut range_map = self
            .summarize_request_token_stats_by_source_between_mixed(
                source_kind,
                range_start_ts,
                range_end_ts,
                closed_before_ts,
            )?
            .into_iter()
            .map(|item| (item.source_id, item.usage))
            .collect::<HashMap<_, _>>();
        Ok(ranked_usage_ids_from_maps(&today_map, &range_map, limit)
            .into_iter()
            .map(|source_id| SourceTokenUsageRanking {
                source_kind: source_kind.to_string(),
                today_usage: today_map.remove(&source_id).unwrap_or_default(),
                range_usage: range_map.remove(&source_id).unwrap_or_default(),
                source_id,
            })
            .collect())
    }
    pub(super) fn ensure_request_token_stats_daily_rollup_marker(&self) -> Result<()> {
        self.ensure_column("request_token_stats", "daily_rolled_at", "INTEGER")?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_request_token_stats_daily_rollup_pending
             ON request_token_stats(daily_rolled_at, created_at, id)",
            [],
        )?;
        Ok(())
    }
    pub(super) fn ensure_request_token_stats_table(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS request_token_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                request_log_id INTEGER NOT NULL,
                key_id TEXT,
                account_id TEXT,
                model TEXT,
                input_tokens INTEGER,
                cached_input_tokens INTEGER,
                output_tokens INTEGER,
                total_tokens INTEGER,
                reasoning_output_tokens INTEGER,
                estimated_cost_usd REAL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_request_token_stats_request_log_id
             ON request_token_stats(request_log_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_request_token_stats_created_at
             ON request_token_stats(created_at DESC)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_request_token_stats_account_id_created_at
             ON request_token_stats(account_id, created_at DESC)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_request_token_stats_key_id_created_at
             ON request_token_stats(key_id, created_at DESC)",
            [],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS request_token_stat_rollups (
                key_id TEXT NOT NULL DEFAULT '',
                account_id TEXT NOT NULL DEFAULT '',
                model TEXT NOT NULL DEFAULT '',
                input_tokens INTEGER NOT NULL DEFAULT 0,
                cached_input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                total_tokens INTEGER NOT NULL DEFAULT 0,
                reasoning_output_tokens INTEGER NOT NULL DEFAULT 0,
                estimated_cost_usd REAL NOT NULL DEFAULT 0.0,
                source_rows INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (key_id, account_id, model)
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_request_token_stat_rollups_key_id
             ON request_token_stat_rollups(key_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_request_token_stat_rollups_model
             ON request_token_stat_rollups(model)",
            [],
        )?;
        self.ensure_column("request_token_stats", "total_tokens", "INTEGER")?;
        self.ensure_request_token_stats_daily_rollup_marker()?;

        if self.has_column("request_logs", "input_tokens")? {
            self.conn.execute(
                "INSERT OR IGNORE INTO request_token_stats (
                    request_log_id, key_id, account_id, model,
                    input_tokens, cached_input_tokens, output_tokens, total_tokens, reasoning_output_tokens,
                    estimated_cost_usd, created_at
                 )
                 SELECT
                    id, key_id, account_id, model,
                    input_tokens, cached_input_tokens, output_tokens, NULL, reasoning_output_tokens,
                    estimated_cost_usd, created_at
                 FROM request_logs
                 WHERE input_tokens IS NOT NULL
                    OR cached_input_tokens IS NOT NULL
                    OR output_tokens IS NOT NULL
                    OR reasoning_output_tokens IS NOT NULL
                    OR estimated_cost_usd IS NOT NULL",
                [],
            )?;
        }

        // 初始化日级 rollup 表（migration 074）
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS request_token_stat_daily_rollups (
                day_start INTEGER NOT NULL,
                key_id TEXT NOT NULL DEFAULT '',
                account_id TEXT NOT NULL DEFAULT '',
                source_kind TEXT NOT NULL DEFAULT '',
                source_id TEXT NOT NULL DEFAULT '',
                user_id TEXT NOT NULL DEFAULT '',
                model TEXT NOT NULL DEFAULT '',
                status_bucket TEXT NOT NULL DEFAULT '',
                input_tokens INTEGER NOT NULL DEFAULT 0,
                cached_input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                total_tokens INTEGER NOT NULL DEFAULT 0,
                reasoning_output_tokens INTEGER NOT NULL DEFAULT 0,
                estimated_cost REAL NOT NULL DEFAULT 0.0,
                request_count INTEGER NOT NULL DEFAULT 0,
                success_count INTEGER NOT NULL DEFAULT 0,
                error_count INTEGER NOT NULL DEFAULT 0,
                source_rows INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (day_start, key_id, account_id, source_kind, source_id, user_id, model, status_bucket)
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_request_token_stat_daily_rollups_day_account
             ON request_token_stat_daily_rollups(day_start, account_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_request_token_stat_daily_rollups_day_user
             ON request_token_stat_daily_rollups(day_start, user_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_request_token_stat_daily_rollups_day_source
             ON request_token_stat_daily_rollups(day_start, source_kind, source_id)",
            [],
        )?;

        Ok(())
    }

    /// 函数 `insert_request_token_stat_daily_rollup`
    ///
    /// 作者: CCD-Opus
    ///
    /// 时间: 2026-06-22
    ///
    /// # 参数
    /// - self: Storage 实例
    /// - rollup: 日级 rollup 记录
    ///
    /// # 返回
    /// 返回函数执行结果
    pub fn insert_request_token_stat_daily_rollup(
        &self,
        rollup: &RequestTokenStatDailyRollup,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO request_token_stat_daily_rollups (
                day_start, key_id, account_id, source_kind, source_id, user_id, model, status_bucket,
                input_tokens, cached_input_tokens, output_tokens, total_tokens, reasoning_output_tokens,
                estimated_cost, request_count, success_count, error_count, source_rows, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
             ON CONFLICT(day_start, key_id, account_id, source_kind, source_id, user_id, model, status_bucket) DO UPDATE SET
                input_tokens = request_token_stat_daily_rollups.input_tokens + excluded.input_tokens,
                cached_input_tokens = request_token_stat_daily_rollups.cached_input_tokens + excluded.cached_input_tokens,
                output_tokens = request_token_stat_daily_rollups.output_tokens + excluded.output_tokens,
                total_tokens = request_token_stat_daily_rollups.total_tokens + excluded.total_tokens,
                reasoning_output_tokens = request_token_stat_daily_rollups.reasoning_output_tokens + excluded.reasoning_output_tokens,
                estimated_cost = request_token_stat_daily_rollups.estimated_cost + excluded.estimated_cost,
                request_count = request_token_stat_daily_rollups.request_count + excluded.request_count,
                success_count = request_token_stat_daily_rollups.success_count + excluded.success_count,
                error_count = request_token_stat_daily_rollups.error_count + excluded.error_count,
                source_rows = request_token_stat_daily_rollups.source_rows + excluded.source_rows,
                updated_at = excluded.updated_at",
            params![
                rollup.day_start,
                &rollup.key_id,
                &rollup.account_id,
                &rollup.source_kind,
                &rollup.source_id,
                &rollup.user_id,
                &rollup.model,
                &rollup.status_bucket,
                rollup.input_tokens,
                rollup.cached_input_tokens,
                rollup.output_tokens,
                rollup.total_tokens,
                rollup.reasoning_output_tokens,
                rollup.estimated_cost,
                rollup.request_count,
                rollup.success_count,
                rollup.error_count,
                rollup.source_rows,
                rollup.updated_at,
            ],
        )?;
        Ok(())
    }

    /// 函数 `query_request_token_stat_daily_rollups`
    ///
    /// 作者: CCD-Opus
    ///
    /// 时间: 2026-06-22
    ///
    /// # 参数
    /// - self: Storage 实例
    /// - day_start: 日期起始时间戳
    ///
    /// # 返回
    /// 返回指定日期的所有 rollup 记录
    pub fn query_request_token_stat_daily_rollups(
        &self,
        day_start: i64,
    ) -> Result<Vec<RequestTokenStatDailyRollup>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                day_start, key_id, account_id, source_kind, source_id, user_id, model, status_bucket,
                input_tokens, cached_input_tokens, output_tokens, total_tokens, reasoning_output_tokens,
                estimated_cost, request_count, success_count, error_count, source_rows, updated_at
             FROM request_token_stat_daily_rollups
             WHERE day_start = ?1",
        )?;
        let mut rows = stmt.query(params![day_start])?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(RequestTokenStatDailyRollup {
                day_start: row.get(0)?,
                key_id: row.get(1)?,
                account_id: row.get(2)?,
                source_kind: row.get(3)?,
                source_id: row.get(4)?,
                user_id: row.get(5)?,
                model: row.get(6)?,
                status_bucket: row.get(7)?,
                input_tokens: row.get::<_, i64>(8)?.max(0),
                cached_input_tokens: row.get::<_, i64>(9)?.max(0),
                output_tokens: row.get::<_, i64>(10)?.max(0),
                total_tokens: row.get::<_, i64>(11)?.max(0),
                reasoning_output_tokens: row.get::<_, i64>(12)?.max(0),
                estimated_cost: row.get::<_, f64>(13)?.max(0.0),
                request_count: row.get::<_, i64>(14)?.max(0),
                success_count: row.get::<_, i64>(15)?.max(0),
                error_count: row.get::<_, i64>(16)?.max(0),
                source_rows: row.get::<_, i64>(17)?.max(0),
                updated_at: row.get(18)?,
            });
        }
        Ok(items)
    }
}
