use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::thread;

const DEFAULT_OBSERVABILITY_MAINTENANCE_INTERVAL_SECS: i64 = 900;
const OBSERVABILITY_MAINTENANCE_INTERVAL_SECS_ENV: &str =
    "CODEXMANAGER_OBSERVABILITY_MAINTENANCE_INTERVAL_SECS";

static OBSERVABILITY_MAINTENANCE_RUNNING: AtomicBool = AtomicBool::new(false);
static LAST_OBSERVABILITY_MAINTENANCE_SCHEDULED_AT: AtomicI64 = AtomicI64::new(0);

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
        Some(storage) => match storage.prune_observability_history(now) {
            Ok(()) => {
                log::debug!("event=gateway_observability_maintenance_completed");
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
        },
        None => {
            log::warn!("event=gateway_observability_maintenance_storage_unavailable");
            false
        }
    };

    finish_observability_maintenance_slot(previous_last, succeeded);
}

#[cfg(test)]
mod tests {
    use super::{
        finish_observability_maintenance_slot_with_state,
        try_reserve_observability_maintenance_slot_with_state,
    };
    use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

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
}
