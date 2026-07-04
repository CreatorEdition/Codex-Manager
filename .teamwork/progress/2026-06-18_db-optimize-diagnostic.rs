use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let db_path = r"D:\Apps\CodexManager\codexmanager.db";
    println!("正在连接数据库: {}", db_path);

    let conn = Connection::open(db_path)?;

    // 执行 WAL checkpoint
    println!("执行 WAL checkpoint (TRUNCATE)...");
    conn.execute("PRAGMA wal_checkpoint(TRUNCATE);", [])?;
    println!("WAL checkpoint 完成");

    // 检查页面统计
    let mut stmt = conn.prepare("PRAGMA page_count;")?;
    let page_count: i64 = stmt.query_row([], |row| row.get(0))?;

    let mut stmt = conn.prepare("PRAGMA page_size;")?;
    let page_size: i64 = stmt.query_row([], |row| row.get(0))?;

    let mut stmt = conn.prepare("PRAGMA freelist_count;")?;
    let freelist_count: i64 = stmt.query_row([], |row| row.get(0))?;

    println!("\n数据库统计:");
    println!("  总页数: {}", page_count);
    println!("  页大小: {} bytes", page_size);
    println!("  空闲页: {}", freelist_count);
    println!("  总大小: {:.2} MB", (page_count * page_size) as f64 / 1024.0 / 1024.0);
    println!("  空闲空间: {:.2} MB", (freelist_count * page_size) as f64 / 1024.0 / 1024.0);

    if freelist_count > 1000 {
        println!("\n检测到较多空闲页，建议执行 VACUUM");
        println!("注意：VACUUM 会锁定数据库，请在应用停止时执行");
    }

    Ok(())
}
