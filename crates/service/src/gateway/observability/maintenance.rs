use chrono::{Duration as ChronoDuration, Local, LocalResult, NaiveDate, TimeZone, Timelike};
use codexmanager_core::storage::{
    observability_maintenance_batch_limit, DatabasePageStats, Storage,
};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Response};

const DEFAULT_OBSERVABILITY_MAINTENANCE_INTERVAL_SECS: i64 = 900;
const DEFAULT_DB_AUTO_VACUUM_IDLE_SECS: i64 = 3600;
const DEFAULT_DB_AUTO_VACUUM_MIN_INTERVAL_SECS: i64 = 86_400;
const DEFAULT_DB_AUTO_VACUUM_MIN_FREE_MB: i64 = 128;
const DEFAULT_DB_AUTO_VACUUM_MIN_FREE_PERCENT: i64 = 25;
const DB_AUTO_VACUUM_ENABLED_ENV: &str = "CODEXMANAGER_DB_AUTO_VACUUM_ENABLED";
const DB_AUTO_VACUUM_IDLE_SECS_ENV: &str = "CODEXMANAGER_DB_AUTO_VACUUM_IDLE_SECS";
const DB_AUTO_VACUUM_MIN_INTERVAL_SECS_ENV: &str = "CODEXMANAGER_DB_AUTO_VACUUM_MIN_INTERVAL_SECS";
const DB_AUTO_VACUUM_MIN_FREE_MB_ENV: &str = "CODEXMANAGER_DB_AUTO_VACUUM_MIN_FREE_MB";
const DB_AUTO_VACUUM_MIN_FREE_PERCENT_ENV: &str = "CODEXMANAGER_DB_AUTO_VACUUM_MIN_FREE_PERCENT";
const DB_AUTO_VACUUM_WINDOW_START_HOUR_ENV: &str = "CODEXMANAGER_DB_AUTO_VACUUM_WINDOW_START_HOUR";
const DB_AUTO_VACUUM_WINDOW_END_HOUR_ENV: &str = "CODEXMANAGER_DB_AUTO_VACUUM_WINDOW_END_HOUR";
const OBSERVABILITY_MAINTENANCE_INTERVAL_SECS_ENV: &str =
    "CODEXMANAGER_OBSERVABILITY_MAINTENANCE_INTERVAL_SECS";

static OBSERVABILITY_MAINTENANCE_RUNNING: AtomicBool = AtomicBool::new(false);
static LAST_OBSERVABILITY_MAINTENANCE_SCHEDULED_AT: AtomicI64 = AtomicI64::new(0);
static DB_COMPACTION_RUNNING: AtomicBool = AtomicBool::new(false);
static LAST_DB_COMPACTION_STARTED_AT: AtomicI64 = AtomicI64::new(0);
static IDLE_DB_MAINTENANCE_SCHEDULER_STARTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy)]
struct DbAutoVacuumConfig {
    enabled: bool,
    idle_secs: i64,
    min_interval_secs: i64,
    min_free_bytes: i64,
    min_free_percent: i64,
    window_start_hour: Option<u32>,
    window_end_hour: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DbAutoVacuumDecision {
    Run,
    Disabled,
    ActiveGatewayRequests,
    IdleTooShort,
    TooSoon,
    OutsideWindow,
    FreeSpaceBelowThreshold,
}

fn env_bool_or(name: &str, default: bool) -> bool {
    std::env::var(name)
        .ok()
        .map(|raw| match raw.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => default,
        })
        .unwrap_or(default)
}

fn env_i64_or(name: &str, default: i64) -> i64 {
    std::env::var(name)
        .ok()
        .and_then(|raw| raw.trim().parse::<i64>().ok())
        .unwrap_or(default)
}

fn env_hour(name: &str) -> Option<u32> {
    std::env::var(name)
        .ok()
        .and_then(|raw| raw.trim().parse::<u32>().ok())
        .filter(|hour| *hour < 24)
}

fn now_unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().min(i64::MAX as u64) as i64)
        .unwrap_or(0)
}

fn db_auto_vacuum_config() -> DbAutoVacuumConfig {
    DbAutoVacuumConfig {
        enabled: env_bool_or(DB_AUTO_VACUUM_ENABLED_ENV, false),
        idle_secs: env_i64_or(
            DB_AUTO_VACUUM_IDLE_SECS_ENV,
            DEFAULT_DB_AUTO_VACUUM_IDLE_SECS,
        )
        .max(300),
        min_interval_secs: env_i64_or(
            DB_AUTO_VACUUM_MIN_INTERVAL_SECS_ENV,
            DEFAULT_DB_AUTO_VACUUM_MIN_INTERVAL_SECS,
        )
        .max(3600),
        min_free_bytes: env_i64_or(
            DB_AUTO_VACUUM_MIN_FREE_MB_ENV,
            DEFAULT_DB_AUTO_VACUUM_MIN_FREE_MB,
        )
        .max(0)
        .saturating_mul(1024 * 1024),
        min_free_percent: env_i64_or(
            DB_AUTO_VACUUM_MIN_FREE_PERCENT_ENV,
            DEFAULT_DB_AUTO_VACUUM_MIN_FREE_PERCENT,
        )
        .clamp(0, 100),
        window_start_hour: env_hour(DB_AUTO_VACUUM_WINDOW_START_HOUR_ENV),
        window_end_hour: env_hour(DB_AUTO_VACUUM_WINDOW_END_HOUR_ENV),
    }
}

/// 函数 `observability_maintenance_interval_secs`
///
/// # 返回
/// 返回观测数据维护调度间隔，最低 60 秒。
fn observability_maintenance_interval_secs() -> i64 {
    std::env::var(OBSERVABILITY_MAINTENANCE_INTERVAL_SECS_ENV)
        .ok()
        .and_then(|raw| raw.trim().parse::<i64>().ok())
        .unwrap_or(DEFAULT_OBSERVABILITY_MAINTENANCE_INTERVAL_SECS)
        .max(60)
}

/// 函数 `try_reserve_observability_maintenance_slot`
///
/// # 参数
/// - now: 当前时间戳
/// - interval_secs: 调度间隔
///
/// # 返回
/// 返回被替换前的上次调度时间；返回 `None` 表示本次无需调度。
fn try_reserve_observability_maintenance_slot(now: i64, interval_secs: i64) -> Option<i64> {
    try_reserve_observability_maintenance_slot_with_state(
        now,
        interval_secs,
        &OBSERVABILITY_MAINTENANCE_RUNNING,
        &LAST_OBSERVABILITY_MAINTENANCE_SCHEDULED_AT,
    )
}

/// 函数 `try_reserve_observability_maintenance_slot_with_state`
///
/// # 参数
/// - now: 当前时间戳
/// - interval_secs: 调度间隔
/// - running: 后台任务运行标记
/// - last_scheduled_at: 上次调度时间
///
/// # 返回
/// 返回被替换前的上次调度时间；返回 `None` 表示本次无需调度。
fn try_reserve_observability_maintenance_slot_with_state(
    now: i64,
    interval_secs: i64,
    running: &AtomicBool,
    last_scheduled_at: &AtomicI64,
) -> Option<i64> {
    if running.load(Ordering::Relaxed) {
        return None;
    }

    let interval = interval_secs.max(60);
    let last = last_scheduled_at.load(Ordering::Relaxed);
    if last != 0 && now.saturating_sub(last) < interval {
        return None;
    }
    if last_scheduled_at
        .compare_exchange(last, now, Ordering::SeqCst, Ordering::Relaxed)
        .is_err()
    {
        return None;
    }
    if running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
        .is_err()
    {
        last_scheduled_at.store(last, Ordering::Relaxed);
        return None;
    }
    Some(last)
}

/// 函数 `finish_observability_maintenance_slot`
///
/// # 参数
/// - previous_last: 调度前的上次维护时间
/// - succeeded: 后台维护是否成功
///
/// # 返回
/// 无
fn finish_observability_maintenance_slot(previous_last: i64, succeeded: bool) {
    finish_observability_maintenance_slot_with_state(
        previous_last,
        succeeded,
        &OBSERVABILITY_MAINTENANCE_RUNNING,
        &LAST_OBSERVABILITY_MAINTENANCE_SCHEDULED_AT,
    );
}

/// 函数 `finish_observability_maintenance_slot_with_state`
///
/// # 参数
/// - previous_last: 调度前的上次维护时间
/// - succeeded: 后台维护是否成功
/// - running: 后台任务运行标记
/// - last_scheduled_at: 上次调度时间
///
/// # 返回
/// 无
fn finish_observability_maintenance_slot_with_state(
    previous_last: i64,
    succeeded: bool,
    running: &AtomicBool,
    last_scheduled_at: &AtomicI64,
) {
    if !succeeded {
        last_scheduled_at.store(previous_last, Ordering::Relaxed);
    }
    running.store(false, Ordering::Release);
}

pub(crate) fn db_compaction_in_progress() -> bool {
    DB_COMPACTION_RUNNING.load(Ordering::Relaxed)
}

pub(crate) fn db_compaction_busy_response() -> Response<std::io::Cursor<Vec<u8>>> {
    let mut response = super::error_response::terminal_text_response(
        429,
        "database maintenance in progress",
        None,
    );
    if let Ok(header) = Header::from_bytes(b"Retry-After".as_slice(), b"60".as_slice()) {
        response.add_header(header);
    }
    response
}

fn local_hour_now() -> u32 {
    Local::now().hour()
}

fn hour_inside_window(hour: u32, start: Option<u32>, end: Option<u32>) -> bool {
    let (Some(start), Some(end)) = (start, end) else {
        return true;
    };
    if start == end {
        return true;
    }
    if start < end {
        hour >= start && hour < end
    } else {
        hour >= start || hour < end
    }
}

fn should_run_db_auto_vacuum(
    config: DbAutoVacuumConfig,
    now: i64,
    active_gateway_requests: usize,
    gateway_idle_secs: i64,
    last_started_at: i64,
    stats: DatabasePageStats,
    local_hour: u32,
) -> DbAutoVacuumDecision {
    if !config.enabled {
        return DbAutoVacuumDecision::Disabled;
    }
    if active_gateway_requests > 0 {
        return DbAutoVacuumDecision::ActiveGatewayRequests;
    }
    if gateway_idle_secs < config.idle_secs {
        return DbAutoVacuumDecision::IdleTooShort;
    }
    if last_started_at > 0 && now.saturating_sub(last_started_at) < config.min_interval_secs {
        return DbAutoVacuumDecision::TooSoon;
    }
    if !hour_inside_window(local_hour, config.window_start_hour, config.window_end_hour) {
        return DbAutoVacuumDecision::OutsideWindow;
    }
    if stats.free_bytes() < config.min_free_bytes || stats.free_percent() < config.min_free_percent
    {
        return DbAutoVacuumDecision::FreeSpaceBelowThreshold;
    }
    DbAutoVacuumDecision::Run
}

/// 函数 `local_today_start_ts`
///
/// # 返回
/// 返回当前本地日期的起始 Unix 时间戳，供日级 rollup 与 dashboard 日边界保持一致。
fn local_midnight_ts(date: NaiveDate, prefer_latest: bool) -> Result<i64, String> {
    let naive = date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| "build local midnight failed".to_string())?;
    match Local.from_local_datetime(&naive) {
        LocalResult::Single(value) => Ok(value.timestamp()),
        LocalResult::Ambiguous(a, b) if prefer_latest => Ok(a.timestamp().max(b.timestamp())),
        LocalResult::Ambiguous(a, b) => Ok(a.timestamp().min(b.timestamp())),
        LocalResult::None => Err(format!("resolve local midnight failed: {date}")),
    }
}

fn local_today_start_ts() -> Result<i64, String> {
    let now = Local::now();
    local_midnight_ts(now.date_naive(), false)
}

fn local_closed_day_ranges(
    storage: &Storage,
    today_start_ts: i64,
) -> Result<Vec<(i64, i64)>, String> {
    let pending_timestamps = storage
        .pending_daily_rollup_timestamps_before_limited(
            today_start_ts,
            observability_maintenance_batch_limit(),
        )
        .map_err(|err| format!("load pending daily rollup timestamps failed: {err}"))?;
    local_closed_day_ranges_from_timestamps(&pending_timestamps)
}

fn local_closed_day_ranges_from_timestamps(
    pending_timestamps: &[i64],
) -> Result<Vec<(i64, i64)>, String> {
    let mut dates = BTreeSet::new();
    for &timestamp in pending_timestamps {
        let local = Local
            .timestamp_opt(timestamp, 0)
            .single()
            .ok_or_else(|| format!("resolve pending local day failed: {timestamp}"))?;
        dates.insert(local.date_naive());
    }
    let mut ranges = Vec::new();
    for date in dates {
        let day_start = local_midnight_ts(date, false)?;
        let day_end = local_midnight_ts(date + ChronoDuration::days(1), true)?;
        if day_end <= day_start {
            return Err(format!("invalid local day range: {day_start}..{day_end}"));
        }
        ranges.push((day_start, day_end));
    }
    Ok(ranges)
}
/// 函数 `schedule_observability_maintenance`
///
/// # 参数
/// - now: 当前请求日志时间戳
///
/// # 返回
/// 无
pub(crate) fn schedule_observability_maintenance(now: i64) {
    let interval = observability_maintenance_interval_secs();
    let Some(previous_last) = try_reserve_observability_maintenance_slot(now, interval) else {
        return;
    };

    let spawn_result = thread::Builder::new()
        .name("codexmanager-observability-maintenance".to_string())
        .spawn(move || run_observability_maintenance(now, previous_last));
    if let Err(err) = spawn_result {
        finish_observability_maintenance_slot(previous_last, false);
        log::warn!(
            "event=gateway_observability_maintenance_spawn_failed err={}",
            err
        );
    }
}

pub(crate) fn ensure_idle_db_maintenance_scheduler() {
    if IDLE_DB_MAINTENANCE_SCHEDULER_STARTED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
        .is_err()
    {
        return;
    }
    let _ = super::metrics::gateway_idle_secs(now_unix_secs());
    let spawn_result = thread::Builder::new()
        .name("codexmanager-idle-db-maintenance".to_string())
        .spawn(run_idle_db_maintenance_loop);
    if let Err(err) = spawn_result {
        IDLE_DB_MAINTENANCE_SCHEDULER_STARTED.store(false, Ordering::Release);
        log::warn!(
            "event=idle_db_maintenance_scheduler_spawn_failed err={}",
            err
        );
    }
}

fn idle_db_maintenance_sleep_secs(config: DbAutoVacuumConfig) -> u64 {
    if !config.enabled {
        return DEFAULT_OBSERVABILITY_MAINTENANCE_INTERVAL_SECS as u64;
    }
    (config.idle_secs / 4).clamp(60, 900) as u64
}

fn run_idle_db_maintenance_loop() {
    while !crate::shutdown_requested() {
        let config = db_auto_vacuum_config();
        thread::sleep(Duration::from_secs(idle_db_maintenance_sleep_secs(config)));
        if crate::shutdown_requested() {
            break;
        }
        if !db_auto_vacuum_config().enabled {
            continue;
        }
        let now = now_unix_secs();
        let Some(storage) = crate::storage_helpers::open_storage() else {
            continue;
        };
        maybe_run_idle_db_auto_vacuum(&storage, now);
    }
}

/// 函数 `run_observability_maintenance`
///
/// # 参数
/// - now: 当前请求日志时间戳
/// - previous_last: 调度前的上次维护时间
///
/// # 返回
/// 无
fn run_observability_maintenance(now: i64, previous_last: i64) {
    let succeeded = match crate::storage_helpers::open_storage() {
        Some(storage) => {
            let maintenance_result = local_today_start_ts().and_then(|today_start| {
                let day_ranges = local_closed_day_ranges(&storage, today_start)?;
                storage
                    .prune_observability_history_with_daily_rollup_ranges(now, &day_ranges)
                    .map_err(|err| err.to_string())
            });
            match maintenance_result {
                Ok(()) => {
                    log::debug!("event=gateway_observability_maintenance_completed");
                    maybe_run_idle_db_auto_vacuum(&storage, now);
                    true
                }
                Err(err) => {
                    let err_text = err.to_string();
                    super::metrics::record_db_error(err_text.as_str());
                    log::warn!(
                        "event=gateway_observability_maintenance_failed err={}",
                        err_text
                    );
                    false
                }
            }
        }
        None => {
            log::warn!("event=gateway_observability_maintenance_storage_unavailable");
            false
        }
    };

    finish_observability_maintenance_slot(previous_last, succeeded);
}

fn maybe_run_idle_db_auto_vacuum(storage: &Storage, now: i64) {
    let config = db_auto_vacuum_config();
    if !config.enabled {
        return;
    }
    let Ok(stats) = storage.database_page_stats() else {
        return;
    };
    let active = super::metrics::active_gateway_requests();
    let idle_secs = super::metrics::gateway_idle_secs(now);
    let last_started_at = LAST_DB_COMPACTION_STARTED_AT.load(Ordering::Relaxed);
    let decision = should_run_db_auto_vacuum(
        config,
        now,
        active,
        idle_secs,
        last_started_at,
        stats,
        local_hour_now(),
    );
    if decision != DbAutoVacuumDecision::Run {
        log::debug!(
            "event=db_auto_vacuum_skipped reason={:?} active={} idle_secs={} free_mb={} free_percent={}",
            decision,
            active,
            idle_secs,
            stats.free_bytes() / 1024 / 1024,
            stats.free_percent()
        );
        return;
    }
    if DB_COMPACTION_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
        .is_err()
    {
        return;
    }
    thread::sleep(Duration::from_millis(500));
    if super::metrics::active_gateway_requests() > 0
        || super::metrics::gateway_idle_secs(now) < config.idle_secs
    {
        DB_COMPACTION_RUNNING.store(false, Ordering::Release);
        return;
    }
    LAST_DB_COMPACTION_STARTED_AT.store(now, Ordering::Relaxed);
    let result = run_idle_db_auto_vacuum(storage, stats);
    DB_COMPACTION_RUNNING.store(false, Ordering::Release);

    if let Err(err) = result {
        super::metrics::record_db_error(err.as_str());
        log::warn!("event=db_auto_vacuum_failed err={}", err);
    }
}

fn run_idle_db_auto_vacuum(storage: &Storage, before: DatabasePageStats) -> Result<(), String> {
    log::warn!(
        "event=db_auto_vacuum_started total_mb={} free_mb={} free_percent={}",
        before.total_bytes() / 1024 / 1024,
        before.free_bytes() / 1024 / 1024,
        before.free_percent()
    );
    storage
        .checkpoint_wal_truncate()
        .map_err(|err| format!("wal checkpoint before vacuum failed: {err}"))?;
    storage
        .vacuum_database()
        .map_err(|err| format!("vacuum failed: {err}"))?;
    let _ = storage.checkpoint_wal_truncate();
    if let Ok(after) = storage.database_page_stats() {
        log::warn!(
            "event=db_auto_vacuum_completed total_mb_before={} total_mb_after={} free_mb_after={} reclaimed_mb={}",
            before.total_bytes() / 1024 / 1024,
            after.total_bytes() / 1024 / 1024,
            after.free_bytes() / 1024 / 1024,
            before
                .total_bytes()
                .saturating_sub(after.total_bytes())
                / 1024
                / 1024
        );
    } else {
        log::warn!("event=db_auto_vacuum_completed");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        db_compaction_busy_response, finish_observability_maintenance_slot_with_state,
        hour_inside_window, local_closed_day_ranges_from_timestamps, should_run_db_auto_vacuum,
        try_reserve_observability_maintenance_slot_with_state, DbAutoVacuumConfig,
        DbAutoVacuumDecision,
    };
    use codexmanager_core::storage::DatabasePageStats;
    use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

    #[test]
    fn sparse_pending_history_only_builds_ranges_for_dates_with_rows() {
        // 两批明细相隔 25 年，中间没有数据时只应生成两个本地日区间。
        let ranges =
            local_closed_day_ranges_from_timestamps(&[946_728_000, 946_731_600, 1_735_732_800])
                .expect("build sparse local day ranges");

        assert_eq!(ranges.len(), 2);
        assert!(ranges.iter().all(|(start, end)| end > start));
    }

    #[test]
    fn reserves_first_maintenance_slot() {
        let running = AtomicBool::new(false);
        let last = AtomicI64::new(0);

        let previous =
            try_reserve_observability_maintenance_slot_with_state(1_000, 900, &running, &last);

        assert_eq!(previous, Some(0));
        assert!(running.load(Ordering::SeqCst));
        assert_eq!(last.load(Ordering::SeqCst), 1_000);
    }

    #[test]
    fn skips_inside_maintenance_interval() {
        let running = AtomicBool::new(false);
        let last = AtomicI64::new(1_000);

        let previous =
            try_reserve_observability_maintenance_slot_with_state(1_500, 900, &running, &last);

        assert_eq!(previous, None);
        assert!(!running.load(Ordering::SeqCst));
        assert_eq!(last.load(Ordering::SeqCst), 1_000);
    }

    #[test]
    fn skips_while_background_maintenance_is_running() {
        let running = AtomicBool::new(true);
        let last = AtomicI64::new(1_000);

        let previous =
            try_reserve_observability_maintenance_slot_with_state(2_000, 900, &running, &last);

        assert_eq!(previous, None);
        assert!(running.load(Ordering::SeqCst));
        assert_eq!(last.load(Ordering::SeqCst), 1_000);
    }

    #[test]
    fn failure_restores_previous_maintenance_time() {
        let running = AtomicBool::new(true);
        let last = AtomicI64::new(2_000);

        finish_observability_maintenance_slot_with_state(1_000, false, &running, &last);

        assert!(!running.load(Ordering::SeqCst));
        assert_eq!(last.load(Ordering::SeqCst), 1_000);
    }

    #[test]
    fn success_keeps_new_maintenance_time() {
        let running = AtomicBool::new(true);
        let last = AtomicI64::new(2_000);

        finish_observability_maintenance_slot_with_state(1_000, true, &running, &last);

        assert!(!running.load(Ordering::SeqCst));
        assert_eq!(last.load(Ordering::SeqCst), 2_000);
    }

    fn vacuum_config(enabled: bool) -> DbAutoVacuumConfig {
        DbAutoVacuumConfig {
            enabled,
            idle_secs: 3600,
            min_interval_secs: 86_400,
            min_free_bytes: 128 * 1024 * 1024,
            min_free_percent: 25,
            window_start_hour: None,
            window_end_hour: None,
        }
    }

    fn fragmented_stats() -> DatabasePageStats {
        DatabasePageStats {
            page_count: 1000,
            page_size: 4096,
            freelist_count: 400,
        }
    }

    #[test]
    fn auto_vacuum_decision_requires_enabled_idle_and_free_space() {
        assert_eq!(
            should_run_db_auto_vacuum(
                vacuum_config(false),
                10_000,
                0,
                7200,
                0,
                fragmented_stats(),
                12
            ),
            DbAutoVacuumDecision::Disabled
        );
        assert_eq!(
            should_run_db_auto_vacuum(
                vacuum_config(true),
                10_000,
                1,
                7200,
                0,
                fragmented_stats(),
                12
            ),
            DbAutoVacuumDecision::ActiveGatewayRequests
        );
        assert_eq!(
            should_run_db_auto_vacuum(
                vacuum_config(true),
                10_000,
                0,
                1200,
                0,
                fragmented_stats(),
                12
            ),
            DbAutoVacuumDecision::IdleTooShort
        );
        assert_eq!(
            should_run_db_auto_vacuum(
                vacuum_config(true),
                10_000,
                0,
                7200,
                0,
                fragmented_stats(),
                12
            ),
            DbAutoVacuumDecision::FreeSpaceBelowThreshold
        );

        let mut config = vacuum_config(true);
        config.min_free_bytes = 1 * 1024 * 1024;
        assert_eq!(
            should_run_db_auto_vacuum(config, 10_000, 0, 7200, 0, fragmented_stats(), 12),
            DbAutoVacuumDecision::Run
        );
    }

    #[test]
    fn auto_vacuum_decision_respects_interval_and_hour_window() {
        let mut config = vacuum_config(true);
        config.min_free_bytes = 1 * 1024 * 1024;
        config.window_start_hour = Some(2);
        config.window_end_hour = Some(5);

        assert_eq!(
            should_run_db_auto_vacuum(config, 10_000, 0, 7200, 9_000, fragmented_stats(), 3),
            DbAutoVacuumDecision::TooSoon
        );
        assert_eq!(
            should_run_db_auto_vacuum(config, 100_000, 0, 7200, 0, fragmented_stats(), 12),
            DbAutoVacuumDecision::OutsideWindow
        );
        assert_eq!(
            should_run_db_auto_vacuum(config, 100_000, 0, 7200, 0, fragmented_stats(), 3),
            DbAutoVacuumDecision::Run
        );
    }

    #[test]
    fn hour_window_supports_cross_midnight_ranges() {
        assert!(hour_inside_window(23, Some(22), Some(3)));
        assert!(hour_inside_window(2, Some(22), Some(3)));
        assert!(!hour_inside_window(12, Some(22), Some(3)));
        assert!(hour_inside_window(12, None, Some(3)));
    }

    #[test]
    fn db_compaction_busy_response_uses_429_and_retry_after() {
        let response = db_compaction_busy_response();
        assert_eq!(response.status_code().0, 429);
        let retry_after = response
            .headers()
            .iter()
            .find(|header| {
                header
                    .field
                    .as_str()
                    .as_str()
                    .eq_ignore_ascii_case("Retry-After")
            })
            .map(|header| header.value.as_str().to_string());
        assert_eq!(retry_after.as_deref(), Some("60"));
    }
}
