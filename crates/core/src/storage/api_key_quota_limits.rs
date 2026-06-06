use std::collections::HashMap;

use rusqlite::{params_from_iter, OptionalExtension, Result};

use super::{now_ts, sqlite_placeholders, sqlite_text_params, ApiKeyQuotaOverviewSummary, Storage};

fn quota_token_total_sql_expr() -> &'static str {
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

impl Storage {
    pub fn upsert_api_key_quota_limit(
        &self,
        key_id: &str,
        quota_limit_tokens: Option<i64>,
    ) -> Result<()> {
        let normalized = quota_limit_tokens.filter(|value| *value > 0);
        let Some(limit) = normalized else {
            self.conn.execute(
                "DELETE FROM api_key_quota_limits WHERE key_id = ?1",
                [key_id],
            )?;
            return Ok(());
        };

        let now = now_ts();
        self.conn.execute(
            "INSERT INTO api_key_quota_limits (
                key_id, quota_limit_tokens, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?3)
             ON CONFLICT(key_id) DO UPDATE SET
                quota_limit_tokens = excluded.quota_limit_tokens,
                updated_at = excluded.updated_at",
            (key_id, limit, now),
        )?;
        Ok(())
    }

    pub fn find_api_key_quota_limit(&self, key_id: &str) -> Result<Option<i64>> {
        self.conn
            .query_row(
                "SELECT quota_limit_tokens
                 FROM api_key_quota_limits
                 WHERE key_id = ?1
                 LIMIT 1",
                [key_id],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn list_api_key_quota_limits(&self) -> Result<HashMap<String, i64>> {
        let mut stmt = self.conn.prepare(
            "SELECT key_id, quota_limit_tokens
             FROM api_key_quota_limits
             WHERE quota_limit_tokens > 0",
        )?;
        let mut rows = stmt.query([])?;
        let mut out = HashMap::new();
        while let Some(row) = rows.next()? {
            out.insert(row.get(0)?, row.get(1)?);
        }
        Ok(out)
    }

    pub fn list_api_key_quota_limits_for_key_ids(
        &self,
        key_ids: &[String],
    ) -> Result<HashMap<String, i64>> {
        if key_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let placeholders = sqlite_placeholders(key_ids.len());
        let sql = format!(
            "SELECT key_id, quota_limit_tokens
             FROM api_key_quota_limits
             WHERE quota_limit_tokens > 0
               AND key_id IN ({placeholders})"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params_from_iter(sqlite_text_params(key_ids)))?;
        let mut out = HashMap::new();
        while let Some(row) = rows.next()? {
            out.insert(row.get(0)?, row.get(1)?);
        }
        Ok(out)
    }

    /// 汇总平台 Key 配额概览，避免上层读取全部 Key、quota limit 和按 Key 用量后再聚合。
    pub fn quota_api_key_overview_summary(&self) -> Result<ApiKeyQuotaOverviewSummary> {
        self.conn.query_row(
            &format!(
                "WITH key_usage AS (
                    SELECT
                        key_id,
                        IFNULL(SUM({token_total}), 0) AS total_tokens,
                        IFNULL(SUM(estimated_cost_usd), 0.0) AS estimated_cost_usd
                    FROM (
                        SELECT
                            key_id,
                            input_tokens,
                            cached_input_tokens,
                            output_tokens,
                            total_tokens,
                            estimated_cost_usd
                        FROM request_token_stats
                        WHERE key_id IS NOT NULL AND TRIM(key_id) <> ''
                        UNION ALL
                        SELECT
                            NULLIF(key_id, '') AS key_id,
                            input_tokens,
                            cached_input_tokens,
                            output_tokens,
                            total_tokens,
                            estimated_cost_usd
                        FROM request_token_stat_rollups
                        WHERE TRIM(key_id) <> ''
                    )
                    WHERE key_id IS NOT NULL AND TRIM(key_id) <> ''
                    GROUP BY key_id
                 ),
                 scoped AS (
                    SELECT
                        k.id,
                        IFNULL(l.quota_limit_tokens, 0) AS quota_limit_tokens,
                        IFNULL(u.total_tokens, 0) AS total_tokens,
                        IFNULL(u.estimated_cost_usd, 0.0) AS estimated_cost_usd
                    FROM api_keys k
                    LEFT JOIN api_key_quota_limits l
                      ON l.key_id = k.id
                     AND l.quota_limit_tokens > 0
                    LEFT JOIN key_usage u
                      ON u.key_id = k.id
                 )
                 SELECT
                    COUNT(1) AS key_count,
                    IFNULL(SUM(CASE WHEN quota_limit_tokens > 0 THEN 1 ELSE 0 END), 0) AS limited_key_count,
                    IFNULL(SUM(CASE WHEN quota_limit_tokens > 0 THEN quota_limit_tokens ELSE 0 END), 0) AS total_limit_tokens,
                    IFNULL(SUM(total_tokens), 0) AS total_used_tokens,
                    IFNULL(SUM(
                        CASE
                            WHEN quota_limit_tokens > 0
                            THEN MAX(0, quota_limit_tokens - total_tokens)
                            ELSE 0
                        END
                    ), 0) AS total_remaining_tokens,
                    IFNULL(SUM(estimated_cost_usd), 0.0) AS estimated_cost_usd
                 FROM scoped",
                token_total = quota_token_total_sql_expr(),
            ),
            [],
            |row| {
                Ok(ApiKeyQuotaOverviewSummary {
                    key_count: row.get::<_, i64>(0)?.max(0),
                    limited_key_count: row.get::<_, i64>(1)?.max(0),
                    total_limit_tokens: row.get::<_, i64>(2)?.max(0),
                    total_used_tokens: row.get::<_, i64>(3)?.max(0),
                    total_remaining_tokens: row.get::<_, i64>(4)?.max(0),
                    estimated_cost_usd: row.get::<_, f64>(5)?.max(0.0),
                })
            },
        )
    }

    pub fn api_key_total_token_usage(&self, key_id: &str) -> Result<i64> {
        let mut stmt = self.conn.prepare(
            "WITH all_stats AS (
                SELECT
                    key_id,
                    input_tokens,
                    cached_input_tokens,
                    output_tokens,
                    total_tokens
                FROM request_token_stats
                UNION ALL
                SELECT
                    NULLIF(key_id, '') AS key_id,
                    input_tokens,
                    cached_input_tokens,
                    output_tokens,
                    total_tokens
                FROM request_token_stat_rollups
             )
             SELECT
                IFNULL(
                    SUM(
                        CASE
                            WHEN total_tokens IS NOT NULL THEN
                                CASE WHEN total_tokens > 0 THEN total_tokens ELSE 0 END
                            ELSE
                                CASE
                                    WHEN IFNULL(input_tokens, 0) - IFNULL(cached_input_tokens, 0) + IFNULL(output_tokens, 0) > 0
                                        THEN IFNULL(input_tokens, 0) - IFNULL(cached_input_tokens, 0) + IFNULL(output_tokens, 0)
                                    ELSE 0
                                END
                        END
                    ),
                    0
                ) AS total_tokens
             FROM all_stats
             WHERE key_id = ?1",
        )?;
        let mut rows = stmt.query([key_id])?;
        if let Some(row) = rows.next()? {
            let total: i64 = row.get(0)?;
            return Ok(total.max(0));
        }
        Ok(0)
    }

    pub(super) fn ensure_api_key_quota_limits_table(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS api_key_quota_limits (
                key_id TEXT PRIMARY KEY REFERENCES api_keys(id) ON DELETE CASCADE,
                quota_limit_tokens INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_api_key_quota_limits_updated_at
             ON api_key_quota_limits(updated_at DESC)",
            [],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{ApiKey, RequestTokenStat};

    fn sample_api_key(id: &str, now: i64) -> ApiKey {
        ApiKey {
            id: id.to_string(),
            name: Some(id.to_string()),
            model_slug: Some("gpt-test".to_string()),
            reasoning_effort: None,
            service_tier: None,
            rotation_strategy: "account_rotation".to_string(),
            aggregate_api_id: None,
            account_plan_filter: None,
            aggregate_api_url: None,
            client_type: "codex".to_string(),
            protocol_type: "openai_compat".to_string(),
            auth_scheme: "authorization_bearer".to_string(),
            upstream_base_url: None,
            static_headers_json: None,
            key_hash: format!("hash-{id}"),
            status: "active".to_string(),
            created_at: now,
            last_used_at: None,
        }
    }

    fn token_stat(
        request_log_id: i64,
        key_id: &str,
        total_tokens: i64,
        cost: f64,
    ) -> RequestTokenStat {
        RequestTokenStat {
            request_log_id,
            key_id: Some(key_id.to_string()),
            account_id: None,
            model: Some("gpt-test".to_string()),
            input_tokens: None,
            cached_input_tokens: None,
            output_tokens: None,
            total_tokens: Some(total_tokens),
            reasoning_output_tokens: None,
            estimated_cost_usd: Some(cost),
            created_at: now_ts(),
        }
    }

    #[test]
    fn quota_api_key_overview_summary_aggregates_limits_and_usage() {
        let storage = Storage::open_in_memory().expect("open");
        storage.init().expect("init");
        let now = now_ts();

        for key_id in ["key-limited", "key-unlimited", "key-over-limit"] {
            storage
                .insert_api_key(&sample_api_key(key_id, now))
                .expect("insert api key");
        }
        storage
            .upsert_api_key_quota_limit("key-limited", Some(1_000))
            .expect("limit key-limited");
        storage
            .upsert_api_key_quota_limit("key-over-limit", Some(100))
            .expect("limit key-over-limit");
        storage
            .insert_request_token_stat(&token_stat(1, "key-limited", 250, 0.25))
            .expect("usage key-limited");
        storage
            .insert_request_token_stat(&token_stat(2, "key-unlimited", 300, 0.30))
            .expect("usage key-unlimited");
        storage
            .insert_request_token_stat(&token_stat(3, "key-over-limit", 150, 0.15))
            .expect("usage key-over-limit");

        let summary = storage
            .quota_api_key_overview_summary()
            .expect("quota api key overview");

        assert_eq!(summary.key_count, 3);
        assert_eq!(summary.limited_key_count, 2);
        assert_eq!(summary.total_limit_tokens, 1_100);
        assert_eq!(summary.total_used_tokens, 700);
        assert_eq!(summary.total_remaining_tokens, 750);
        assert!((summary.estimated_cost_usd - 0.70).abs() < f64::EPSILON);
    }
}
