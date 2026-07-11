use super::{
    clear_pending_usage_refresh_tasks_for_tests, enqueue_usage_refresh_with_worker,
    enqueue_usage_validation_after_token_refresh_with, next_usage_poll_cursor,
    notify_usage_refresh_completed, reset_usage_poll_cursor_for_tests,
    resolve_token_refresh_issuer, run_token_refresh_task, schedule_token_refresh_failure_retry,
    set_usage_refresh_completed_handler, should_retry_usage_refresh_with_token,
    sleep_startup_stagger_with, subscribe_usage_refresh_completed, token_refresh_access_exp_cutoff,
    token_refresh_due_cutoff, token_refresh_failure_cooldown_secs, token_refresh_schedule,
    token_refresh_transient_backoff_secs, usage_poll_batch_indices,
    GATEWAY_KEEPALIVE_STARTUP_STAGGER_SECS, TOKEN_REFRESH_STARTUP_STAGGER_SECS,
    USAGE_POLLING_STARTUP_STAGGER_SECS, WARMUP_CRON_STARTUP_STAGGER_SECS,
};
use crate::usage_scheduler::DEFAULT_USAGE_POLL_INTERVAL_SECS;
use codexmanager_core::storage::{now_ts, Account, Event, Storage, Token};
use std::collections::HashSet;
use std::sync::mpsc;
use std::time::Duration;

#[test]
fn background_loops_use_staggered_startup_delays() {
    assert_eq!(USAGE_POLLING_STARTUP_STAGGER_SECS, 5);
    assert_eq!(GATEWAY_KEEPALIVE_STARTUP_STAGGER_SECS, 15);
    assert_eq!(TOKEN_REFRESH_STARTUP_STAGGER_SECS, 25);
    assert_eq!(WARMUP_CRON_STARTUP_STAGGER_SECS, 35);
}

#[test]
fn startup_stagger_sleeps_in_interruptible_chunks() {
    let mut slept = Vec::new();

    let completed = sleep_startup_stagger_with(
        Duration::from_millis(2_500),
        || false,
        |duration| slept.push(duration),
    );

    assert!(completed);
    assert_eq!(
        slept,
        vec![
            Duration::from_secs(1),
            Duration::from_secs(1),
            Duration::from_millis(500)
        ]
    );
}

#[test]
fn startup_stagger_stops_when_shutdown_is_requested() {
    let mut slept = Vec::new();
    let mut checks = 0usize;

    let completed = sleep_startup_stagger_with(
        Duration::from_secs(3),
        || {
            checks = checks.saturating_add(1);
            checks > 1
        },
        |duration| slept.push(duration),
    );

    assert!(!completed);
    assert_eq!(slept, vec![Duration::from_secs(1)]);
}

#[test]
fn usage_refresh_completed_handler_receives_notification() {
    let _guard = crate::test_env_guard();
    let (tx, rx) = mpsc::channel();
    set_usage_refresh_completed_handler(move |event| {
        let _ = tx.send(event);
    });

    notify_usage_refresh_completed("test-notify", 2, 3);
    let event = rx
        .recv_timeout(Duration::from_secs(1))
        .expect("usage refresh completed event");
    assert_eq!(event.source, "test-notify");
    assert_eq!(event.processed, 2);
    assert_eq!(event.total, 3);
    assert!(event.completed_at > 0);
}

#[test]
fn usage_refresh_completed_subscriber_receives_notification() {
    let _guard = crate::test_env_guard();
    let rx = subscribe_usage_refresh_completed();

    notify_usage_refresh_completed("test-subscribe", 1, 1);
    let event = rx
        .recv_timeout(Duration::from_secs(1))
        .expect("usage refresh completed event");
    assert_eq!(event.source, "test-subscribe");
    assert_eq!(event.processed, 1);
    assert_eq!(event.total, 1);
    assert!(event.completed_at > 0);
}

/// 函数 `enqueue_usage_refresh_for_same_account_is_deduplicated_until_finish`
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
fn enqueue_usage_refresh_for_same_account_is_deduplicated_until_finish() {
    let _guard = crate::test_env_guard();
    clear_pending_usage_refresh_tasks_for_tests();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();

    let first = enqueue_usage_refresh_with_worker("acc-dedup", move |_| {
        let _ = started_tx.send(());
        let _ = release_rx.recv();
    });
    assert!(first);
    started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("worker started");

    let second = enqueue_usage_refresh_with_worker("acc-dedup", |_| {});
    assert!(!second);

    let _ = release_tx.send(());
    std::thread::sleep(Duration::from_millis(20));

    let third = enqueue_usage_refresh_with_worker("acc-dedup", |_| {});
    assert!(third);
    std::thread::sleep(Duration::from_millis(20));
    clear_pending_usage_refresh_tasks_for_tests();
}

/// 函数 `enqueue_usage_refresh_for_different_accounts_keeps_queue_progress`
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
fn enqueue_usage_refresh_for_different_accounts_keeps_queue_progress() {
    let _guard = crate::test_env_guard();
    clear_pending_usage_refresh_tasks_for_tests();
    let (started_tx, started_rx) = mpsc::channel::<String>();
    let (release_tx, release_rx) = mpsc::channel();
    let started_tx_first = started_tx.clone();

    let first = enqueue_usage_refresh_with_worker("acc-a", move |_| {
        let _ = started_tx_first.send("acc-a".to_string());
        let _ = release_rx.recv_timeout(Duration::from_secs(1));
    });
    assert!(first);

    let started_tx = started_tx.clone();
    let second = enqueue_usage_refresh_with_worker("acc-b", move |_| {
        let _ = started_tx.send("acc-b".to_string());
    });
    assert!(second);

    let first_started = started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("first task should start");
    let _ = release_tx.send(());
    let second_started = started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("second task should start");

    let seen: HashSet<String> = [first_started, second_started].into_iter().collect();
    assert_eq!(seen.len(), 2);
    assert!(seen.contains("acc-a"));
    assert!(seen.contains("acc-b"));

    std::thread::sleep(Duration::from_millis(20));
    clear_pending_usage_refresh_tasks_for_tests();
}

/// Token 刷新成功后的恢复验证直接进入单账号队列，并绕过轮询冷却；同账号并发仍去重。
#[test]
fn token_refresh_success_validation_bypasses_stale_cooldown_and_deduplicates() {
    let _guard = crate::test_env_guard();
    clear_pending_usage_refresh_tasks_for_tests();
    let storage = Storage::open_in_memory().expect("open in memory");
    storage.init().expect("init");
    let now = now_ts();
    let account_id = "acc-token-recovery-queue";
    insert_account_and_token(&storage, account_id, now);
    storage
        .update_account_status_if_changed(account_id, "unavailable")
        .expect("mark unavailable");
    storage
        .insert_event(&Event {
            account_id: Some(account_id.to_string()),
            event_type: "account_status_update".to_string(),
            message: "status=unavailable reason=usage_http_401".to_string(),
            created_at: now,
        })
        .expect("insert status reason");
    storage
        .insert_event(&Event {
            account_id: Some(account_id.to_string()),
            event_type: "usage_refresh_failed".to_string(),
            message: "usage endpoint failed: status=401".to_string(),
            created_at: now,
        })
        .expect("insert stale cooldown event");

    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let first =
        enqueue_usage_validation_after_token_refresh_with(&storage, account_id, move |id| {
            enqueue_usage_refresh_with_worker(id, move |_| {
                let _ = started_tx.send(());
                let _ = release_rx.recv_timeout(Duration::from_secs(1));
            })
        });
    assert!(
        first,
        "陈旧 usage_refresh_failed 冷却不得阻塞 Token 成功后的验证"
    );
    started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("validation worker started");

    let duplicate = enqueue_usage_validation_after_token_refresh_with(&storage, account_id, |id| {
        enqueue_usage_refresh_with_worker(id, |_| {})
    });
    assert!(
        !duplicate,
        "同账号恢复验证必须复用队列去重，避免递归并发刷新"
    );

    let _ = release_tx.send(());
    std::thread::sleep(Duration::from_millis(20));
    clear_pending_usage_refresh_tasks_for_tests();
}

/// 后台恢复验证只处理认证/用量鉴权类原因，且不会覆盖并发发生的禁用或封禁。
#[test]
fn token_refresh_success_validation_respects_reason_and_current_status() {
    for (account_id, status, reason, expected) in [
        (
            "acc-refresh-invalid",
            "unavailable",
            "refresh_token_invalid:refresh_token_unknown_401",
            true,
        ),
        (
            "acc-region-blocked",
            "unavailable",
            "refresh_token_region_blocked",
            true,
        ),
        ("acc-usage-403", "unavailable", "usage_http_403", true),
        ("acc-active", "active", "usage_ok", false),
        (
            "acc-disabled-race",
            "disabled",
            "refresh_token_invalid:refresh_token_unknown_401",
            false,
        ),
        ("acc-banned-race", "banned", "usage_http_401", false),
    ] {
        let storage = Storage::open_in_memory().expect("open in memory");
        storage.init().expect("init");
        let now = now_ts();
        insert_account_and_token(&storage, account_id, now);
        storage
            .update_account_status_if_changed(account_id, status)
            .expect("update status");
        storage
            .insert_event(&Event {
                account_id: Some(account_id.to_string()),
                event_type: "account_status_update".to_string(),
                message: format!("status={status} reason={reason}"),
                created_at: now,
            })
            .expect("insert status reason");

        let called = std::cell::Cell::new(false);
        let queued =
            enqueue_usage_validation_after_token_refresh_with(&storage, account_id, |_| {
                called.set(true);
                true
            });
        assert_eq!(queued, expected, "account_id={account_id}");
        assert_eq!(called.get(), expected, "account_id={account_id}");
    }
}

/// 函数 `schedule_prefers_exp_minus_ahead`
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
fn schedule_prefers_exp_minus_ahead() {
    let now = now_ts();
    let token = Token {
        account_id: "acc-1".to_string(),
        id_token: "id".to_string(),
        access_token: "a.eyJleHAiOjQxMDI0NDQ4MDB9.s".to_string(),
        refresh_token: "refresh".to_string(),
        api_key_access_token: None,
        last_refresh: now - 10,
    };
    let (exp, scheduled_at) = token_refresh_schedule(&token, now, 3600, 2700);
    assert_eq!(exp, Some(4_102_444_800));
    assert_eq!(scheduled_at, 4_102_441_200);
}

#[test]
fn schedule_prefers_refresh_token_exp_when_it_expires_first() {
    let now = now_ts();
    let token = Token {
        account_id: "acc-refresh-exp-first".to_string(),
        id_token: "id".to_string(),
        access_token: "a.eyJleHAiOjQxMDI0NDQ4MDB9.s".to_string(),
        refresh_token: "r.eyJleHAiOjQxMDI0NDMwMDB9.s".to_string(),
        api_key_access_token: None,
        last_refresh: now - 10,
    };
    let (exp, scheduled_at) = token_refresh_schedule(&token, now, 3600, 2700);
    assert_eq!(exp, Some(4_102_444_800));
    assert_eq!(scheduled_at, 4_102_439_400);
}

/// 函数 `schedule_falls_back_to_last_refresh_when_exp_missing`
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
fn schedule_falls_back_to_last_refresh_when_exp_missing() {
    let now = now_ts();
    let token = Token {
        account_id: "acc-2".to_string(),
        id_token: "id".to_string(),
        access_token: "no-jwt".to_string(),
        refresh_token: "refresh".to_string(),
        api_key_access_token: None,
        last_refresh: now - 5000,
    };
    let (exp, scheduled_at) = token_refresh_schedule(&token, now, 300, 2700);
    assert_eq!(exp, None);
    assert_eq!(scheduled_at, now);
}

/// 函数 `schedule_skips_when_refresh_token_is_empty`
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
fn schedule_skips_when_refresh_token_is_empty() {
    let now = now_ts();
    let token = Token {
        account_id: "acc-empty-refresh".to_string(),
        id_token: "id".to_string(),
        access_token: "a.eyJleHAiOjQxMDI0NDQ4MDB9.s".to_string(),
        refresh_token: String::new(),
        api_key_access_token: None,
        last_refresh: now - 10,
    };
    let (exp, scheduled_at) = token_refresh_schedule(&token, now, 600, 2700);
    assert_eq!(exp, None);
    assert_eq!(scheduled_at, i64::MAX);
}

/// 函数 `usage_refresh_retry_skips_when_refresh_token_is_empty`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-12
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn usage_refresh_retry_skips_when_refresh_token_is_empty() {
    let token = Token {
        account_id: "acc-empty-refresh".to_string(),
        id_token: "id".to_string(),
        access_token: "access".to_string(),
        refresh_token: String::new(),
        api_key_access_token: None,
        last_refresh: now_ts(),
    };

    assert!(!should_retry_usage_refresh_with_token(
        &token,
        "usage endpoint status 401 Unauthorized"
    ));
    assert!(!should_retry_usage_refresh_with_token(
        &token,
        "usage endpoint status 403 Forbidden"
    ));
}

#[test]
fn usage_refresh_retry_skips_region_blocked_errors() {
    let token = Token {
        account_id: "acc-region-blocked-retry".to_string(),
        id_token: "id".to_string(),
        access_token: "access".to_string(),
        refresh_token: "refresh".to_string(),
        api_key_access_token: None,
        last_refresh: now_ts(),
    };

    assert!(!should_retry_usage_refresh_with_token(
        &token,
        "usage endpoint failed: status=403 Forbidden body=code=unsupported_country_region_territory cf_ray=ray-HKG",
    ));
    assert!(should_retry_usage_refresh_with_token(
        &token,
        "usage endpoint status 403 Forbidden"
    ));
}

/// 函数 `due_cutoff_includes_next_poll_window_and_buffer`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-06
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn due_cutoff_includes_next_poll_window_and_buffer() {
    let now = now_ts();
    assert_eq!(token_refresh_due_cutoff(now, 600), now + 660);
}

/// 函数 `access_exp_cutoff_includes_refresh_ahead_window`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-26
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn access_exp_cutoff_includes_refresh_ahead_window() {
    assert_eq!(token_refresh_access_exp_cutoff(1_000, 3600), 4_600);
}

#[test]
fn default_token_refresh_failure_cooldown_covers_usage_poll_interval() {
    let _guard = crate::test_env_guard();
    let previous = std::env::var("CODEXMANAGER_TOKEN_REFRESH_FAILURE_COOLDOWN_SECS").ok();
    std::env::remove_var("CODEXMANAGER_TOKEN_REFRESH_FAILURE_COOLDOWN_SECS");

    assert!(
        token_refresh_failure_cooldown_secs() >= DEFAULT_USAGE_POLL_INTERVAL_SECS,
        "默认 token refresh 失败冷却必须避免坏 token 每分钟反复进入轮询"
    );

    if let Some(previous) = previous {
        std::env::set_var("CODEXMANAGER_TOKEN_REFRESH_FAILURE_COOLDOWN_SECS", previous);
    }
}

/// 函数 `transient_backoff_follows_exponential_sequence`
///
/// 中文注释：验证临时类失败 per-account 指数退避数列（默认 base=60s、封顶 1800s）：
/// n=1→60、n=2→120、n=3→240、n=4→480、n=5→960、n=6→1800（首次封顶）、n>=7 维持 1800。
/// 同时校验 n<=0 按首次（base）处理。
#[test]
fn transient_backoff_follows_exponential_sequence() {
    let _guard = crate::test_env_guard();
    for key in [
        "CODEXMANAGER_TOKEN_REFRESH_TRANSIENT_BACKOFF_BASE_SECS",
        "CODEXMANAGER_TOKEN_REFRESH_TRANSIENT_BACKOFF_MAX_SECS",
    ] {
        std::env::remove_var(key);
    }

    // 默认 base=60、cap=1800：60,120,240,480,960,1800(封顶),1800...
    let expected = [
        (1_i64, 60_u64),
        (2, 120),
        (3, 240),
        (4, 480),
        (5, 960),
        (6, 1_800),
        (7, 1_800),
        (20, 1_800),
    ];
    for (failure_count, want) in expected {
        assert_eq!(
            token_refresh_transient_backoff_secs(failure_count),
            want,
            "failure_count={failure_count} 的退避应为 {want}s"
        );
    }
    // n<=0 视为首次，取 base。
    assert_eq!(token_refresh_transient_backoff_secs(0), 60);
    assert_eq!(token_refresh_transient_backoff_secs(-5), 60);
}

/// 函数 `schedule_failure_retry_transient_uses_exponential_backoff`
///
/// 中文注释：临时失败（网络错误，分类器返回 None）应递增连续失败计数并按指数退避写
/// `next_refresh_at`，而非一刀切 6 小时。连续两次失败后计数应为 2，退避应为 120s。
#[test]
fn schedule_failure_retry_transient_uses_exponential_backoff() {
    let _guard = crate::test_env_guard();
    for key in [
        "CODEXMANAGER_TOKEN_REFRESH_TRANSIENT_BACKOFF_BASE_SECS",
        "CODEXMANAGER_TOKEN_REFRESH_TRANSIENT_BACKOFF_MAX_SECS",
    ] {
        std::env::remove_var(key);
    }
    let storage = Storage::open_in_memory().expect("open in memory");
    storage.init().expect("init");
    let now = now_ts();
    let account_id = "acc-transient-backoff";
    insert_account_and_token(&storage, account_id, now);

    // 网络类错误：分类器返回 None，视为临时失败。
    let transient_err = "refresh token network error: connection reset by peer";

    schedule_token_refresh_failure_retry(&storage, account_id, now, transient_err);
    assert_eq!(
        storage
            .token_consecutive_failure_count(account_id)
            .expect("read count"),
        1
    );
    assert_eq!(read_next_refresh_at(&storage, account_id), Some(now + 60));

    schedule_token_refresh_failure_retry(&storage, account_id, now, transient_err);
    assert_eq!(
        storage
            .token_consecutive_failure_count(account_id)
            .expect("read count"),
        2
    );
    assert_eq!(read_next_refresh_at(&storage, account_id), Some(now + 120));
}

/// 函数 `schedule_failure_retry_permanent_uses_long_cooldown`
///
/// 中文注释：永久无效失败（reused 等）应施加长冷却（默认 6 小时），且不依赖失败计数、
/// 反而把计数清零；与临时失败的短退避形成明确分流。
#[test]
fn schedule_failure_retry_permanent_uses_long_cooldown() {
    let _guard = crate::test_env_guard();
    std::env::remove_var("CODEXMANAGER_TOKEN_REFRESH_FAILURE_COOLDOWN_SECS");
    let storage = Storage::open_in_memory().expect("open in memory");
    storage.init().expect("init");
    let now = now_ts();
    let account_id = "acc-permanent-cooldown";
    insert_account_and_token(&storage, account_id, now);

    // 先制造一次临时失败，确认计数被永久失败清零。
    schedule_token_refresh_failure_retry(
        &storage,
        account_id,
        now,
        "refresh token network error: timeout",
    );
    assert_eq!(
        storage
            .token_consecutive_failure_count(account_id)
            .expect("read count"),
        1
    );

    // 永久无效：refresh token reused。使用生产路径真实错误串格式
    // （format_refresh_token_status_error 会前缀 "refresh token failed with status 401: "），
    // 否则消息分类器无法识别为 401 类，会误判为临时失败。
    let permanent_err = "refresh token failed with status 401: Your access token could not be refreshed because your refresh token was already used. Please log out and sign in again.";
    schedule_token_refresh_failure_retry(&storage, account_id, now, permanent_err);

    let cooldown = token_refresh_failure_cooldown_secs() as i64;
    assert_eq!(
        read_next_refresh_at(&storage, account_id),
        Some(now + cooldown),
        "永久失败应施加长冷却"
    );
    assert_eq!(
        storage
            .token_consecutive_failure_count(account_id)
            .expect("read count"),
        0,
        "永久失败应清零连续失败计数"
    );
}

/// refresh_token_expired 可能误判：使用长冷却抑制请求，但不应永久退出候选。
#[test]
fn schedule_failure_retry_expired_uses_long_recoverable_cooldown() {
    let _guard = crate::test_env_guard();
    std::env::remove_var("CODEXMANAGER_TOKEN_REFRESH_FAILURE_COOLDOWN_SECS");
    let storage = Storage::open_in_memory().expect("open in memory");
    storage.init().expect("init");
    let now = now_ts();
    let account_id = "acc-expired-recheck";
    insert_account_and_token(&storage, account_id, now);

    schedule_token_refresh_failure_retry(
        &storage,
        account_id,
        now,
        "refresh token failed with status 401 Unauthorized: Your access token could not be refreshed because your refresh token has expired. Please log out and sign in again.",
    );

    assert_eq!(
        read_next_refresh_at(&storage, account_id),
        Some(now + token_refresh_failure_cooldown_secs() as i64)
    );
    assert_eq!(
        storage
            .token_consecutive_failure_count(account_id)
            .expect("read count"),
        0
    );
}

/// 函数 `insert_account_and_token`
///
/// 中文注释：测试辅助——插入账号与对应 token，便于失败退避用例复用。
fn insert_account_and_token(storage: &Storage, account_id: &str, now: i64) {
    storage
        .insert_account(&Account {
            id: account_id.to_string(),
            label: account_id.to_string(),
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
            account_id: account_id.to_string(),
            id_token: "id".to_string(),
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
            api_key_access_token: None,
            last_refresh: now,
        })
        .expect("insert token");
}

/// 函数 `read_next_refresh_at`
///
/// 中文注释：测试辅助——读取 tokens.next_refresh_at，用于断言退避写入结果。
fn read_next_refresh_at(storage: &Storage, account_id: &str) -> Option<i64> {
    storage
        .token_next_refresh_at(account_id)
        .expect("read next_refresh_at")
}

/// 函数 `due_cutoff_covers_boundary_when_poll_interval_matches_refresh_ahead`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-06
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn due_cutoff_covers_boundary_when_poll_interval_matches_refresh_ahead() {
    let exp = 4_102_444_800;
    let now = exp - 7_260;
    let token = Token {
        account_id: "acc-boundary".to_string(),
        id_token: "id".to_string(),
        access_token: "a.eyJleHAiOjQxMDI0NDQ4MDB9.s".to_string(),
        refresh_token: "refresh".to_string(),
        api_key_access_token: None,
        last_refresh: now - 10,
    };
    let (_, scheduled_at) = token_refresh_schedule(&token, now, 3600, 2700);

    assert_eq!(scheduled_at, exp - 3600);
    assert!(scheduled_at > now);
    assert!(scheduled_at <= token_refresh_due_cutoff(now, 3600));
}

/// 函数 `token_refresh_issuer_uses_account_issuer`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-26
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn token_refresh_issuer_uses_account_issuer() {
    let now = now_ts();
    let account = Account {
        id: "acc-custom-issuer".to_string(),
        label: "custom issuer".to_string(),
        issuer: "https://custom-issuer.example".to_string(),
        chatgpt_account_id: None,
        workspace_id: None,
        group_name: None,
        sort: 0,
        status: "active".to_string(),
        created_at: now,
        updated_at: now,
    };

    assert_eq!(
        resolve_token_refresh_issuer(Some(&account), "https://auth.openai.com"),
        "https://custom-issuer.example"
    );
}

/// 函数 `token_refresh_issuer_falls_back_to_default`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-26
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn token_refresh_issuer_falls_back_to_default() {
    let now = now_ts();
    let account = Account {
        id: "acc-empty-issuer".to_string(),
        label: "empty issuer".to_string(),
        issuer: "  ".to_string(),
        chatgpt_account_id: None,
        workspace_id: None,
        group_name: None,
        sort: 0,
        status: "active".to_string(),
        created_at: now,
        updated_at: now,
    };

    assert_eq!(
        resolve_token_refresh_issuer(Some(&account), "https://auth.openai.com"),
        "https://auth.openai.com"
    );
    assert_eq!(
        resolve_token_refresh_issuer(None, "https://auth.openai.com"),
        "https://auth.openai.com"
    );
}

/// 函数 `run_token_refresh_task_skips_empty_refresh_token`
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
fn run_token_refresh_task_skips_empty_refresh_token() {
    let storage = Storage::open_in_memory().expect("open in memory");
    storage.init().expect("init");
    let now = now_ts();
    let mut token = Token {
        account_id: "acc-empty-refresh".to_string(),
        id_token: "id".to_string(),
        access_token: "access".to_string(),
        refresh_token: String::new(),
        api_key_access_token: None,
        last_refresh: now,
    };

    let refreshed =
        run_token_refresh_task(&storage, &mut token, "https://auth.openai.com", "codex-cli");
    assert!(!refreshed);
}

/// 函数 `usage_poll_batch_indices_rotate_from_cursor`
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
fn usage_poll_batch_indices_rotate_from_cursor() {
    reset_usage_poll_cursor_for_tests();
    assert_eq!(usage_poll_batch_indices(5, 4, 3), vec![4, 0, 1]);
    assert_eq!(usage_poll_batch_indices(3, 1, 10), vec![1, 2, 0]);
}

/// 函数 `usage_poll_cursor_advances_by_processed_count`
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
fn usage_poll_cursor_advances_by_processed_count() {
    reset_usage_poll_cursor_for_tests();
    assert_eq!(next_usage_poll_cursor(5, 4, 2), 1);
    assert_eq!(next_usage_poll_cursor(5, 1, 5), 1);
    assert_eq!(next_usage_poll_cursor(0, 7, 3), 0);
}
