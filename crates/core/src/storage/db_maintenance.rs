use rusqlite::Result;

use super::{DatabasePageStats, Storage};

impl Storage {
    pub fn database_page_stats(&self) -> Result<DatabasePageStats> {
        let page_count = self
            .conn
            .query_row("PRAGMA page_count;", [], |row| row.get(0))?;
        let page_size = self
            .conn
            .query_row("PRAGMA page_size;", [], |row| row.get(0))?;
        let freelist_count = self
            .conn
            .query_row("PRAGMA freelist_count;", [], |row| row.get(0))?;
        Ok(DatabasePageStats {
            page_count,
            page_size,
            freelist_count,
        })
    }

    pub fn checkpoint_wal_truncate(&self) -> Result<(i32, i32, i32)> {
        self.conn
            .query_row("PRAGMA wal_checkpoint(TRUNCATE);", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
    }

    pub fn vacuum_database(&self) -> Result<()> {
        self.conn.execute_batch("VACUUM;")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_page_stats_reports_non_negative_sizes() {
        let storage = Storage::open_in_memory().expect("open storage");

        let stats = storage.database_page_stats().expect("page stats");

        assert!(stats.page_count >= 0);
        assert!(stats.page_size > 0);
        assert!(stats.freelist_count >= 0);
        assert!(stats.total_bytes() >= 0);
        assert!(stats.free_bytes() >= 0);
        assert!((0..=100).contains(&stats.free_percent()));
    }
}
