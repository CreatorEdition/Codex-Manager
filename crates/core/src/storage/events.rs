use rusqlite::{params_from_iter, types::Value, Result};
use std::collections::HashMap;

use super::{Event, Storage};

const DEFAULT_EVENTS_RETENTION_DAYS: i64 = 14;
const EVENTS_RETENTION_DAYS_ENV: &str = "CODEXMANAGER_EVENTS_RETENTION_DAYS";
const DEFAULT_USAGE_REFRESH_FAILURE_DUPLICATE_WINDOW_SECS: i64 = 6 * 60 * 60;
const USAGE_REFRESH_FAILURE_EVENT_WINDOW_SECS_ENV: &str =
    "CODEXMANAGER_USAGE_REFRESH_FAILURE_EVENT_WINDOW_SECS";

pub(super) fn events_retention_days() -> i64 {
    std::env::var(EVENTS_RETENTION_DAYS_ENV)
        .ok()
        .and_then(|raw| raw.trim().parse::<i64>().ok())
        .unwrap_or(DEFAULT_EVENTS_RETENTION_DAYS)
}

fn usage_refresh_failure_duplicate_window_secs() -> i64 {
    std::env::var(USAGE_REFRESH_FAILURE_EVENT_WINDOW_SECS_ENV)
        .ok()
        .and_then(|raw| raw.trim().parse::<i64>().ok())
        .unwrap_or(DEFAULT_USAGE_REFRESH_FAILURE_DUPLICATE_WINDOW_SECS)
}
impl Storage {
    /// 函数 `insert_event`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - self: 参数 self
    /// - event: 参数 event
    ///
    /// # 返回
    /// 返回函数执行结果
    pub fn insert_event(&self, event: &Event) -> Result<()> {
        self.conn.execute(
            "INSERT INTO events (account_id, type, message, created_at) VALUES (?1, ?2, ?3, ?4)",
            (
                &event.account_id,
                &event.event_type,
                &event.message,
                event.created_at,
            ),
        )?;
        Ok(())
    }

    /// 函数 `event_count`
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
    pub fn event_count(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(1) FROM events", [], |row| row.get(0))
    }

    pub fn prune_events_by_retention(&self, now: i64) -> Result<usize> {
        self.prune_events_by_retention_limited(now, usize::MAX)
    }

    pub fn prune_events_by_retention_limited(&self, now: i64, limit: usize) -> Result<usize> {
        let days = events_retention_days();
        if days <= 0 {
            return Ok(0);
        }
        if limit == 0 {
            return Ok(0);
        }
        let cutoff = now.saturating_sub(days.saturating_mul(86_400));
        let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
        self.conn.execute(
            "DELETE FROM events
             WHERE id IN (
                SELECT e.id
                FROM events e
                WHERE e.created_at < ?1
                  AND (
                    e.type <> 'account_status_update'
                    OR (
                      e.type = 'account_status_update'
                      AND e.account_id IS NOT NULL
                      AND EXISTS (
                        SELECT 1
                        FROM events latest
                        WHERE latest.type = 'account_status_update'
                          AND latest.account_id = e.account_id
                          AND (
                            latest.created_at > e.created_at
                            OR (
                              latest.created_at = e.created_at
                              AND latest.id > e.id
                            )
                          )
                        LIMIT 1
                      )
                    )
                  )
                ORDER BY e.created_at ASC, e.id ASC
                LIMIT ?2
             )",
            (cutoff, limit_i64),
        )
    }

    pub fn prune_duplicate_usage_refresh_failed_events_limited(
        &self,
        limit: usize,
    ) -> Result<usize> {
        if limit == 0 {
            return Ok(0);
        }
        let duplicate_window_secs = usage_refresh_failure_duplicate_window_secs();
        if duplicate_window_secs <= 0 {
            return Ok(0);
        }
        let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
        self.conn.execute(
            "DELETE FROM events
             WHERE id IN (
                SELECT e.id
                FROM events e
                WHERE e.type = 'usage_refresh_failed'
                  AND e.account_id IS NOT NULL
                  AND TRIM(e.account_id) <> ''
                  AND e.message IS NOT NULL
                  AND TRIM(e.message) <> ''
                  AND EXISTS (
                    SELECT 1
                    FROM events newer
                    WHERE newer.type = 'usage_refresh_failed'
                      AND newer.account_id = e.account_id
                      AND newer.message = e.message
                      AND newer.created_at >= e.created_at
                      AND newer.created_at <= e.created_at + ?1
                      AND (
                        newer.created_at > e.created_at
                        OR (newer.created_at = e.created_at AND newer.id > e.id)
                      )
                    LIMIT 1
                  )
                ORDER BY e.created_at ASC, e.id ASC
                LIMIT ?2
             )",
            (duplicate_window_secs, limit_i64),
        )
    }
    pub fn has_recent_usage_refresh_failure_event(
        &self,
        account_id: &str,
        error_class: &str,
        cutoff_ts: i64,
    ) -> Result<bool> {
        if account_id.trim().is_empty() || error_class.trim().is_empty() {
            return Ok(false);
        }
        let message_prefix = format!("class={} %", error_class.trim());
        self.conn.query_row(
            "SELECT EXISTS (
                SELECT 1
                FROM events
                WHERE type = 'usage_refresh_failed'
                  AND account_id = ?1
                  AND created_at >= ?2
                  AND message LIKE ?3
                LIMIT 1
             )",
            (account_id.trim(), cutoff_ts, message_prefix),
            |row| row.get::<_, i64>(0).map(|value| value != 0),
        )
    }
    /// 函数 `latest_account_status_reasons`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - self: 参数 self
    /// - account_ids: 参数 account_ids
    ///
    /// # 返回
    /// 返回函数执行结果
    pub fn latest_account_status_reasons(
        &self,
        account_ids: &[String],
    ) -> Result<HashMap<String, String>> {
        if account_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let placeholders = vec!["?"; account_ids.len()].join(", ");
        let sql = format!(
            "WITH ranked AS (
                SELECT
                    account_id,
                    message,
                    ROW_NUMBER() OVER (
                        PARTITION BY account_id
                        ORDER BY created_at DESC, id DESC
                    ) AS rn
                FROM events
                WHERE type = 'account_status_update'
                  AND account_id IN ({placeholders})
            )
            SELECT account_id, message
            FROM ranked
            WHERE rn = 1"
        );

        let params = account_ids
            .iter()
            .map(|account_id| Value::Text(account_id.clone()))
            .collect::<Vec<_>>();
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params_from_iter(params))?;
        let mut out = HashMap::new();
        while let Some(row) = rows.next()? {
            let account_id: String = row.get(0)?;
            let message: String = row.get(1)?;
            if let Some(reason) = extract_status_reason_from_event_message(&message) {
                out.insert(account_id, reason.to_string());
            }
        }
        Ok(out)
    }
}

/// 函数 `extract_status_reason_from_event_message`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - message: 参数 message
///
/// # 返回
/// 返回函数执行结果
fn extract_status_reason_from_event_message(message: &str) -> Option<&str> {
    let marker = " reason=";
    let start = message.find(marker)? + marker.len();
    let reason = message.get(start..)?.trim();
    if reason.is_empty() {
        None
    } else {
        Some(reason)
    }
}

#[cfg(test)]
#[path = "tests/events_tests.rs"]
mod tests;
