use super::{Event, Storage};

/// 函数 `latest_account_status_reasons_returns_latest_reason_per_account`
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
fn latest_account_status_reasons_returns_latest_reason_per_account() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    storage
        .insert_event(&Event {
            account_id: Some("acc-1".to_string()),
            event_type: "account_status_update".to_string(),
            message: "status=unavailable reason=usage_http_401".to_string(),
            created_at: 10,
        })
        .expect("insert first");
    storage
        .insert_event(&Event {
            account_id: Some("acc-1".to_string()),
            event_type: "account_status_update".to_string(),
            message: "status=unavailable reason=account_deactivated".to_string(),
            created_at: 20,
        })
        .expect("insert second");
    storage
        .insert_event(&Event {
            account_id: Some("acc-2".to_string()),
            event_type: "account_status_update".to_string(),
            message: "status=unavailable reason=workspace_deactivated".to_string(),
            created_at: 15,
        })
        .expect("insert third");

    let reasons = storage
        .latest_account_status_reasons(&[
            "acc-1".to_string(),
            "acc-2".to_string(),
            "missing".to_string(),
        ])
        .expect("load reasons");

    assert_eq!(
        reasons.get("acc-1").map(String::as_str),
        Some("account_deactivated")
    );
    assert_eq!(
        reasons.get("acc-2").map(String::as_str),
        Some("workspace_deactivated")
    );
    assert!(!reasons.contains_key("missing"));
}

#[test]
fn prune_events_by_retention_removes_old_events() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    storage
        .insert_event(&Event {
            account_id: Some("acc-old".to_string()),
            event_type: "usage_refresh_failed".to_string(),
            message: "old failure".to_string(),
            created_at: 1,
        })
        .expect("insert old event");
    storage
        .insert_event(&Event {
            account_id: Some("acc-recent".to_string()),
            event_type: "usage_refresh_failed".to_string(),
            message: "recent failure".to_string(),
            created_at: 1_300_000,
        })
        .expect("insert recent event");
    storage
        .insert_event(&Event {
            account_id: Some("acc-status".to_string()),
            event_type: "account_status_update".to_string(),
            message: "status=unavailable reason=workspace_deactivated".to_string(),
            created_at: 1,
        })
        .expect("insert old status event");

    let removed = storage
        .prune_events_by_retention(1_300_000)
        .expect("prune events");

    assert_eq!(removed, 1);
    assert_eq!(storage.event_count().expect("event count"), 2);
    let reasons = storage
        .latest_account_status_reasons(&["acc-status".to_string()])
        .expect("load status reason");
    assert_eq!(
        reasons.get("acc-status").map(String::as_str),
        Some("workspace_deactivated")
    );
}

#[test]
fn prune_events_by_retention_limited_removes_only_one_batch() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    for index in 0..3_i64 {
        storage
            .insert_event(&Event {
                account_id: Some(format!("acc-old-{index}")),
                event_type: "usage_refresh_failed".to_string(),
                message: "old failure".to_string(),
                created_at: 1 + index,
            })
            .expect("insert old event");
    }
    storage
        .insert_event(&Event {
            account_id: Some("acc-status".to_string()),
            event_type: "account_status_update".to_string(),
            message: "status=unavailable reason=workspace_deactivated".to_string(),
            created_at: 1,
        })
        .expect("insert old status event");

    let removed = storage
        .prune_events_by_retention_limited(1_300_000, 2)
        .expect("prune events");

    assert_eq!(removed, 2);
    assert_eq!(storage.event_count().expect("event count"), 2);
    let reasons = storage
        .latest_account_status_reasons(&["acc-status".to_string()])
        .expect("load status reason");
    assert_eq!(
        reasons.get("acc-status").map(String::as_str),
        Some("workspace_deactivated")
    );
}

#[test]
fn prune_events_by_retention_removes_stale_account_status_history() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    storage
        .insert_event(&Event {
            account_id: Some("acc-status".to_string()),
            event_type: "account_status_update".to_string(),
            message: "status=unavailable reason=usage_http_401".to_string(),
            created_at: 1,
        })
        .expect("insert old status event");
    storage
        .insert_event(&Event {
            account_id: Some("acc-status".to_string()),
            event_type: "account_status_update".to_string(),
            message: "status=unavailable reason=refresh_token_invalid:refresh_token_reused"
                .to_string(),
            created_at: 2,
        })
        .expect("insert latest old status event");
    storage
        .insert_event(&Event {
            account_id: Some("acc-other".to_string()),
            event_type: "account_status_update".to_string(),
            message: "status=unavailable reason=workspace_deactivated".to_string(),
            created_at: 1,
        })
        .expect("insert only status event for other account");

    let removed = storage
        .prune_events_by_retention(1_300_000)
        .expect("prune events");

    assert_eq!(removed, 1);
    assert_eq!(storage.event_count().expect("event count"), 2);
    let reasons = storage
        .latest_account_status_reasons(&["acc-status".to_string(), "acc-other".to_string()])
        .expect("load status reasons");
    assert_eq!(
        reasons.get("acc-status").map(String::as_str),
        Some("refresh_token_invalid:refresh_token_reused")
    );
    assert_eq!(
        reasons.get("acc-other").map(String::as_str),
        Some("workspace_deactivated")
    );
}

#[test]
fn prune_events_by_retention_limited_counts_stale_status_history_in_batch() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    for index in 0..3_i64 {
        storage
            .insert_event(&Event {
                account_id: Some("acc-status".to_string()),
                event_type: "account_status_update".to_string(),
                message: format!("status=unavailable reason=old_{index}"),
                created_at: 1 + index,
            })
            .expect("insert old status event");
    }

    let removed = storage
        .prune_events_by_retention_limited(1_300_000, 1)
        .expect("prune events");

    assert_eq!(removed, 1);
    assert_eq!(storage.event_count().expect("event count"), 2);
}
