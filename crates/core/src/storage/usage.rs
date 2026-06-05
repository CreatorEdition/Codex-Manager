use rusqlite::{params_from_iter, Result, Row};

use crate::rpc::types::UsageAggregateSummaryResult;

use super::{sqlite_placeholders, sqlite_text_params, Storage, UsageSnapshotRecord};

const DEFAULT_USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT: usize = 1;
const USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT_ENV: &str =
    "CODEXMANAGER_USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT";
const LONG_USAGE_WINDOW_MINUTES: i64 = 24 * 60 + 3;

pub(super) fn usage_snapshots_retain_per_account() -> usize {
    std::env::var(USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT_ENV)
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .unwrap_or(DEFAULT_USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT)
}

impl Storage {
    /// 函数 `insert_usage_snapshot`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - self: 参数 self
    /// - snap: 参数 snap
    ///
    /// # 返回
    /// 返回函数执行结果
    pub fn insert_usage_snapshot(&self, snap: &UsageSnapshotRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO usage_snapshots (account_id, used_percent, window_minutes, resets_at, secondary_used_percent, secondary_window_minutes, secondary_resets_at, credits_json, captured_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (
                &snap.account_id,
                snap.used_percent,
                snap.window_minutes,
                snap.resets_at,
                snap.secondary_used_percent,
                snap.secondary_window_minutes,
                snap.secondary_resets_at,
                &snap.credits_json,
                snap.captured_at,
            ),
        )?;
        Ok(())
    }

    /// 函数 `prune_usage_snapshots_for_account`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - self: 参数 self
    /// - account_id: 参数 account_id
    /// - retain: 参数 retain
    ///
    /// # 返回
    /// 返回函数执行结果
    pub fn prune_usage_snapshots_for_account(
        &self,
        account_id: &str,
        retain: usize,
    ) -> Result<usize> {
        if retain == 0 {
            return Ok(0);
        }
        self.conn.execute(
            "DELETE FROM usage_snapshots
             WHERE account_id = ?1
               AND id NOT IN (
                 SELECT id
                 FROM usage_snapshots
                 WHERE account_id = ?1
                 ORDER BY captured_at DESC, id DESC
                 LIMIT ?2
               )",
            (account_id, retain as i64),
        )
    }

    pub fn prune_usage_snapshots_all_accounts(&self, retain: usize) -> Result<usize> {
        if retain == 0 {
            return Ok(0);
        }
        self.conn.execute(
            "WITH ranked AS (
                SELECT
                    id,
                    ROW_NUMBER() OVER (
                        PARTITION BY account_id
                        ORDER BY captured_at DESC, id DESC
                    ) AS rn
                FROM usage_snapshots
            )
            DELETE FROM usage_snapshots
            WHERE id IN (
                SELECT id
                FROM ranked
                WHERE rn > ?1
            )",
            [retain as i64],
        )
    }

    /// 函数 `usage_snapshot_count_for_account`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - self: 参数 self
    /// - account_id: 参数 account_id
    ///
    /// # 返回
    /// 返回函数执行结果
    pub fn usage_snapshot_count_for_account(&self, account_id: &str) -> Result<i64> {
        self.conn.query_row(
            "SELECT COUNT(1) FROM usage_snapshots WHERE account_id = ?1",
            [account_id],
            |row| row.get(0),
        )
    }

    /// 函数 `latest_usage_snapshot`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - self: 参数 self
    ///
    /// # 返回
    /// 返回函数执行结果
    pub fn latest_usage_snapshot(&self) -> Result<Option<UsageSnapshotRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT account_id, used_percent, window_minutes, resets_at, secondary_used_percent, secondary_window_minutes, secondary_resets_at, credits_json, captured_at FROM usage_snapshots ORDER BY captured_at DESC, id DESC LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_usage_snapshot_row(row)?))
        } else {
            Ok(None)
        }
    }

    /// 函数 `latest_usage_snapshot_for_account`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - self: 参数 self
    /// - account_id: 参数 account_id
    ///
    /// # 返回
    /// 返回函数执行结果
    pub fn latest_usage_snapshot_for_account(
        &self,
        account_id: &str,
    ) -> Result<Option<UsageSnapshotRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT account_id, used_percent, window_minutes, resets_at, secondary_used_percent, secondary_window_minutes, secondary_resets_at, credits_json, captured_at
             FROM usage_snapshots
             WHERE account_id = ?1
             ORDER BY captured_at DESC, id DESC
             LIMIT 1",
        )?;
        let mut rows = stmt.query([account_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_usage_snapshot_row(row)?))
        } else {
            Ok(None)
        }
    }

    /// 函数 `latest_usage_snapshots_by_account`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - self: 参数 self
    ///
    /// # 返回
    /// 返回函数执行结果
    pub fn latest_usage_snapshots_by_account(&self) -> Result<Vec<UsageSnapshotRecord>> {
        // 中文注释：窗口函数 + 复合索引可稳定处理“同 captured_at 并发写入”场景；
        // 不这样做会依赖复杂子查询拼接，后续维护和优化都更难。
        let mut stmt = self.conn.prepare(
            "WITH ranked AS (
                SELECT
                    id,
                    account_id,
                    used_percent,
                    window_minutes,
                    resets_at,
                    secondary_used_percent,
                    secondary_window_minutes,
                    secondary_resets_at,
                    credits_json,
                    captured_at,
                    ROW_NUMBER() OVER (
                        PARTITION BY account_id
                        ORDER BY captured_at DESC, id DESC
                    ) AS rn
                FROM usage_snapshots
            )
            SELECT
                account_id,
                used_percent,
                window_minutes,
                resets_at,
                secondary_used_percent,
                secondary_window_minutes,
                secondary_resets_at,
                credits_json,
                captured_at
            FROM ranked
            WHERE rn = 1
            ORDER BY captured_at DESC, id DESC",
        )?;
        let mut rows = stmt.query([])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(map_usage_snapshot_row(row)?);
        }
        Ok(out)
    }

    /// 按最近捕获时间限量读取各账号最新用量，供兼容性无参接口使用。
    pub fn latest_usage_snapshots_by_account_limited(
        &self,
        limit: i64,
    ) -> Result<Vec<UsageSnapshotRecord>> {
        if limit <= 0 {
            return Ok(Vec::new());
        }

        let mut stmt = self.conn.prepare(
            "WITH ranked AS (
                SELECT
                    id,
                    account_id,
                    used_percent,
                    window_minutes,
                    resets_at,
                    secondary_used_percent,
                    secondary_window_minutes,
                    secondary_resets_at,
                    credits_json,
                    captured_at,
                    ROW_NUMBER() OVER (
                        PARTITION BY account_id
                        ORDER BY captured_at DESC, id DESC
                    ) AS rn
                FROM usage_snapshots
            )
            SELECT
                account_id,
                used_percent,
                window_minutes,
                resets_at,
                secondary_used_percent,
                secondary_window_minutes,
                secondary_resets_at,
                credits_json,
                captured_at
            FROM ranked
            WHERE rn = 1
            ORDER BY captured_at DESC, id DESC
            LIMIT ?1",
        )?;
        let mut rows = stmt.query([limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(map_usage_snapshot_row(row)?);
        }
        Ok(out)
    }

    /// 函数 `latest_usage_snapshots_by_account_ids`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-06-04
    ///
    /// # 参数
    /// - self: 参数 self
    /// - account_ids: 参数 account_ids
    ///
    /// # 返回
    /// 返回函数执行结果
    pub fn latest_usage_snapshots_by_account_ids(
        &self,
        account_ids: &[String],
    ) -> Result<Vec<UsageSnapshotRecord>> {
        if account_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = sqlite_placeholders(account_ids.len());
        let sql = format!(
            "WITH ranked AS (
                SELECT
                    id,
                    account_id,
                    used_percent,
                    window_minutes,
                    resets_at,
                    secondary_used_percent,
                    secondary_window_minutes,
                    secondary_resets_at,
                    credits_json,
                    captured_at,
                    ROW_NUMBER() OVER (
                        PARTITION BY account_id
                        ORDER BY captured_at DESC, id DESC
                    ) AS rn
                FROM usage_snapshots
                WHERE account_id IN ({placeholders})
            )
            SELECT
                account_id,
                used_percent,
                window_minutes,
                resets_at,
                secondary_used_percent,
                secondary_window_minutes,
                secondary_resets_at,
                credits_json,
                captured_at
            FROM ranked
            WHERE rn = 1
            ORDER BY captured_at DESC, id DESC"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params_from_iter(sqlite_text_params(account_ids)))?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(map_usage_snapshot_row(row)?);
        }
        Ok(out)
    }

    /// 在数据库内聚合账号用量摘要，避免 RPC 读取全量账号与快照后本地统计。
    pub fn usage_aggregate_summary(&self) -> Result<UsageAggregateSummaryResult> {
        self.conn.query_row(
            "WITH latest_usage AS (
                SELECT
                    account_id,
                    used_percent,
                    window_minutes,
                    secondary_used_percent,
                    secondary_window_minutes,
                    credits_json,
                    ROW_NUMBER() OVER (
                        PARTITION BY account_id
                        ORDER BY captured_at DESC, id DESC
                    ) AS rn
                FROM usage_snapshots
            ),
            scoped AS (
                SELECT
                    a.id AS account_id,
                    lu.used_percent,
                    lu.window_minutes,
                    lu.secondary_used_percent,
                    lu.secondary_window_minutes,
                    lu.credits_json,
                    CASE
                        WHEN lu.account_id IS NOT NULL
                             AND (lu.used_percent IS NOT NULL OR lu.window_minutes IS NOT NULL)
                        THEN 1 ELSE 0
                    END AS has_primary_signal,
                    CASE
                        WHEN lu.account_id IS NOT NULL
                             AND (
                                lu.secondary_used_percent IS NOT NULL
                                OR lu.secondary_window_minutes IS NOT NULL
                             )
                        THEN 1 ELSE 0
                    END AS has_secondary_signal,
                    CASE
                        WHEN lu.account_id IS NOT NULL
                             AND (lu.used_percent IS NOT NULL OR lu.window_minutes IS NOT NULL)
                             AND NOT (
                                lu.secondary_used_percent IS NOT NULL
                                OR lu.secondary_window_minutes IS NOT NULL
                             )
                             AND (
                                COALESCE(lu.window_minutes, 0) > ?1
                                OR (
                                    lu.credits_json IS NOT NULL
                                    AND json_valid(lu.credits_json)
                                    AND EXISTS (
                                        SELECT 1
                                        FROM json_tree(lu.credits_json) AS credits
                                        WHERE credits.type = 'text'
                                          AND credits.key IN (
                                            'plan_type',
                                            'planType',
                                            'subscription_tier',
                                            'subscriptionTier',
                                            'tier',
                                            'account_type',
                                            'accountType',
                                            'type'
                                          )
                                          AND LOWER(credits.value) LIKE '%free%'
                                    )
                                )
                             )
                        THEN 1 ELSE 0
                    END AS primary_belongs_to_secondary
                FROM accounts a
                LEFT JOIN latest_usage lu
                  ON lu.account_id = a.id
                 AND lu.rn = 1
            )
            SELECT
                COALESCE(SUM(
                    CASE
                        WHEN has_primary_signal = 1 AND primary_belongs_to_secondary = 0
                        THEN 1 ELSE 0
                    END
                ), 0) AS primary_bucket_count,
                COALESCE(SUM(
                    CASE
                        WHEN has_primary_signal = 1
                             AND primary_belongs_to_secondary = 0
                             AND used_percent IS NOT NULL
                        THEN 1 ELSE 0
                    END
                ), 0) AS primary_known_count,
                COALESCE(SUM(
                    CASE
                        WHEN has_primary_signal = 1
                             AND primary_belongs_to_secondary = 0
                             AND used_percent IS NOT NULL
                        THEN MAX(0.0, 100.0 - MIN(100.0, MAX(0.0, used_percent)))
                        ELSE 0.0
                    END
                ), 0.0) AS primary_remaining_total,
                COALESCE(SUM(
                    CASE
                        WHEN primary_belongs_to_secondary = 1 THEN 1 ELSE 0
                    END
                    +
                    CASE
                        WHEN has_secondary_signal = 1 THEN 1 ELSE 0
                    END
                ), 0) AS secondary_bucket_count,
                COALESCE(SUM(
                    CASE
                        WHEN primary_belongs_to_secondary = 1 AND used_percent IS NOT NULL
                        THEN 1 ELSE 0
                    END
                    +
                    CASE
                        WHEN has_secondary_signal = 1 AND secondary_used_percent IS NOT NULL
                        THEN 1 ELSE 0
                    END
                ), 0) AS secondary_known_count,
                COALESCE(SUM(
                    CASE
                        WHEN primary_belongs_to_secondary = 1 AND used_percent IS NOT NULL
                        THEN MAX(0.0, 100.0 - MIN(100.0, MAX(0.0, used_percent)))
                        ELSE 0.0
                    END
                    +
                    CASE
                        WHEN has_secondary_signal = 1 AND secondary_used_percent IS NOT NULL
                        THEN MAX(0.0, 100.0 - MIN(100.0, MAX(0.0, secondary_used_percent)))
                        ELSE 0.0
                    END
                ), 0.0) AS secondary_remaining_total
            FROM scoped",
            [LONG_USAGE_WINDOW_MINUTES],
            |row| {
                let primary_bucket_count: i64 = row.get(0)?;
                let primary_known_count: i64 = row.get(1)?;
                let primary_remaining_total: f64 = row.get(2)?;
                let secondary_bucket_count: i64 = row.get(3)?;
                let secondary_known_count: i64 = row.get(4)?;
                let secondary_remaining_total: f64 = row.get(5)?;

                Ok(UsageAggregateSummaryResult {
                    primary_bucket_count,
                    primary_known_count,
                    primary_unknown_count: (primary_bucket_count - primary_known_count).max(0),
                    primary_remain_percent: average_percent(
                        primary_remaining_total,
                        primary_known_count,
                    ),
                    secondary_bucket_count,
                    secondary_known_count,
                    secondary_unknown_count: (secondary_bucket_count - secondary_known_count)
                        .max(0),
                    secondary_remain_percent: average_percent(
                        secondary_remaining_total,
                        secondary_known_count,
                    ),
                })
            },
        )
    }

    /// 函数 `ensure_usage_secondary_columns`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - super: 参数 super
    ///
    /// # 返回
    /// 返回函数执行结果
    pub(super) fn ensure_usage_secondary_columns(&self) -> Result<()> {
        self.ensure_column("usage_snapshots", "secondary_used_percent", "REAL")?;
        self.ensure_column("usage_snapshots", "secondary_window_minutes", "INTEGER")?;
        self.ensure_column("usage_snapshots", "secondary_resets_at", "INTEGER")?;
        Ok(())
    }
}

/// 函数 `map_usage_snapshot_row`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - row: 参数 row
///
/// # 返回
/// 返回函数执行结果
fn map_usage_snapshot_row(row: &Row<'_>) -> Result<UsageSnapshotRecord> {
    Ok(UsageSnapshotRecord {
        account_id: row.get(0)?,
        used_percent: row.get(1)?,
        window_minutes: row.get(2)?,
        resets_at: row.get(3)?,
        secondary_used_percent: row.get(4)?,
        secondary_window_minutes: row.get(5)?,
        secondary_resets_at: row.get(6)?,
        credits_json: row.get(7)?,
        captured_at: row.get(8)?,
    })
}

fn average_percent(total: f64, count: i64) -> Option<i64> {
    if count <= 0 {
        return None;
    }
    Some((total / count as f64).round() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{now_ts, Account};

    fn sample_account(id: &str, now: i64) -> Account {
        Account {
            id: id.to_string(),
            label: id.to_string(),
            issuer: "issuer".to_string(),
            chatgpt_account_id: None,
            workspace_id: None,
            group_name: None,
            sort: 0,
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    fn sample_snapshot(
        account_id: &str,
        captured_at: i64,
        used_percent: f64,
    ) -> UsageSnapshotRecord {
        UsageSnapshotRecord {
            account_id: account_id.to_string(),
            used_percent: Some(used_percent),
            window_minutes: Some(300),
            resets_at: None,
            secondary_used_percent: None,
            secondary_window_minutes: None,
            secondary_resets_at: None,
            credits_json: None,
            captured_at,
        }
    }

    fn usage_snapshot(
        account_id: &str,
        captured_at: i64,
        used_percent: Option<f64>,
        window_minutes: Option<i64>,
        secondary_used_percent: Option<f64>,
        secondary_window_minutes: Option<i64>,
        credits_json: Option<&str>,
    ) -> UsageSnapshotRecord {
        UsageSnapshotRecord {
            account_id: account_id.to_string(),
            used_percent,
            window_minutes,
            resets_at: None,
            secondary_used_percent,
            secondary_window_minutes,
            secondary_resets_at: None,
            credits_json: credits_json.map(ToOwned::to_owned),
            captured_at,
        }
    }

    #[test]
    fn latest_usage_snapshots_by_account_limited_returns_recent_unique_accounts() {
        let storage = Storage::open_in_memory().expect("open");
        storage.init().expect("init");
        let now = now_ts();

        storage
            .insert_usage_snapshot(&sample_snapshot("acc-a", now, 10.0))
            .expect("insert old acc-a usage");
        storage
            .insert_usage_snapshot(&sample_snapshot("acc-a", now + 10, 20.0))
            .expect("insert latest acc-a usage");
        storage
            .insert_usage_snapshot(&sample_snapshot("acc-b", now + 20, 30.0))
            .expect("insert acc-b usage");
        storage
            .insert_usage_snapshot(&sample_snapshot("acc-c", now + 30, 40.0))
            .expect("insert acc-c usage");

        let records = storage
            .latest_usage_snapshots_by_account_limited(2)
            .expect("read limited usage snapshots");
        let ids = records
            .iter()
            .map(|record| record.account_id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["acc-c", "acc-b"]);
        assert_eq!(
            storage
                .latest_usage_snapshots_by_account_limited(0)
                .expect("zero limit")
                .len(),
            0
        );
    }

    #[test]
    fn prune_usage_snapshots_all_accounts_keeps_recent_rows_per_account() {
        let storage = Storage::open_in_memory().expect("open");
        storage.init().expect("init");
        let now = now_ts();

        for snapshot in [
            sample_snapshot("acc-a", now, 10.0),
            sample_snapshot("acc-a", now + 10, 20.0),
            sample_snapshot("acc-a", now + 10, 30.0),
            sample_snapshot("acc-b", now, 40.0),
            sample_snapshot("acc-b", now + 20, 50.0),
            sample_snapshot("acc-b", now + 30, 55.0),
            sample_snapshot("acc-c", now, 60.0),
        ] {
            storage
                .insert_usage_snapshot(&snapshot)
                .expect("insert usage snapshot");
        }

        let removed = storage
            .prune_usage_snapshots_all_accounts(2)
            .expect("prune all account usage snapshots");
        assert_eq!(removed, 2);

        assert_eq!(
            storage
                .usage_snapshot_count_for_account("acc-a")
                .expect("count acc-a"),
            2
        );
        assert_eq!(
            storage
                .usage_snapshot_count_for_account("acc-b")
                .expect("count acc-b"),
            2
        );
        assert_eq!(
            storage
                .usage_snapshot_count_for_account("acc-c")
                .expect("count acc-c"),
            1
        );

        let latest_a = storage
            .latest_usage_snapshots_by_account_ids(&["acc-a".to_string()])
            .expect("read acc-a latest");
        assert_eq!(latest_a.len(), 1);
        assert_eq!(latest_a[0].used_percent, Some(30.0));
    }

    #[test]
    fn usage_aggregate_summary_matches_bucket_semantics() {
        let storage = Storage::open_in_memory().expect("open");
        storage.init().expect("init");
        let now = now_ts();

        for account_id in [
            "acc-pro",
            "acc-free",
            "acc-unknown",
            "acc-note-free",
            "acc-nested-free",
            "acc-unused",
        ] {
            storage
                .insert_account(&sample_account(account_id, now))
                .expect("insert account");
        }
        storage
            .insert_usage_snapshot(&usage_snapshot(
                "acc-pro",
                now,
                Some(99.0),
                Some(300),
                Some(99.0),
                Some(10080),
                None,
            ))
            .expect("insert old pro usage");
        storage
            .insert_usage_snapshot(&usage_snapshot(
                "acc-pro",
                now + 10,
                Some(20.0),
                Some(300),
                Some(40.0),
                Some(10080),
                None,
            ))
            .expect("insert latest pro usage");
        storage
            .insert_usage_snapshot(&usage_snapshot(
                "acc-free",
                now + 20,
                Some(10.0),
                Some(10080),
                None,
                None,
                Some(r#"{"planType":"free"}"#),
            ))
            .expect("insert free usage");
        storage
            .insert_usage_snapshot(&usage_snapshot(
                "acc-unknown",
                now + 30,
                None,
                Some(10080),
                None,
                None,
                Some(r#"{"planType":"free"}"#),
            ))
            .expect("insert unknown usage");
        storage
            .insert_usage_snapshot(&usage_snapshot(
                "acc-note-free",
                now + 40,
                Some(30.0),
                Some(300),
                None,
                None,
                Some(r#"{"note":"free text should not classify plan"}"#),
            ))
            .expect("insert note usage");
        storage
            .insert_usage_snapshot(&usage_snapshot(
                "acc-nested-free",
                now + 50,
                Some(50.0),
                Some(300),
                None,
                None,
                Some(r#"{"nested":{"planType":"free"}} "#),
            ))
            .expect("insert nested free usage");

        let result = storage.usage_aggregate_summary().expect("usage aggregate");

        assert_eq!(result.primary_bucket_count, 2);
        assert_eq!(result.primary_known_count, 2);
        assert_eq!(result.primary_unknown_count, 0);
        assert_eq!(result.primary_remain_percent, Some(75));
        assert_eq!(result.secondary_bucket_count, 4);
        assert_eq!(result.secondary_known_count, 3);
        assert_eq!(result.secondary_unknown_count, 1);
        assert_eq!(result.secondary_remain_percent, Some(67));
    }
}
