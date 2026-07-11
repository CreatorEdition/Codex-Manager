use rusqlite::{Connection, OpenFlags};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const VACUUM_RECOMMENDATION_THRESHOLD: i64 = 1000;
const HELP: &str = r#"数据库检查与空间回收工具

用法：
  db-optimize <数据库路径> [--vacuum]
  db-optimize <数据库路径> [--check-only]
  db-optimize --help

说明：
  默认以只读方式检查数据库，不执行 WAL checkpoint 或 VACUUM。
  只有显式传入 --vacuum 才会以读写方式打开数据库并执行锁库操作。
  --check-only 为兼容旧命令保留，与默认行为相同。
"#;

#[derive(Debug, PartialEq, Eq)]
enum OperationMode {
    Check,
    Vacuum,
}

#[derive(Debug, PartialEq, Eq)]
struct CliOptions {
    db_path: PathBuf,
    mode: OperationMode,
}

#[derive(Debug, PartialEq, Eq)]
enum CliAction {
    Run(CliOptions),
    Help,
}

#[derive(Debug, Clone, Copy)]
struct DatabaseStats {
    page_count: i64,
    page_size: i64,
    freelist_count: i64,
}

/// 解析命令行参数，并拒绝缺失路径、未知参数和相互冲突的模式。
fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliAction, String> {
    let args: Vec<String> = args.into_iter().collect();
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        return Ok(CliAction::Help);
    }
    if args.is_empty() {
        return Err("缺少数据库路径".to_string());
    }

    let mut db_path = None;
    let mut vacuum = false;
    let mut check_only = false;

    for arg in args {
        match arg.as_str() {
            "--vacuum" => {
                if vacuum {
                    return Err("参数 --vacuum 不能重复".to_string());
                }
                vacuum = true;
            }
            "--check-only" => {
                if check_only {
                    return Err("参数 --check-only 不能重复".to_string());
                }
                check_only = true;
            }
            "--help" | "-h" => return Err("帮助参数不能与其他参数同时使用".to_string()),
            value if value.starts_with('-') => return Err(format!("未知参数: {value}")),
            value => {
                if db_path.is_some() {
                    return Err(format!("只能提供一个数据库路径，多余参数: {value}"));
                }
                db_path = Some(PathBuf::from(value));
            }
        }
    }

    if vacuum && check_only {
        return Err("--vacuum 与 --check-only 不能同时使用".to_string());
    }

    let db_path = db_path.ok_or_else(|| "缺少数据库路径".to_string())?;
    let mode = if vacuum {
        OperationMode::Vacuum
    } else {
        OperationMode::Check
    };

    Ok(CliAction::Run(CliOptions { db_path, mode }))
}

/// 校验数据库路径，避免 SQLite 因输入错误而创建新的空数据库。
fn validate_db_path(db_path: &Path) -> Result<(), String> {
    let metadata = fs::metadata(db_path)
        .map_err(|error| format!("数据库路径不可访问 {}: {error}", db_path.display()))?;
    if !metadata.is_file() {
        return Err(format!("数据库路径不是文件: {}", db_path.display()));
    }
    Ok(())
}

/// 读取 SQLite 页面统计，任何读取错误都会使命令以非零状态退出。
fn collect_stats(conn: &Connection) -> Result<DatabaseStats, String> {
    let page_count = conn
        .query_row("PRAGMA page_count;", [], |row| row.get(0))
        .map_err(|error| format!("读取 page_count 失败: {error}"))?;
    let page_size = conn
        .query_row("PRAGMA page_size;", [], |row| row.get(0))
        .map_err(|error| format!("读取 page_size 失败: {error}"))?;
    let freelist_count = conn
        .query_row("PRAGMA freelist_count;", [], |row| row.get(0))
        .map_err(|error| format!("读取 freelist_count 失败: {error}"))?;

    Ok(DatabaseStats {
        page_count,
        page_size,
        freelist_count,
    })
}

/// 将 SQLite 页数换算为 MiB，避免整数乘法溢出。
fn pages_to_mib(pages: i64, page_size: i64) -> f64 {
    pages as f64 * page_size as f64 / 1024.0 / 1024.0
}

/// 输出数据库页面统计。
fn print_stats(title: &str, stats: DatabaseStats) {
    println!("\n{title}:");
    println!("  总页数: {}", stats.page_count);
    println!("  页大小: {} bytes", stats.page_size);
    println!("  空闲页: {}", stats.freelist_count);
    println!(
        "  总大小: {:.2} MiB",
        pages_to_mib(stats.page_count, stats.page_size)
    );
    println!(
        "  空闲空间: {:.2} MiB",
        pages_to_mib(stats.freelist_count, stats.page_size)
    );
}

/// 执行只读检查，不运行可能改变数据库或需要排他锁的操作。
fn run_check(db_path: &Path) -> Result<(), String> {
    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| format!("无法以只读方式打开数据库 {}: {error}", db_path.display()))?;
    let stats = collect_stats(&conn)?;
    print_stats("数据库统计", stats);

    if stats.freelist_count > VACUUM_RECOMMENDATION_THRESHOLD {
        println!(
            "\n检测到较多空闲页（{}），可在停止应用后显式执行 --vacuum",
            stats.freelist_count
        );
    } else {
        println!("\n空闲页数量正常（{}），无需 VACUUM", stats.freelist_count);
    }

    Ok(())
}

/// 在用户显式授权后执行 checkpoint 与 VACUUM，并输出前后统计。
fn run_vacuum(db_path: &Path) -> Result<(), String> {
    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .map_err(|error| format!("无法以读写方式打开数据库 {}: {error}", db_path.display()))?;

    println!("警告：即将执行 WAL checkpoint 与 VACUUM，请确认应用已停止");
    let (busy, log, checkpointed) = conn
        .query_row("PRAGMA wal_checkpoint(TRUNCATE);", [], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, i32>(2)?,
            ))
        })
        .map_err(|error| format!("WAL checkpoint 失败: {error}"))?;
    if busy != 0 {
        return Err(format!(
            "WAL checkpoint 未完成，数据库可能仍在使用中（busy={busy}, log={log}, checkpointed={checkpointed}）"
        ));
    }
    println!("WAL checkpoint 完成（log={log}, checkpointed={checkpointed}）");

    let before = collect_stats(&conn)?;
    print_stats("VACUUM 前统计", before);
    conn.execute_batch("VACUUM;")
        .map_err(|error| format!("VACUUM 执行失败，请确认应用已停止: {error}"))?;
    let after = collect_stats(&conn)?;
    print_stats("VACUUM 后统计", after);
    println!(
        "  回收空间: {:.2} MiB",
        pages_to_mib(
            before.page_count.saturating_sub(after.page_count),
            before.page_size
        )
    );

    Ok(())
}

/// 根据解析后的模式执行只读检查或显式空间回收。
fn run(options: CliOptions) -> Result<(), String> {
    validate_db_path(&options.db_path)?;
    println!("正在连接数据库: {}", options.db_path.display());

    match options.mode {
        OperationMode::Check => run_check(&options.db_path),
        OperationMode::Vacuum => run_vacuum(&options.db_path),
    }
}

/// 命令行入口，参数或执行错误统一返回非零退出码。
fn main() -> ExitCode {
    match parse_args(env::args().skip(1)) {
        Ok(CliAction::Help) => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Ok(CliAction::Run(options)) => match run(options) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("错误：{error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("错误：{error}\n\n{HELP}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// 将测试参数转换为命令行字符串列表。
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    /// 创建独立的临时 SQLite 数据库，供读写行为测试使用。
    fn create_temp_db() -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("系统时间应晚于 Unix 纪元")
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "db-optimize-test-{}-{timestamp}.db",
            std::process::id()
        ));
        let conn = Connection::open(&path).expect("应创建临时 SQLite 数据库");
        conn.execute_batch(
            "CREATE TABLE sample (id INTEGER PRIMARY KEY, value TEXT);\
             INSERT INTO sample(value) VALUES ('测试数据');",
        )
        .expect("应写入临时测试数据");
        drop(conn);
        path
    }

    /// 删除临时数据库及 SQLite 可能生成的旁路文件。
    fn remove_temp_db(path: &Path) {
        for candidate in [
            path.to_path_buf(),
            PathBuf::from(format!("{}-wal", path.display())),
            PathBuf::from(format!("{}-shm", path.display())),
        ] {
            let _ = fs::remove_file(candidate);
        }
    }

    #[test]
    fn default_mode_is_read_only_check() {
        let action = parse_args(args(&["test.db"])).expect("应成功解析数据库路径");
        assert_eq!(
            action,
            CliAction::Run(CliOptions {
                db_path: PathBuf::from("test.db"),
                mode: OperationMode::Check,
            })
        );
    }

    #[test]
    fn vacuum_requires_explicit_flag() {
        let action = parse_args(args(&["--vacuum", "test.db"])).expect("应成功解析 VACUUM 模式");
        assert_eq!(
            action,
            CliAction::Run(CliOptions {
                db_path: PathBuf::from("test.db"),
                mode: OperationMode::Vacuum,
            })
        );
    }

    #[test]
    fn legacy_check_only_flag_remains_read_only() {
        let action = parse_args(args(&["test.db", "--check-only"])).expect("应兼容旧检查参数");
        assert_eq!(
            action,
            CliAction::Run(CliOptions {
                db_path: PathBuf::from("test.db"),
                mode: OperationMode::Check,
            })
        );
    }

    #[test]
    fn missing_path_is_rejected() {
        let error = parse_args(Vec::<String>::new()).expect_err("缺少路径必须失败");
        assert!(error.contains("缺少数据库路径"));
    }

    #[test]
    fn unknown_flag_is_rejected() {
        let error = parse_args(args(&["test.db", "--force"])).expect_err("未知参数必须失败");
        assert!(error.contains("未知参数"));
    }

    #[test]
    fn conflicting_modes_are_rejected() {
        let error = parse_args(args(&["test.db", "--vacuum", "--check-only"]))
            .expect_err("冲突模式必须失败");
        assert!(error.contains("不能同时使用"));
    }

    #[test]
    fn check_and_explicit_vacuum_handle_existing_database() {
        let db_path = create_temp_db();
        run_check(&db_path).expect("默认只读检查应成功");
        run_vacuum(&db_path).expect("显式 VACUUM 应成功");
        remove_temp_db(&db_path);
    }

    #[test]
    fn invalid_path_does_not_create_database() {
        let path = env::temp_dir().join(format!("db-optimize-missing-{}.db", std::process::id()));
        remove_temp_db(&path);
        validate_db_path(&path).expect_err("不存在的路径必须失败");
        assert!(!path.exists(), "路径校验不得创建空数据库");
    }
}
