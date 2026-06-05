use rusqlite::{params_from_iter, Result, Row};

use super::{sqlite_placeholders, sqlite_text_params, Storage, UsageSnapshotRecord};

const DEFAULT_USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT: usize = 1;
const USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT_ENV: &str =
    "CODEXMANAGER_USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT";

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
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT account_id FROM usage_snapshots")?;
        let mut rows = stmt.query([])?;
        let mut account_ids = Vec::new();
        while let Some(row) = rows.next()? {
            account_ids.push(row.get::<_, String>(0)?);
        }
        drop(rows);
        drop(stmt);
        let mut removed = 0_usize;
        for account_id in account_ids {
            removed = removed
                .saturating_add(self.prune_usage_snapshots_for_account(&account_id, retain)?);
        }
        Ok(removed)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::now_ts;

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
}
