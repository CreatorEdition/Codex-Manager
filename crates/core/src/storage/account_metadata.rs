use rusqlite::{params_from_iter, Result, Row};

use super::{now_ts, sqlite_placeholders, sqlite_text_params, AccountMetadata, Storage};

impl Storage {
    /// 函数 `upsert_account_metadata`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - self: 参数 self
    /// - account_id: 参数 account_id
    /// - note: 参数 note
    /// - tags: 参数 tags
    ///
    /// # 返回
    /// 返回函数执行结果
    pub fn upsert_account_metadata(
        &self,
        account_id: &str,
        note: Option<&str>,
        tags: Option<&str>,
    ) -> Result<()> {
        let normalized_note = normalize_optional_text(note);
        let normalized_tags = normalize_optional_text(tags);
        if normalized_note.is_none() && normalized_tags.is_none() {
            self.conn.execute(
                "DELETE FROM account_metadata WHERE account_id = ?1",
                [account_id],
            )?;
            return Ok(());
        }

        self.conn.execute(
            "INSERT INTO account_metadata (account_id, note, tags, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(account_id) DO UPDATE SET
                note = excluded.note,
                tags = excluded.tags,
                updated_at = excluded.updated_at",
            (account_id, normalized_note, normalized_tags, now_ts()),
        )?;
        Ok(())
    }

    /// 函数 `find_account_metadata`
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
    pub fn find_account_metadata(&self, account_id: &str) -> Result<Option<AccountMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT account_id, note, tags, updated_at
             FROM account_metadata
             WHERE account_id = ?1
             LIMIT 1",
        )?;
        let mut rows = stmt.query([account_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_account_metadata_row(row)?))
        } else {
            Ok(None)
        }
    }

    /// 函数 `list_account_metadata`
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
    pub fn list_account_metadata(&self) -> Result<Vec<AccountMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT account_id, note, tags, updated_at
             FROM account_metadata
             ORDER BY updated_at DESC, account_id ASC",
        )?;
        let mut rows = stmt.query([])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(map_account_metadata_row(row)?);
        }
        Ok(out)
    }

    /// 函数 `list_account_metadata_by_account_ids`
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
    pub fn list_account_metadata_by_account_ids(
        &self,
        account_ids: &[String],
    ) -> Result<Vec<AccountMetadata>> {
        if account_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = sqlite_placeholders(account_ids.len());
        let sql = format!(
            "SELECT account_id, note, tags, updated_at
             FROM account_metadata
             WHERE account_id IN ({placeholders})
             ORDER BY updated_at DESC, account_id ASC"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params_from_iter(sqlite_text_params(account_ids)))?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(map_account_metadata_row(row)?);
        }
        Ok(out)
    }
}

/// 函数 `normalize_optional_text`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - value: 参数 value
///
/// # 返回
/// 返回函数执行结果
fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(ToString::to_string)
}

/// 函数 `map_account_metadata_row`
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
fn map_account_metadata_row(row: &Row<'_>) -> Result<AccountMetadata> {
    Ok(AccountMetadata {
        account_id: row.get(0)?,
        note: row.get(1)?,
        tags: row.get(2)?,
        updated_at: row.get(3)?,
    })
}
