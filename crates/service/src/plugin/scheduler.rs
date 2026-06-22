use crate::storage_helpers::open_storage;

use super::runtime::run_plugin_task;
use super::store::rearm_enabled_interval_tasks_for_plugin;

const DEFAULT_PLUGIN_SCHEDULER_INTERVAL_SECS: u64 = 5;

/// 函数 `run_due_tasks_once`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - crate: 参数 crate
///
/// # 返回
/// 返回函数执行结果
pub(crate) fn run_due_tasks_once() -> u64 {
    let Some(storage) = open_storage() else {
        return DEFAULT_PLUGIN_SCHEDULER_INTERVAL_SECS;
    };
    let now = codexmanager_core::storage::now_ts();
    if rearm_enabled_interval_tasks_for_plugin(&storage, None, now).is_err() {
        log::warn!("repair plugin task schedules failed");
    }
    let tasks = match storage.list_due_plugin_tasks(now, 100) {
        Ok(items) => items,
        Err(err) => {
            log::warn!("list due plugin tasks failed: {err}");
            return DEFAULT_PLUGIN_SCHEDULER_INTERVAL_SECS;
        }
    };
    for task in tasks {
        let _ = run_plugin_task(&task.id, None);
    }

    // 直接查询下一个到期任务的时间戳
    let next_due_at = match storage.next_plugin_task_due_at(now) {
        Ok(Some(timestamp)) => timestamp,
        Ok(None) => {
            // 没有待执行任务，使用默认间隔
            return DEFAULT_PLUGIN_SCHEDULER_INTERVAL_SECS;
        }
        Err(err) => {
            log::warn!("query next plugin task due time failed: {err}");
            return DEFAULT_PLUGIN_SCHEDULER_INTERVAL_SECS;
        }
    };

    // 如果下一个任务已到期，立即返回 1 秒
    if next_due_at <= now {
        return 1;
    }

    // 计算睡眠时长
    let next_sleep_secs = (next_due_at - now) as u64;
    next_sleep_secs.clamp(1, DEFAULT_PLUGIN_SCHEDULER_INTERVAL_SECS)
}
