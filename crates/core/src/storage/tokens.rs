use rusqlite::{params_from_iter, Result, Row};

use super::{sqlite_placeholders, sqlite_text_params, Storage, Token};

impl Storage {
    /// 函数 `insert_token`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - self: 参数 self
    /// - token: 参数 token
    ///
    /// # 返回
    /// 返回函数执行结果
    pub fn insert_token(&self, token: &Token) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tokens (account_id, id_token, access_token, refresh_token, api_key_access_token, last_refresh)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(account_id) DO UPDATE SET
                id_token = excluded.id_token,
                access_token = excluded.access_token,
                refresh_token = excluded.refresh_token,
                api_key_access_token = excluded.api_key_access_token,
                last_refresh = excluded.last_refresh",
            (
                &token.account_id,
                &token.id_token,
                &token.access_token,
                &token.refresh_token,
                &token.api_key_access_token,
                token.last_refresh,
            ),
        )?;
        Ok(())
    }

    /// 函数 `list_tokens_due_for_refresh`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - self: 参数 self
    /// - refresh_due_cutoff_ts: 参数 refresh_due_cutoff_ts
    /// - access_exp_cutoff_ts: 参数 access_exp_cutoff_ts
    /// - limit: 参数 limit
    ///
    /// # 返回
    /// 返回函数执行结果
    pub fn list_tokens_due_for_refresh(
        &self,
        refresh_due_cutoff_ts: i64,
        access_exp_cutoff_ts: i64,
        limit: usize,
    ) -> Result<Vec<Token>> {
        let mut stmt = self.conn.prepare(
            "WITH due_token_candidates AS (
                SELECT
                    tokens.account_id,
                    tokens.id_token,
                    tokens.access_token,
                    tokens.refresh_token,
                    tokens.api_key_access_token,
                    tokens.last_refresh,
                    tokens.next_refresh_at,
                    (
                        SELECT events.message
                        FROM events
                        WHERE events.type = 'account_status_update'
                          AND events.account_id = tokens.account_id
                        ORDER BY events.created_at DESC, events.id DESC
                        LIMIT 1
                    ) AS latest_status_message
                FROM tokens
                WHERE TRIM(COALESCE(refresh_token, '')) <> ''
                  AND (
                       next_refresh_at <= ?1
                       OR (
                            next_refresh_at IS NULL
                            AND access_token_exp IS NOT NULL
                            AND access_token_exp <= ?2
                        )
                       OR (
                            next_refresh_at IS NULL
                            AND access_token_exp IS NULL
                       )
                  )
             )
             SELECT account_id, id_token, access_token, refresh_token, api_key_access_token, last_refresh
             FROM due_token_candidates
             WHERE latest_status_message IS NULL
                OR (
                    LOWER(TRIM(latest_status_message)) NOT LIKE '% reason=account_deactivated'
                    AND LOWER(TRIM(latest_status_message)) NOT LIKE '% reason=workspace_deactivated'
                    AND LOWER(TRIM(latest_status_message)) NOT LIKE '% reason=deactivated_workspace'
                    AND LOWER(TRIM(latest_status_message)) NOT LIKE '% reason=refresh_token_region_blocked'
                    AND LOWER(TRIM(latest_status_message)) NOT LIKE '% reason=refresh_token_invalid:%'
                )
             ORDER BY COALESCE(next_refresh_at, 0) ASC, account_id ASC
             LIMIT ?3",
        )?;
        let mut rows = stmt.query((refresh_due_cutoff_ts, access_exp_cutoff_ts, limit as i64))?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(map_token_row(row)?);
        }
        Ok(out)
    }

    /// 函数 `update_token_refresh_schedule`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - self: 参数 self
    /// - account_id: 参数 account_id
    /// - access_token_exp: 参数 access_token_exp
    /// - next_refresh_at: 参数 next_refresh_at
    ///
    /// # 返回
    /// 返回函数执行结果
    pub fn update_token_refresh_schedule(
        &self,
        account_id: &str,
        access_token_exp: Option<i64>,
        next_refresh_at: Option<i64>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE tokens
             SET access_token_exp = ?1,
                 next_refresh_at = ?2
             WHERE account_id = ?3",
            (access_token_exp, next_refresh_at, account_id),
        )?;
        Ok(())
    }

    pub fn update_token_next_refresh_at(
        &self,
        account_id: &str,
        next_refresh_at: Option<i64>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE tokens
             SET next_refresh_at = ?1
             WHERE account_id = ?2",
            (next_refresh_at, account_id),
        )?;
        Ok(())
    }

    /// 函数 `touch_token_refresh_attempt`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - self: 参数 self
    /// - account_id: 参数 account_id
    /// - attempt_ts: 参数 attempt_ts
    ///
    /// # 返回
    /// 返回函数执行结果
    pub fn touch_token_refresh_attempt(&self, account_id: &str, attempt_ts: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE tokens
             SET last_refresh_attempt_at = ?1
             WHERE account_id = ?2",
            (attempt_ts, account_id),
        )?;
        Ok(())
    }

    /// 函数 `token_count`
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
    pub fn token_count(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(1) FROM tokens", [], |row| row.get(0))
    }

    /// 函数 `list_tokens`
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
    pub fn list_tokens(&self) -> Result<Vec<Token>> {
        let mut stmt = self.conn.prepare(
            "SELECT account_id, id_token, access_token, refresh_token, api_key_access_token, last_refresh FROM tokens",
        )?;
        let mut rows = stmt.query([])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(map_token_row(row)?);
        }
        Ok(out)
    }

    /// 函数 `list_tokens_by_account_ids`
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
    pub fn list_tokens_by_account_ids(&self, account_ids: &[String]) -> Result<Vec<Token>> {
        if account_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = sqlite_placeholders(account_ids.len());
        let sql = format!(
            "SELECT account_id, id_token, access_token, refresh_token, api_key_access_token, last_refresh
             FROM tokens
             WHERE account_id IN ({placeholders})
             ORDER BY account_id ASC"
        );
        let params = sqlite_text_params(account_ids);
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params_from_iter(params))?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(map_token_row(row)?);
        }
        Ok(out)
    }

    /// 函数 `find_token_by_account_id`
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
    pub fn find_token_by_account_id(&self, account_id: &str) -> Result<Option<Token>> {
        let mut stmt = self.conn.prepare(
            "SELECT account_id, id_token, access_token, refresh_token, api_key_access_token, last_refresh
             FROM tokens
             WHERE account_id = ?1
             LIMIT 1",
        )?;
        let mut rows = stmt.query([account_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_token_row(row)?))
        } else {
            Ok(None)
        }
    }

    /// 函数 `ensure_token_api_key_column`
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
    pub(super) fn ensure_token_api_key_column(&self) -> Result<()> {
        if self.has_column("tokens", "api_key_access_token")? {
            return Ok(());
        }
        self.conn.execute(
            "ALTER TABLE tokens ADD COLUMN api_key_access_token TEXT",
            [],
        )?;
        Ok(())
    }

    /// 函数 `ensure_token_refresh_schedule_columns`
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
    pub(super) fn ensure_token_refresh_schedule_columns(&self) -> Result<()> {
        self.ensure_column("tokens", "access_token_exp", "INTEGER")?;
        self.ensure_column("tokens", "next_refresh_at", "INTEGER")?;
        self.ensure_column("tokens", "last_refresh_attempt_at", "INTEGER")?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tokens_next_refresh_at ON tokens(next_refresh_at)",
            [],
        )?;
        Ok(())
    }
}

/// 函数 `map_token_row`
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
fn map_token_row(row: &Row<'_>) -> Result<Token> {
    Ok(Token {
        account_id: row.get(0)?,
        id_token: row.get(1)?,
        access_token: row.get(2)?,
        refresh_token: row.get(3)?,
        api_key_access_token: row.get(4)?,
        last_refresh: row.get(5)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{now_ts, Account, Event};

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

    fn sample_token(account_id: &str, now: i64) -> Token {
        Token {
            account_id: account_id.to_string(),
            id_token: "id".to_string(),
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
            api_key_access_token: None,
            last_refresh: now,
        }
    }

    #[test]
    fn list_tokens_due_for_refresh_filters_latest_blocked_status() {
        let storage = Storage::open_in_memory().expect("open");
        storage.init().expect("init");
        let now = now_ts();

        for account_id in [
            "acc-due",
            "acc-blocked",
            "acc-invalid-refresh",
            "acc-restored",
        ] {
            storage
                .insert_account(&sample_account(account_id, now))
                .expect("insert account");
            storage
                .insert_token(&sample_token(account_id, now))
                .expect("insert token");
            storage
                .update_token_refresh_schedule(account_id, None, Some(now - 1))
                .expect("schedule token");
        }

        storage
            .insert_event(&Event {
                account_id: Some("acc-blocked".to_string()),
                event_type: "account_status_update".to_string(),
                message: "status=banned reason=refresh_token_region_blocked".to_string(),
                created_at: now + 10,
            })
            .expect("insert blocked status");
        storage
            .insert_event(&Event {
                account_id: Some("acc-invalid-refresh".to_string()),
                event_type: "account_status_update".to_string(),
                message: "status=unavailable reason=refresh_token_invalid:refresh_token_reused"
                    .to_string(),
                created_at: now + 10,
            })
            .expect("insert invalid refresh status");
        storage
            .insert_event(&Event {
                account_id: Some("acc-restored".to_string()),
                event_type: "account_status_update".to_string(),
                message: "status=banned reason=workspace_deactivated".to_string(),
                created_at: now + 10,
            })
            .expect("insert old blocked status");
        storage
            .insert_event(&Event {
                account_id: Some("acc-restored".to_string()),
                event_type: "account_status_update".to_string(),
                message: "status=active reason=usage_ok".to_string(),
                created_at: now + 20,
            })
            .expect("insert restored status");

        let ids = storage
            .list_tokens_due_for_refresh(now, now, 2)
            .expect("list due tokens")
            .into_iter()
            .map(|token| token.account_id)
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["acc-due".to_string(), "acc-restored".to_string()]);
    }

    #[test]
    fn update_token_next_refresh_at_preserves_access_exp() {
        let storage = Storage::open_in_memory().expect("open");
        storage.init().expect("init");
        let now = now_ts();
        let account_id = "acc-token-next-refresh";
        storage
            .insert_account(&sample_account(account_id, now))
            .expect("insert account");
        storage
            .insert_token(&sample_token(account_id, now))
            .expect("insert token");
        storage
            .update_token_refresh_schedule(account_id, Some(now + 3_600), Some(now + 600))
            .expect("seed schedule");
        storage
            .update_token_next_refresh_at(account_id, Some(now + 21_600))
            .expect("update next refresh");

        let row = storage
            .conn
            .query_row(
                "SELECT access_token_exp, next_refresh_at FROM tokens WHERE account_id = ?1",
                [account_id],
                |row| Ok((row.get::<_, Option<i64>>(0)?, row.get::<_, Option<i64>>(1)?)),
            )
            .expect("read token schedule");
        assert_eq!(row, (Some(now + 3_600), Some(now + 21_600)));
    }

    #[test]
    fn list_tokens_due_for_refresh_respects_future_retry_after_expired_access_token() {
        let storage = Storage::open_in_memory().expect("open");
        storage.init().expect("init");
        let now = now_ts();
        let account_id = "acc-token-backoff";
        storage
            .insert_account(&sample_account(account_id, now))
            .expect("insert account");
        storage
            .insert_token(&sample_token(account_id, now))
            .expect("insert token");
        storage
            .update_token_refresh_schedule(account_id, Some(now - 60), Some(now + 21_600))
            .expect("seed failure backoff schedule");

        let due = storage
            .list_tokens_due_for_refresh(now, now + 3_600, 10)
            .expect("list due tokens");

        assert!(due.is_empty());
    }
}
