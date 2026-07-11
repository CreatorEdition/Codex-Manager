use super::{
    clear_candidate_cache_for_tests, collect_gateway_candidates,
    collect_gateway_candidates_with_low_quota_mode, load_usage_snapshots_for_candidates,
    LowQuotaCandidateMode, CANDIDATE_CACHE_TTL_ENV, LOW_QUOTA_THRESHOLD_ENV,
    QUOTA_GUARD_ALLOW_ALL_LOW_FALLBACK_ENV,
};
use crate::account_status::mark_account_unavailable_for_gateway_error;
use codexmanager_core::storage::{now_ts, Account, Storage, Token, UsageSnapshotRecord};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

/// 默认候选缓存 TTL 不能是亚秒级，否则几千账号场景下会频繁重建候选池。
#[test]
fn default_candidate_cache_ttl_avoids_subsecond_rebuilds() {
    let _guard = crate::test_env_guard();
    let previous_ttl = std::env::var(CANDIDATE_CACHE_TTL_ENV).ok();
    std::env::remove_var(CANDIDATE_CACHE_TTL_ENV);
    super::reload_from_env();

    assert!(
        super::candidate_cache_ttl() >= std::time::Duration::from_secs(5),
        "默认候选缓存 TTL 不应是亚秒级，否则几千账号下会频繁重建候选池"
    );

    if let Some(value) = previous_ttl {
        std::env::set_var(CANDIDATE_CACHE_TTL_ENV, value);
    } else {
        std::env::remove_var(CANDIDATE_CACHE_TTL_ENV);
    }
    super::reload_from_env();
}

/// 函数 `candidate_snapshot_cache_reuses_recent_snapshot`
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
fn candidate_snapshot_cache_reuses_recent_snapshot() {
    let _guard = crate::test_env_guard();
    let previous_ttl = std::env::var(CANDIDATE_CACHE_TTL_ENV).ok();
    let previous_db_path = std::env::var("CODEXMANAGER_DB_PATH").ok();
    std::env::set_var(CANDIDATE_CACHE_TTL_ENV, "2000");
    std::env::set_var("CODEXMANAGER_DB_PATH", "selection-cache-test-1");
    super::reload_from_env();
    clear_candidate_cache_for_tests();

    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    storage
        .insert_account(&Account {
            id: "acc-cache-1".to_string(),
            label: "cached".to_string(),
            issuer: "issuer".to_string(),
            chatgpt_account_id: None,
            workspace_id: None,
            group_name: None,
            sort: 0,
            status: "active".to_string(),
            created_at: now_ts(),
            updated_at: now_ts(),
        })
        .expect("insert account");
    storage
        .insert_token(&Token {
            account_id: "acc-cache-1".to_string(),
            id_token: "id".to_string(),
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
            api_key_access_token: None,
            last_refresh: now_ts(),
        })
        .expect("insert token");
    storage
        .insert_usage_snapshot(&UsageSnapshotRecord {
            account_id: "acc-cache-1".to_string(),
            used_percent: Some(10.0),
            window_minutes: Some(300),
            resets_at: None,
            secondary_used_percent: None,
            secondary_window_minutes: None,
            secondary_resets_at: None,
            credits_json: None,
            captured_at: now_ts(),
        })
        .expect("insert snapshot");

    let first = collect_gateway_candidates(&storage).expect("first candidates");
    assert_eq!(first.len(), 1);

    storage
        .update_account_status("acc-cache-1", "inactive")
        .expect("mark inactive");
    let second = collect_gateway_candidates(&storage).expect("second candidates");
    assert_eq!(second.len(), 1);
    assert!(
        Arc::ptr_eq(&first, &second),
        "缓存命中应只克隆 Arc 快照，不能深拷贝整个候选列表"
    );

    clear_candidate_cache_for_tests();
    if let Some(value) = previous_ttl {
        std::env::set_var(CANDIDATE_CACHE_TTL_ENV, value);
    } else {
        std::env::remove_var(CANDIDATE_CACHE_TTL_ENV);
    }
    if let Some(value) = previous_db_path {
        std::env::set_var("CODEXMANAGER_DB_PATH", value);
    } else {
        std::env::remove_var("CODEXMANAGER_DB_PATH");
    }
    super::reload_from_env();
}

/// 验证单账号删除成功后立即清除候选缓存。
#[test]
fn deleting_account_invalidates_cached_candidate_snapshot() {
    let _guard = crate::test_env_guard();
    let previous_ttl = std::env::var(CANDIDATE_CACHE_TTL_ENV).ok();
    let previous_db_path = std::env::var("CODEXMANAGER_DB_PATH").ok();
    let db_path = std::env::temp_dir().join(format!(
        "selection-delete-cache-{}-{}.sqlite",
        std::process::id(),
        now_ts()
    ));
    let _ = std::fs::remove_file(&db_path);
    std::env::set_var(CANDIDATE_CACHE_TTL_ENV, "2000");
    std::env::set_var("CODEXMANAGER_DB_PATH", &db_path);
    super::reload_from_env();
    clear_candidate_cache_for_tests();

    let storage = Storage::open(&db_path).expect("open");
    storage.init().expect("init");
    let now = now_ts();
    storage
        .insert_account(&Account {
            id: "acc-delete-cache".to_string(),
            label: "delete cache".to_string(),
            issuer: "issuer".to_string(),
            chatgpt_account_id: None,
            workspace_id: None,
            group_name: None,
            sort: 0,
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        })
        .expect("insert account");
    storage
        .insert_token(&Token {
            account_id: "acc-delete-cache".to_string(),
            id_token: "id".to_string(),
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
            api_key_access_token: None,
            last_refresh: now,
        })
        .expect("insert token");

    assert_eq!(
        collect_gateway_candidates(&storage)
            .expect("cached candidates")
            .len(),
        1
    );
    crate::account_delete::delete_account("acc-delete-cache").expect("delete account");
    assert!(
        collect_gateway_candidates(&storage)
            .expect("candidates after deletion")
            .is_empty(),
        "单删成功后不能继续返回缓存中的旧账号凭据"
    );

    clear_candidate_cache_for_tests();
    let _ = std::fs::remove_file(&db_path);
    if let Some(value) = previous_ttl {
        std::env::set_var(CANDIDATE_CACHE_TTL_ENV, value);
    } else {
        std::env::remove_var(CANDIDATE_CACHE_TTL_ENV);
    }
    if let Some(value) = previous_db_path {
        std::env::set_var("CODEXMANAGER_DB_PATH", value);
    } else {
        std::env::remove_var("CODEXMANAGER_DB_PATH");
    }
    super::reload_from_env();
}

/// 验证批量删除成功后不会继续返回已删除账号的候选快照。
#[test]
fn deleting_many_accounts_invalidates_cached_candidate_snapshot() {
    let _guard = crate::test_env_guard();
    let previous_ttl = std::env::var(CANDIDATE_CACHE_TTL_ENV).ok();
    let previous_db_path = std::env::var("CODEXMANAGER_DB_PATH").ok();
    let db_path = std::env::temp_dir().join(format!(
        "selection-delete-many-cache-{}-{}.sqlite",
        std::process::id(),
        now_ts()
    ));
    let _ = std::fs::remove_file(&db_path);
    std::env::set_var(CANDIDATE_CACHE_TTL_ENV, "2000");
    std::env::set_var("CODEXMANAGER_DB_PATH", &db_path);
    super::reload_from_env();
    clear_candidate_cache_for_tests();

    let storage = Storage::open(&db_path).expect("open");
    storage.init().expect("init");
    let now = now_ts();
    for (id, sort) in [("acc-delete-many-a", 0_i64), ("acc-delete-many-b", 1_i64)] {
        storage
            .insert_account(&Account {
                id: id.to_string(),
                label: id.to_string(),
                issuer: "issuer".to_string(),
                chatgpt_account_id: None,
                workspace_id: None,
                group_name: None,
                sort,
                status: "active".to_string(),
                created_at: now,
                updated_at: now,
            })
            .expect("insert account");
        storage
            .insert_token(&Token {
                account_id: id.to_string(),
                id_token: "id".to_string(),
                access_token: "access".to_string(),
                refresh_token: "refresh".to_string(),
                api_key_access_token: None,
                last_refresh: now,
            })
            .expect("insert token");
    }

    assert_eq!(
        collect_gateway_candidates(&storage)
            .expect("cached candidates")
            .len(),
        2
    );
    crate::account_delete_many::delete_accounts(vec!["acc-delete-many-a".to_string()])
        .expect("delete accounts");
    let remaining = collect_gateway_candidates(&storage).expect("candidates after bulk deletion");
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].0.id, "acc-delete-many-b");

    clear_candidate_cache_for_tests();
    let _ = std::fs::remove_file(&db_path);
    if let Some(value) = previous_ttl {
        std::env::set_var(CANDIDATE_CACHE_TTL_ENV, value);
    } else {
        std::env::remove_var(CANDIDATE_CACHE_TTL_ENV);
    }
    if let Some(value) = previous_db_path {
        std::env::set_var("CODEXMANAGER_DB_PATH", value);
    } else {
        std::env::remove_var("CODEXMANAGER_DB_PATH");
    }
    super::reload_from_env();
}

#[test]
fn candidate_cache_refresh_is_single_flight_per_cache_window() {
    let _guard = crate::test_env_guard();
    let previous_ttl = std::env::var(CANDIDATE_CACHE_TTL_ENV).ok();
    let previous_db_path = std::env::var("CODEXMANAGER_DB_PATH").ok();
    std::env::set_var(CANDIDATE_CACHE_TTL_ENV, "2000");
    std::env::set_var("CODEXMANAGER_DB_PATH", "selection-cache-single-flight");
    super::reload_from_env();
    clear_candidate_cache_for_tests();

    let first_guard = super::acquire_candidate_cache_refresh(LowQuotaCandidateMode::NormalOnly)
        .expect("first refresh guard");
    let (started_tx, started_rx) = mpsc::channel();
    let (tx, rx) = mpsc::channel();
    let waiter = thread::spawn(move || {
        started_tx.send(()).expect("send started");
        let _second_guard =
            super::acquire_candidate_cache_refresh(LowQuotaCandidateMode::NormalOnly)
                .expect("second refresh guard");
        tx.send(()).expect("send completion");
    });

    started_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("waiter should start");
    assert!(
        rx.recv_timeout(Duration::from_millis(100)).is_err(),
        "second refresh should wait while first refresh is active"
    );
    drop(first_guard);
    rx.recv_timeout(Duration::from_secs(2))
        .expect("second refresh should continue after first guard drops");
    waiter.join().expect("waiter thread");

    clear_candidate_cache_for_tests();
    if let Some(value) = previous_ttl {
        std::env::set_var(CANDIDATE_CACHE_TTL_ENV, value);
    } else {
        std::env::remove_var(CANDIDATE_CACHE_TTL_ENV);
    }
    if let Some(value) = previous_db_path {
        std::env::set_var("CODEXMANAGER_DB_PATH", value);
    } else {
        std::env::remove_var("CODEXMANAGER_DB_PATH");
    }
    super::reload_from_env();
}

/// 验证不同低额度候选模式可以并行刷新，不共享全局串行锁。
#[test]
fn candidate_cache_refresh_allows_different_modes_in_parallel() {
    let _guard = crate::test_env_guard();
    let previous_ttl = std::env::var(CANDIDATE_CACHE_TTL_ENV).ok();
    let previous_db_path = std::env::var("CODEXMANAGER_DB_PATH").ok();
    std::env::set_var(CANDIDATE_CACHE_TTL_ENV, "2000");
    std::env::set_var(
        "CODEXMANAGER_DB_PATH",
        "selection-cache-mode-parallel-refresh",
    );
    super::reload_from_env();
    clear_candidate_cache_for_tests();

    let normal_guard = super::acquire_candidate_cache_refresh(LowQuotaCandidateMode::NormalOnly)
        .expect("normal refresh guard");
    let (tx, rx) = mpsc::channel();
    let fallback_worker = thread::spawn(move || {
        let _fallback_guard =
            super::acquire_candidate_cache_refresh(LowQuotaCandidateMode::AppendFallback)
                .expect("fallback refresh guard");
        tx.send(()).expect("send fallback completion");
    });

    rx.recv_timeout(Duration::from_secs(2))
        .expect("不同候选模式不应共用全局 single-flight 锁");
    drop(normal_guard);
    fallback_worker.join().expect("fallback worker");

    clear_candidate_cache_for_tests();
    if let Some(value) = previous_ttl {
        std::env::set_var(CANDIDATE_CACHE_TTL_ENV, value);
    } else {
        std::env::remove_var(CANDIDATE_CACHE_TTL_ENV);
    }
    if let Some(value) = previous_db_path {
        std::env::set_var("CODEXMANAGER_DB_PATH", value);
    } else {
        std::env::remove_var("CODEXMANAGER_DB_PATH");
    }
    super::reload_from_env();
}

/// 验证交替读取两种候选模式时，各自缓存不会相互驱逐。
#[test]
fn alternating_candidate_modes_keep_independent_cached_snapshots() {
    let _guard = crate::test_env_guard();
    let previous_ttl = std::env::var(CANDIDATE_CACHE_TTL_ENV).ok();
    let previous_db_path = std::env::var("CODEXMANAGER_DB_PATH").ok();
    let previous_threshold = std::env::var(LOW_QUOTA_THRESHOLD_ENV).ok();
    std::env::set_var(CANDIDATE_CACHE_TTL_ENV, "2000");
    std::env::set_var("CODEXMANAGER_DB_PATH", "selection-cache-alternating-modes");
    std::env::set_var(LOW_QUOTA_THRESHOLD_ENV, "95");
    super::reload_from_env();
    clear_candidate_cache_for_tests();

    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    let now = now_ts();
    for (id, sort, used_percent) in [
        ("acc-healthy-cache", 0_i64, 10.0),
        ("acc-low-cache", 1_i64, 99.0),
    ] {
        storage
            .insert_account(&Account {
                id: id.to_string(),
                label: id.to_string(),
                issuer: "issuer".to_string(),
                chatgpt_account_id: None,
                workspace_id: None,
                group_name: None,
                sort,
                status: "active".to_string(),
                created_at: now,
                updated_at: now,
            })
            .expect("insert account");
        storage
            .insert_token(&Token {
                account_id: id.to_string(),
                id_token: "id".to_string(),
                access_token: "access".to_string(),
                refresh_token: "refresh".to_string(),
                api_key_access_token: None,
                last_refresh: now,
            })
            .expect("insert token");
        storage
            .insert_usage_snapshot(&UsageSnapshotRecord {
                account_id: id.to_string(),
                used_percent: Some(used_percent),
                window_minutes: Some(300),
                resets_at: None,
                secondary_used_percent: None,
                secondary_window_minutes: None,
                secondary_resets_at: None,
                credits_json: None,
                captured_at: now,
            })
            .expect("insert snapshot");
    }

    let normal_first = collect_gateway_candidates(&storage).expect("normal candidates");
    let fallback = collect_gateway_candidates_with_low_quota_mode(
        &storage,
        LowQuotaCandidateMode::AppendFallback,
    )
    .expect("fallback candidates");
    let normal_second = collect_gateway_candidates(&storage).expect("normal cached candidates");

    assert_eq!(normal_first.len(), 1);
    assert_eq!(fallback.len(), 2);
    assert!(
        Arc::ptr_eq(&normal_first, &normal_second),
        "交替读取不同模式时不应驱逐另一个模式的候选快照"
    );

    clear_candidate_cache_for_tests();
    if let Some(value) = previous_ttl {
        std::env::set_var(CANDIDATE_CACHE_TTL_ENV, value);
    } else {
        std::env::remove_var(CANDIDATE_CACHE_TTL_ENV);
    }
    if let Some(value) = previous_db_path {
        std::env::set_var("CODEXMANAGER_DB_PATH", value);
    } else {
        std::env::remove_var("CODEXMANAGER_DB_PATH");
    }
    if let Some(value) = previous_threshold {
        std::env::set_var(LOW_QUOTA_THRESHOLD_ENV, value);
    } else {
        std::env::remove_var(LOW_QUOTA_THRESHOLD_ENV);
    }
    super::reload_from_env();
}

/// 函数 `candidates_follow_account_sort_order`
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
fn candidates_follow_account_sort_order() {
    let _guard = crate::test_env_guard();
    let previous_ttl = std::env::var(CANDIDATE_CACHE_TTL_ENV).ok();
    let previous_db_path = std::env::var("CODEXMANAGER_DB_PATH").ok();
    std::env::set_var(CANDIDATE_CACHE_TTL_ENV, "0");
    std::env::set_var("CODEXMANAGER_DB_PATH", "selection-cache-test-2");
    super::reload_from_env();
    clear_candidate_cache_for_tests();

    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");

    let now = now_ts();
    let accounts = vec![
        ("acc-sort-10", 10_i64),
        ("acc-sort-0", 0_i64),
        ("acc-sort-1", 1_i64),
    ];
    for (id, sort) in &accounts {
        storage
            .insert_account(&Account {
                id: (*id).to_string(),
                label: (*id).to_string(),
                issuer: "issuer".to_string(),
                chatgpt_account_id: None,
                workspace_id: None,
                group_name: None,
                sort: *sort,
                status: "active".to_string(),
                created_at: now,
                updated_at: now,
            })
            .expect("insert account");
        storage
            .insert_token(&Token {
                account_id: (*id).to_string(),
                id_token: "id".to_string(),
                access_token: "access".to_string(),
                refresh_token: "refresh".to_string(),
                api_key_access_token: None,
                last_refresh: now,
            })
            .expect("insert token");
        storage
            .insert_usage_snapshot(&UsageSnapshotRecord {
                account_id: (*id).to_string(),
                used_percent: Some(10.0),
                window_minutes: Some(300),
                resets_at: None,
                secondary_used_percent: None,
                secondary_window_minutes: None,
                secondary_resets_at: None,
                credits_json: None,
                captured_at: now,
            })
            .expect("insert usage");
    }

    let candidates = collect_gateway_candidates(&storage).expect("collect candidates");
    let ordered_ids = candidates
        .iter()
        .map(|(account, _)| account.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ordered_ids, vec!["acc-sort-0", "acc-sort-1", "acc-sort-10"]);

    clear_candidate_cache_for_tests();
    if let Some(value) = previous_ttl {
        std::env::set_var(CANDIDATE_CACHE_TTL_ENV, value);
    } else {
        std::env::remove_var(CANDIDATE_CACHE_TTL_ENV);
    }
    if let Some(value) = previous_db_path {
        std::env::set_var("CODEXMANAGER_DB_PATH", value);
    } else {
        std::env::remove_var("CODEXMANAGER_DB_PATH");
    }
    super::reload_from_env();
}

/// 配额保护只应读取当前候选账号的用量快照，避免网关请求在缓存失效时扫描无关账号快照。
#[test]
fn quota_guard_usage_lookup_scopes_to_candidate_accounts() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    let now = now_ts();

    for (account_id, used_percent) in [
        ("candidate-a", 10.0),
        ("candidate-b", 20.0),
        ("unrelated-low-quota", 99.0),
    ] {
        storage
            .insert_usage_snapshot(&UsageSnapshotRecord {
                account_id: account_id.to_string(),
                used_percent: Some(used_percent),
                window_minutes: Some(300),
                resets_at: None,
                secondary_used_percent: None,
                secondary_window_minutes: None,
                secondary_resets_at: None,
                credits_json: None,
                captured_at: now,
            })
            .expect("insert snapshot");
    }

    let candidates = ["candidate-a", "candidate-b"]
        .into_iter()
        .enumerate()
        .map(|(sort, id)| {
            (
                Account {
                    id: id.to_string(),
                    label: id.to_string(),
                    issuer: "issuer".to_string(),
                    chatgpt_account_id: None,
                    workspace_id: None,
                    group_name: None,
                    sort: sort as i64,
                    status: "active".to_string(),
                    created_at: now,
                    updated_at: now,
                },
                Token {
                    account_id: id.to_string(),
                    id_token: "id".to_string(),
                    access_token: "access".to_string(),
                    refresh_token: "refresh".to_string(),
                    api_key_access_token: None,
                    last_refresh: now,
                },
            )
        })
        .collect::<Vec<_>>();

    let snapshots = load_usage_snapshots_for_candidates(&storage, &candidates);

    assert_eq!(snapshots.len(), 2);
    assert!(snapshots.contains_key("candidate-a"));
    assert!(snapshots.contains_key("candidate-b"));
    assert!(!snapshots.contains_key("unrelated-low-quota"));
}

#[test]
fn no_candidate_diagnostic_samples_accounts_without_full_detail_load() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    let now = now_ts();

    for index in 0..20 {
        let account_id = format!("diag-{index:02}");
        storage
            .insert_account(&Account {
                id: account_id.clone(),
                label: account_id.clone(),
                issuer: "issuer".to_string(),
                chatgpt_account_id: None,
                workspace_id: None,
                group_name: None,
                sort: index,
                status: "inactive".to_string(),
                created_at: now,
                updated_at: now + index,
            })
            .expect("insert diagnostic account");
        storage
            .insert_token(&Token {
                account_id: account_id.clone(),
                id_token: "id".to_string(),
                access_token: "access".to_string(),
                refresh_token: "refresh".to_string(),
                api_key_access_token: None,
                last_refresh: now,
            })
            .expect("insert diagnostic token");
        storage
            .insert_usage_snapshot(&UsageSnapshotRecord {
                account_id,
                used_percent: Some(index as f64),
                window_minutes: Some(300),
                resets_at: None,
                secondary_used_percent: Some(index as f64 + 1.0),
                secondary_window_minutes: Some(10080),
                secondary_resets_at: None,
                credits_json: None,
                captured_at: now + index,
            })
            .expect("insert diagnostic usage");
    }

    let diagnostic = super::build_no_candidate_diagnostic(&storage, 3);

    assert_eq!(diagnostic.account_total, 20);
    assert_eq!(diagnostic.token_total, 20);
    assert_eq!(diagnostic.usage_snapshot_total, 20);
    assert_eq!(diagnostic.sample_limit, 3);
    assert_eq!(diagnostic.samples.len(), 3);
    let sample_ids = diagnostic
        .samples
        .iter()
        .map(|sample| sample.account.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(sample_ids, vec!["diag-00", "diag-01", "diag-02"]);
    assert!(diagnostic.samples.iter().all(|sample| sample.has_token));
    assert!(diagnostic
        .samples
        .iter()
        .all(|sample| sample.usage.is_some()));
}

/// 函数 `gateway_error_status_change_invalidates_candidate_snapshot_cache`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-03
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn gateway_error_status_change_invalidates_candidate_snapshot_cache() {
    let _guard = crate::test_env_guard();
    let previous_ttl = std::env::var(CANDIDATE_CACHE_TTL_ENV).ok();
    let previous_db_path = std::env::var("CODEXMANAGER_DB_PATH").ok();
    std::env::set_var(CANDIDATE_CACHE_TTL_ENV, "2000");
    std::env::set_var("CODEXMANAGER_DB_PATH", "selection-cache-test-3");
    super::reload_from_env();
    clear_candidate_cache_for_tests();

    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    let now = now_ts();
    storage
        .insert_account(&Account {
            id: "acc-cache-usage-limit".to_string(),
            label: "cache-usage-limit".to_string(),
            issuer: "issuer".to_string(),
            chatgpt_account_id: None,
            workspace_id: None,
            group_name: None,
            sort: 0,
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        })
        .expect("insert account");
    storage
        .insert_token(&Token {
            account_id: "acc-cache-usage-limit".to_string(),
            id_token: "id".to_string(),
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
            api_key_access_token: None,
            last_refresh: now,
        })
        .expect("insert token");
    storage
        .insert_usage_snapshot(&UsageSnapshotRecord {
            account_id: "acc-cache-usage-limit".to_string(),
            used_percent: Some(10.0),
            window_minutes: Some(300),
            resets_at: None,
            secondary_used_percent: None,
            secondary_window_minutes: None,
            secondary_resets_at: None,
            credits_json: None,
            captured_at: now,
        })
        .expect("insert snapshot");

    let first = collect_gateway_candidates(&storage).expect("first candidates");
    assert_eq!(first.len(), 1);

    assert!(mark_account_unavailable_for_gateway_error(
        &storage,
        "acc-cache-usage-limit",
        "You've hit your usage limit. To get more access now, try again at 8:02 PM."
    ));

    let second = collect_gateway_candidates(&storage).expect("second candidates");
    assert!(
        second.is_empty(),
        "usage-limit should mark the account limited and evict cached candidate"
    );

    clear_candidate_cache_for_tests();
    if let Some(value) = previous_ttl {
        std::env::set_var(CANDIDATE_CACHE_TTL_ENV, value);
    } else {
        std::env::remove_var(CANDIDATE_CACHE_TTL_ENV);
    }
    if let Some(value) = previous_db_path {
        std::env::set_var("CODEXMANAGER_DB_PATH", value);
    } else {
        std::env::remove_var("CODEXMANAGER_DB_PATH");
    }
    super::reload_from_env();
}

/// 函数 `gateway_deactivation_status_change_invalidates_candidate_snapshot_cache`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-03
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn gateway_deactivation_status_change_invalidates_candidate_snapshot_cache() {
    let _guard = crate::test_env_guard();
    let previous_ttl = std::env::var(CANDIDATE_CACHE_TTL_ENV).ok();
    let previous_db_path = std::env::var("CODEXMANAGER_DB_PATH").ok();
    std::env::set_var(CANDIDATE_CACHE_TTL_ENV, "2000");
    std::env::set_var("CODEXMANAGER_DB_PATH", "selection-cache-test-4");
    super::reload_from_env();
    clear_candidate_cache_for_tests();

    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    let now = now_ts();
    storage
        .insert_account(&Account {
            id: "acc-cache-deactivated".to_string(),
            label: "cache-deactivated".to_string(),
            issuer: "issuer".to_string(),
            chatgpt_account_id: None,
            workspace_id: None,
            group_name: None,
            sort: 0,
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        })
        .expect("insert account");
    storage
        .insert_token(&Token {
            account_id: "acc-cache-deactivated".to_string(),
            id_token: "id".to_string(),
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
            api_key_access_token: None,
            last_refresh: now,
        })
        .expect("insert token");
    storage
        .insert_usage_snapshot(&UsageSnapshotRecord {
            account_id: "acc-cache-deactivated".to_string(),
            used_percent: Some(10.0),
            window_minutes: Some(300),
            resets_at: None,
            secondary_used_percent: None,
            secondary_window_minutes: None,
            secondary_resets_at: None,
            credits_json: None,
            captured_at: now,
        })
        .expect("insert snapshot");

    let first = collect_gateway_candidates(&storage).expect("first candidates");
    assert_eq!(first.len(), 1);

    assert!(mark_account_unavailable_for_gateway_error(
        &storage,
        "acc-cache-deactivated",
        "Your OpenAI account has been deactivated"
    ));

    let second = collect_gateway_candidates(&storage).expect("second candidates");
    assert!(
        second.is_empty(),
        "deactivation should invalidate cached candidate"
    );

    clear_candidate_cache_for_tests();
    if let Some(value) = previous_ttl {
        std::env::set_var(CANDIDATE_CACHE_TTL_ENV, value);
    } else {
        std::env::remove_var(CANDIDATE_CACHE_TTL_ENV);
    }
    if let Some(value) = previous_db_path {
        std::env::set_var("CODEXMANAGER_DB_PATH", value);
    } else {
        std::env::remove_var("CODEXMANAGER_DB_PATH");
    }
    super::reload_from_env();
}

/// 函数 `gateway_usage_limit_with_exhausted_snapshot_invalidates_candidate_snapshot_cache`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-03
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn gateway_usage_limit_with_exhausted_snapshot_invalidates_candidate_snapshot_cache() {
    let _guard = crate::test_env_guard();
    let previous_ttl = std::env::var(CANDIDATE_CACHE_TTL_ENV).ok();
    let previous_db_path = std::env::var("CODEXMANAGER_DB_PATH").ok();
    std::env::set_var(CANDIDATE_CACHE_TTL_ENV, "2000");
    std::env::set_var("CODEXMANAGER_DB_PATH", "selection-cache-test-5");
    super::reload_from_env();
    clear_candidate_cache_for_tests();

    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    let now = now_ts();
    storage
        .insert_account(&Account {
            id: "acc-cache-usage-exhausted".to_string(),
            label: "cache-usage-exhausted".to_string(),
            issuer: "issuer".to_string(),
            chatgpt_account_id: None,
            workspace_id: None,
            group_name: None,
            sort: 0,
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        })
        .expect("insert account");
    storage
        .insert_token(&Token {
            account_id: "acc-cache-usage-exhausted".to_string(),
            id_token: "id".to_string(),
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
            api_key_access_token: None,
            last_refresh: now,
        })
        .expect("insert token");
    storage
        .insert_usage_snapshot(&UsageSnapshotRecord {
            account_id: "acc-cache-usage-exhausted".to_string(),
            used_percent: Some(10.0),
            window_minutes: Some(300),
            resets_at: None,
            secondary_used_percent: Some(10.0),
            secondary_window_minutes: Some(10080),
            secondary_resets_at: None,
            credits_json: None,
            captured_at: now,
        })
        .expect("insert snapshot");

    let first = collect_gateway_candidates(&storage).expect("first candidates");
    assert_eq!(first.len(), 1);

    storage
        .insert_usage_snapshot(&UsageSnapshotRecord {
            account_id: "acc-cache-usage-exhausted".to_string(),
            used_percent: Some(100.0),
            window_minutes: Some(300),
            resets_at: None,
            secondary_used_percent: Some(100.0),
            secondary_window_minutes: Some(10080),
            secondary_resets_at: None,
            credits_json: None,
            captured_at: now + 1,
        })
        .expect("insert exhausted snapshot");

    assert!(mark_account_unavailable_for_gateway_error(
        &storage,
        "acc-cache-usage-exhausted",
        "You've hit your usage limit. To get more access now, try again at 8:02 PM."
    ));

    let second = collect_gateway_candidates(&storage).expect("second candidates");
    assert!(
        second.is_empty(),
        "confirmed exhausted snapshot should invalidate cached candidate"
    );

    clear_candidate_cache_for_tests();
    if let Some(value) = previous_ttl {
        std::env::set_var(CANDIDATE_CACHE_TTL_ENV, value);
    } else {
        std::env::remove_var(CANDIDATE_CACHE_TTL_ENV);
    }
    if let Some(value) = previous_db_path {
        std::env::set_var("CODEXMANAGER_DB_PATH", value);
    } else {
        std::env::remove_var("CODEXMANAGER_DB_PATH");
    }
    super::reload_from_env();
}

/// 低配额账号（used_percent 超过阈值）应当进入低额度备用池（候选列表尾部），
/// 配额充足的账号优先被挑选，保留每个池子内的原有顺序。
#[test]
fn low_quota_accounts_are_skipped_when_healthy_available() {
    let _guard = crate::test_env_guard();
    let previous_ttl = std::env::var(CANDIDATE_CACHE_TTL_ENV).ok();
    let previous_db_path = std::env::var("CODEXMANAGER_DB_PATH").ok();
    let previous_threshold = std::env::var(LOW_QUOTA_THRESHOLD_ENV).ok();
    std::env::set_var(CANDIDATE_CACHE_TTL_ENV, "0");
    std::env::set_var("CODEXMANAGER_DB_PATH", "selection-low-quota-test");
    std::env::set_var(LOW_QUOTA_THRESHOLD_ENV, "95");
    super::reload_from_env();
    clear_candidate_cache_for_tests();

    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");

    let now = now_ts();
    let rows: Vec<(&str, i64, f64, Option<f64>)> = vec![
        ("acc-exhausted", 0, 99.0, None),
        ("acc-healthy-high", 1, 10.0, None),
        ("acc-secondary-low", 2, 10.0, Some(99.0)),
        ("acc-healthy-low", 3, 5.0, None),
    ];
    for (id, sort, primary_pct, secondary_pct) in &rows {
        storage
            .insert_account(&Account {
                id: (*id).to_string(),
                label: (*id).to_string(),
                issuer: "issuer".to_string(),
                chatgpt_account_id: None,
                workspace_id: None,
                group_name: None,
                sort: *sort,
                status: "active".to_string(),
                created_at: now,
                updated_at: now,
            })
            .expect("insert account");
        storage
            .insert_token(&Token {
                account_id: (*id).to_string(),
                id_token: "id".to_string(),
                access_token: "access".to_string(),
                refresh_token: "refresh".to_string(),
                api_key_access_token: None,
                last_refresh: now,
            })
            .expect("insert token");
        storage
            .insert_usage_snapshot(&UsageSnapshotRecord {
                account_id: (*id).to_string(),
                used_percent: Some(*primary_pct),
                window_minutes: Some(300),
                resets_at: None,
                secondary_used_percent: *secondary_pct,
                secondary_window_minutes: secondary_pct.map(|_| 10_080),
                secondary_resets_at: None,
                credits_json: None,
                captured_at: now,
            })
            .expect("insert snapshot");
    }

    let candidates = collect_gateway_candidates(&storage).expect("collect candidates");
    let ids: Vec<&str> = candidates
        .iter()
        .map(|(account, _)| account.id.as_str())
        .collect();
    assert_eq!(ids, vec!["acc-healthy-high", "acc-healthy-low"]);

    clear_candidate_cache_for_tests();
    if let Some(value) = previous_ttl {
        std::env::set_var(CANDIDATE_CACHE_TTL_ENV, value);
    } else {
        std::env::remove_var(CANDIDATE_CACHE_TTL_ENV);
    }
    if let Some(value) = previous_db_path {
        std::env::set_var("CODEXMANAGER_DB_PATH", value);
    } else {
        std::env::remove_var("CODEXMANAGER_DB_PATH");
    }
    if let Some(value) = previous_threshold {
        std::env::set_var(LOW_QUOTA_THRESHOLD_ENV, value);
    } else {
        std::env::remove_var(LOW_QUOTA_THRESHOLD_ENV);
    }
    super::reload_from_env();
}

#[test]
fn append_low_quota_fallback_keeps_low_quota_candidates_at_tail() {
    let _guard = crate::test_env_guard();
    let previous_ttl = std::env::var(CANDIDATE_CACHE_TTL_ENV).ok();
    let previous_db_path = std::env::var("CODEXMANAGER_DB_PATH").ok();
    let previous_threshold = std::env::var(LOW_QUOTA_THRESHOLD_ENV).ok();
    std::env::set_var(CANDIDATE_CACHE_TTL_ENV, "0");
    std::env::set_var(
        "CODEXMANAGER_DB_PATH",
        "selection-low-quota-append-fallback-test",
    );
    std::env::set_var(LOW_QUOTA_THRESHOLD_ENV, "95");
    super::reload_from_env();
    clear_candidate_cache_for_tests();

    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");

    let now = now_ts();
    for (id, sort, used_percent) in [
        ("acc-low-a", 0_i64, 99.0),
        ("acc-healthy", 1_i64, 10.0),
        ("acc-low-b", 2_i64, 98.0),
    ] {
        storage
            .insert_account(&Account {
                id: id.to_string(),
                label: id.to_string(),
                issuer: "issuer".to_string(),
                chatgpt_account_id: None,
                workspace_id: None,
                group_name: None,
                sort,
                status: "active".to_string(),
                created_at: now,
                updated_at: now,
            })
            .expect("insert account");
        storage
            .insert_token(&Token {
                account_id: id.to_string(),
                id_token: "id".to_string(),
                access_token: "access".to_string(),
                refresh_token: "refresh".to_string(),
                api_key_access_token: None,
                last_refresh: now,
            })
            .expect("insert token");
        storage
            .insert_usage_snapshot(&UsageSnapshotRecord {
                account_id: id.to_string(),
                used_percent: Some(used_percent),
                window_minutes: Some(300),
                resets_at: None,
                secondary_used_percent: None,
                secondary_window_minutes: None,
                secondary_resets_at: None,
                credits_json: None,
                captured_at: now,
            })
            .expect("insert snapshot");
    }

    let candidates = collect_gateway_candidates_with_low_quota_mode(
        &storage,
        LowQuotaCandidateMode::AppendFallback,
    )
    .expect("collect candidates");
    let ids: Vec<&str> = candidates
        .iter()
        .map(|(account, _)| account.id.as_str())
        .collect();
    assert_eq!(ids, vec!["acc-healthy", "acc-low-a", "acc-low-b"]);

    clear_candidate_cache_for_tests();
    if let Some(value) = previous_ttl {
        std::env::set_var(CANDIDATE_CACHE_TTL_ENV, value);
    } else {
        std::env::remove_var(CANDIDATE_CACHE_TTL_ENV);
    }
    if let Some(value) = previous_db_path {
        std::env::set_var("CODEXMANAGER_DB_PATH", value);
    } else {
        std::env::remove_var("CODEXMANAGER_DB_PATH");
    }
    if let Some(value) = previous_threshold {
        std::env::set_var(LOW_QUOTA_THRESHOLD_ENV, value);
    } else {
        std::env::remove_var(LOW_QUOTA_THRESHOLD_ENV);
    }
    super::reload_from_env();
}

/// 全部账号都触阈时仍应返回低额度候选，供正常池不可用后的兜底使用。
#[test]
fn all_low_quota_still_returns_candidates() {
    let _guard = crate::test_env_guard();
    let previous_ttl = std::env::var(CANDIDATE_CACHE_TTL_ENV).ok();
    let previous_db_path = std::env::var("CODEXMANAGER_DB_PATH").ok();
    let previous_threshold = std::env::var(LOW_QUOTA_THRESHOLD_ENV).ok();
    std::env::set_var(CANDIDATE_CACHE_TTL_ENV, "0");
    std::env::set_var("CODEXMANAGER_DB_PATH", "selection-all-low-quota-test");
    std::env::set_var(LOW_QUOTA_THRESHOLD_ENV, "95");
    super::reload_from_env();
    clear_candidate_cache_for_tests();

    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");

    let now = now_ts();
    for (id, sort) in &[("acc-a", 0_i64), ("acc-b", 1_i64)] {
        storage
            .insert_account(&Account {
                id: (*id).to_string(),
                label: (*id).to_string(),
                issuer: "issuer".to_string(),
                chatgpt_account_id: None,
                workspace_id: None,
                group_name: None,
                sort: *sort,
                status: "active".to_string(),
                created_at: now,
                updated_at: now,
            })
            .expect("insert account");
        storage
            .insert_token(&Token {
                account_id: (*id).to_string(),
                id_token: "id".to_string(),
                access_token: "access".to_string(),
                refresh_token: "refresh".to_string(),
                api_key_access_token: None,
                last_refresh: now,
            })
            .expect("insert token");
        storage
            .insert_usage_snapshot(&UsageSnapshotRecord {
                account_id: (*id).to_string(),
                used_percent: Some(98.0),
                window_minutes: Some(300),
                resets_at: None,
                secondary_used_percent: None,
                secondary_window_minutes: None,
                secondary_resets_at: None,
                credits_json: None,
                captured_at: now,
            })
            .expect("insert snapshot");
    }

    let candidates = collect_gateway_candidates(&storage).expect("collect candidates");
    let ids: Vec<&str> = candidates
        .iter()
        .map(|(account, _)| account.id.as_str())
        .collect();
    assert_eq!(ids, vec!["acc-a", "acc-b"]);

    clear_candidate_cache_for_tests();
    if let Some(value) = previous_ttl {
        std::env::set_var(CANDIDATE_CACHE_TTL_ENV, value);
    } else {
        std::env::remove_var(CANDIDATE_CACHE_TTL_ENV);
    }
    if let Some(value) = previous_db_path {
        std::env::set_var("CODEXMANAGER_DB_PATH", value);
    } else {
        std::env::remove_var("CODEXMANAGER_DB_PATH");
    }
    if let Some(value) = previous_threshold {
        std::env::set_var(LOW_QUOTA_THRESHOLD_ENV, value);
    } else {
        std::env::remove_var(LOW_QUOTA_THRESHOLD_ENV);
    }
    super::reload_from_env();
}

#[test]
fn all_low_quota_without_fallback_returns_no_candidates() {
    let _guard = crate::test_env_guard();
    let previous_ttl = std::env::var(CANDIDATE_CACHE_TTL_ENV).ok();
    let previous_db_path = std::env::var("CODEXMANAGER_DB_PATH").ok();
    let previous_threshold = std::env::var(LOW_QUOTA_THRESHOLD_ENV).ok();
    let previous_fallback = std::env::var(QUOTA_GUARD_ALLOW_ALL_LOW_FALLBACK_ENV).ok();
    std::env::set_var(CANDIDATE_CACHE_TTL_ENV, "0");
    std::env::set_var(
        "CODEXMANAGER_DB_PATH",
        "selection-all-low-quota-no-fallback-test",
    );
    std::env::set_var(LOW_QUOTA_THRESHOLD_ENV, "95");
    std::env::set_var(QUOTA_GUARD_ALLOW_ALL_LOW_FALLBACK_ENV, "0");
    super::reload_from_env();
    clear_candidate_cache_for_tests();

    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");

    let now = now_ts();
    for (id, sort) in &[("acc-a", 0_i64), ("acc-b", 1_i64)] {
        storage
            .insert_account(&Account {
                id: (*id).to_string(),
                label: (*id).to_string(),
                issuer: "issuer".to_string(),
                chatgpt_account_id: None,
                workspace_id: None,
                group_name: None,
                sort: *sort,
                status: "active".to_string(),
                created_at: now,
                updated_at: now,
            })
            .expect("insert account");
        storage
            .insert_token(&Token {
                account_id: (*id).to_string(),
                id_token: "id".to_string(),
                access_token: "access".to_string(),
                refresh_token: "refresh".to_string(),
                api_key_access_token: None,
                last_refresh: now,
            })
            .expect("insert token");
        storage
            .insert_usage_snapshot(&UsageSnapshotRecord {
                account_id: (*id).to_string(),
                used_percent: Some(98.0),
                window_minutes: Some(300),
                resets_at: None,
                secondary_used_percent: None,
                secondary_window_minutes: None,
                secondary_resets_at: None,
                credits_json: None,
                captured_at: now,
            })
            .expect("insert snapshot");
    }

    let candidates = collect_gateway_candidates(&storage).expect("collect candidates");
    assert!(candidates.is_empty());

    clear_candidate_cache_for_tests();
    if let Some(value) = previous_ttl {
        std::env::set_var(CANDIDATE_CACHE_TTL_ENV, value);
    } else {
        std::env::remove_var(CANDIDATE_CACHE_TTL_ENV);
    }
    if let Some(value) = previous_db_path {
        std::env::set_var("CODEXMANAGER_DB_PATH", value);
    } else {
        std::env::remove_var("CODEXMANAGER_DB_PATH");
    }
    if let Some(value) = previous_threshold {
        std::env::set_var(LOW_QUOTA_THRESHOLD_ENV, value);
    } else {
        std::env::remove_var(LOW_QUOTA_THRESHOLD_ENV);
    }
    if let Some(value) = previous_fallback {
        std::env::set_var(QUOTA_GUARD_ALLOW_ALL_LOW_FALLBACK_ENV, value);
    } else {
        std::env::remove_var(QUOTA_GUARD_ALLOW_ALL_LOW_FALLBACK_ENV);
    }
    super::reload_from_env();
}
