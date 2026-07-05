use super::{
    classify_usage_refresh_error, record_usage_refresh_failure,
    should_record_failure_event_with_state, usage_refresh_failure_event_message,
    usage_refresh_failure_event_window_secs, FailureThrottleKey,
};
use crate::usage_scheduler::DEFAULT_USAGE_POLL_INTERVAL_SECS;
use codexmanager_core::storage::{now_ts, Event, Storage};
use std::collections::HashMap;

/// 函数 `usage_refresh_error_class_groups_by_status_code`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn usage_refresh_error_class_groups_by_status_code() {
    assert_eq!(
        classify_usage_refresh_error("usage endpoint status 500 Internal Server Error"),
        "usage_status_500"
    );
    assert_eq!(
        classify_usage_refresh_error("usage endpoint status 503 Service Unavailable"),
        "usage_status_503"
    );
    assert_eq!(
        classify_usage_refresh_error("subscription endpoint status 401 Unauthorized"),
        "usage_status_401"
    );
    assert_eq!(
        classify_usage_refresh_error(
            "subscription endpoint failed: status=503 Service Unavailable body=upstream unavailable"
        ),
        "usage_status_503"
    );
}

/// 函数 `usage_refresh_error_class_catches_timeout_and_connection`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn usage_refresh_error_class_catches_timeout_and_connection() {
    assert_eq!(
        classify_usage_refresh_error("request timeout while calling usage"),
        "timeout"
    );
    assert_eq!(
        classify_usage_refresh_error("connection reset by peer"),
        "connection"
    );
    assert_eq!(classify_usage_refresh_error("unknown error"), "other");
}

/// 函数 `failure_event_throttle_dedupes_within_window`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn failure_event_throttle_dedupes_within_window() {
    let mut state = HashMap::new();
    let key = FailureThrottleKey {
        account_id: "acc-1".to_string(),
        error_class: "usage_status_500".to_string(),
    };

    assert!(should_record_failure_event_with_state(
        &mut state,
        key.clone(),
        100,
        60
    ));
    assert!(!should_record_failure_event_with_state(
        &mut state,
        key.clone(),
        120,
        60
    ));
    assert!(should_record_failure_event_with_state(
        &mut state, key, 161, 60
    ));
}

#[test]
fn default_failure_event_window_covers_usage_poll_interval() {
    let _guard = crate::test_env_guard();
    let previous = std::env::var("CODEXMANAGER_USAGE_REFRESH_FAILURE_EVENT_WINDOW_SECS").ok();
    std::env::remove_var("CODEXMANAGER_USAGE_REFRESH_FAILURE_EVENT_WINDOW_SECS");

    assert!(
        usage_refresh_failure_event_window_secs() >= DEFAULT_USAGE_POLL_INTERVAL_SECS as i64,
        "默认失败事件窗口必须覆盖至少一个用量轮询周期，避免每轮失败都写入 events"
    );

    if let Some(previous) = previous {
        std::env::set_var(
            "CODEXMANAGER_USAGE_REFRESH_FAILURE_EVENT_WINDOW_SECS",
            previous,
        );
    }
}

/// 函数 `failure_event_throttle_isolated_by_error_class`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn failure_event_throttle_isolated_by_error_class() {
    let mut state = HashMap::new();
    let key_500 = FailureThrottleKey {
        account_id: "acc-1".to_string(),
        error_class: "usage_status_500".to_string(),
    };
    let key_timeout = FailureThrottleKey {
        account_id: "acc-1".to_string(),
        error_class: "timeout".to_string(),
    };

    assert!(should_record_failure_event_with_state(
        &mut state, key_500, 100, 60
    ));
    assert!(should_record_failure_event_with_state(
        &mut state,
        key_timeout,
        110,
        60
    ));
}
#[test]
fn usage_refresh_failure_db_dedupe_survives_process_memory_loss() {
    let _guard = crate::test_env_guard();
    let previous = std::env::var("CODEXMANAGER_USAGE_REFRESH_FAILURE_EVENT_WINDOW_SECS").ok();
    std::env::set_var(
        "CODEXMANAGER_USAGE_REFRESH_FAILURE_EVENT_WINDOW_SECS",
        "600",
    );

    let storage = Storage::open_in_memory().expect("open storage");
    storage.init().expect("init storage");
    let account_id = "acc-usage-refresh-db-dedupe";
    let created_at = now_ts();
    storage
        .insert_event(&Event {
            account_id: Some(account_id.to_string()),
            event_type: "usage_refresh_failed".to_string(),
            message: "class=usage_status_503 detail=existing".to_string(),
            created_at,
        })
        .expect("insert existing event");

    record_usage_refresh_failure(
        &storage,
        account_id,
        "usage endpoint status 503 Service Unavailable",
    );

    let same_class_count = storage.event_count().expect("count same class events");
    assert_eq!(same_class_count, 1, "已有窗口内同类失败事件时不应重复写入");

    record_usage_refresh_failure(&storage, account_id, "request timeout while calling usage");
    let all_count = storage.event_count().expect("count all events");
    assert_eq!(all_count, 2, "不同错误类别仍应独立记录");

    if let Some(previous) = previous {
        std::env::set_var(
            "CODEXMANAGER_USAGE_REFRESH_FAILURE_EVENT_WINDOW_SECS",
            previous,
        );
    } else {
        std::env::remove_var("CODEXMANAGER_USAGE_REFRESH_FAILURE_EVENT_WINDOW_SECS");
    }
}

#[test]
fn usage_refresh_failure_event_message_keeps_class_and_sanitizes_detail() {
    let detail = format!("{}\n{}", "x".repeat(520), "tail");
    let message = usage_refresh_failure_event_message("usage_status_500", &detail);

    assert!(message.starts_with("class=usage_status_500 detail="));
    assert!(!message.contains('\n'));
    assert!(!message.contains('\r'));
    assert!(message.ends_with("..."));
}
