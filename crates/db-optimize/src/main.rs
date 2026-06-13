use rusqlite::Connection;
use std::env;

fn main() {
    let db_path = env::args()
        .nth(1)
        .unwrap_or_else(|| r"D:\Apps\CodexManager\codexmanager.db".to_string());

    println!("正在连接数据库: {}", db_path);

    match Connection::open(&db_path) {
        Ok(conn) => {
            println!("执行 WAL checkpoint (TRUNCATE)...");

            // PRAGMA wal_checkpoint 返回结果，需要用 query_row
            match conn.query_row("PRAGMA wal_checkpoint(TRUNCATE);", [], |row| {
                Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?, row.get::<_, i32>(2)?))
            }) {
                Ok((busy, log, checkpointed)) => {
                    println!("WAL checkpoint 完成");
                    println!("  Busy: {}, Log: {}, Checkpointed: {}", busy, log, checkpointed);
                }
                Err(e) => {
                    eprintln!("WAL checkpoint 失败: {}", e);
                    return;
                }
            }

            // 检查页面统计
            let page_count: i64 = conn
                .query_row("PRAGMA page_count;", [], |row| row.get(0))
                .unwrap_or(0);

            let page_size: i64 = conn
                .query_row("PRAGMA page_size;", [], |row| row.get(0))
                .unwrap_or(4096);

            let freelist_count: i64 = conn
                .query_row("PRAGMA freelist_count;", [], |row| row.get(0))
                .unwrap_or(0);

            println!("\n数据库统计:");
            println!("  总页数: {}", page_count);
            println!("  页大小: {} bytes", page_size);
            println!("  空闲页: {}", freelist_count);
            println!("  总大小: {:.2} MB", (page_count * page_size) as f64 / 1024.0 / 1024.0);
            println!("  空闲空间: {:.2} MB", (freelist_count * page_size) as f64 / 1024.0 / 1024.0);

            if freelist_count > 1000 {
                println!("\n检测到较多空闲页 ({}), 建议执行 VACUUM", freelist_count);
                println!("注意：VACUUM 会锁定数据库，请在应用停止时执行");
            } else {
                println!("\n空闲页数量正常，无需 VACUUM");
            }
        }
        Err(e) => {
            eprintln!("无法打开数据库: {}", e);
        }
    }
}
