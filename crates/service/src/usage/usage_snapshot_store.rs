use crate::account_availability::{evaluate_snapshot, Availability};
use crate::account_status::{is_refresh_blocked_status_reason, set_account_status};
use codexmanager_core::storage::{now_ts, Storage, UsageSnapshotRecord};
use codexmanager_core::usage::parse_usage_snapshot;
use serde_json::Value;

const DEFAULT_USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT: usize = 1;
const USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT_ENV: &str =
    "CODEXMANAGER_USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT";

fn usage_status_updates_blocked(storage: &Storage, account_id: &str, current_status: &str) -> bool {
    if current_status.trim().eq_ignore_ascii_case("disabled") {
        return true;
    }
    storage
        .latest_account_status_reasons(&[account_id.to_string()])
        .ok()
        .and_then(|mut reasons| reasons.remove(account_id))
        .as_deref()
        .is_some_and(is_refresh_blocked_status_reason)
}

/// 函数 `usage_snapshots_retain_per_account`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// 无
///
/// # 返回
/// 返回函数执行结果
fn usage_snapshots_retain_per_account() -> usize {
    std::env::var(USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT_ENV)
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .unwrap_or(DEFAULT_USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT)
}

fn credits_json_semantically_equal(left: Option<&str>, right: Option<&str>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => match (
            serde_json::from_str::<Value>(left),
            serde_json::from_str::<Value>(right),
        ) {
            (Ok(left), Ok(right)) => left == right,
            _ => left == right,
        },
        _ => false,
    }
}

fn usage_snapshot_key_fields_equal(
    latest: &UsageSnapshotRecord,
    current: &UsageSnapshotRecord,
) -> bool {
    latest.used_percent == current.used_percent
        && latest.window_minutes == current.window_minutes
        && latest.resets_at == current.resets_at
        && latest.secondary_used_percent == current.secondary_used_percent
        && latest.secondary_window_minutes == current.secondary_window_minutes
        && latest.secondary_resets_at == current.secondary_resets_at
        && credits_json_semantically_equal(
            latest.credits_json.as_deref(),
            current.credits_json.as_deref(),
        )
}

fn persist_usage_snapshot(storage: &Storage, record: &UsageSnapshotRecord) -> Result<(), String> {
    let latest = storage
        .latest_usage_snapshot_for_account(&record.account_id)
        .map_err(|err| err.to_string())?;
    if latest
        .as_ref()
        .is_some_and(|latest| usage_snapshot_key_fields_equal(latest, record))
    {
        let updated = storage
            .update_latest_usage_snapshot_captured_at_for_account(
                &record.account_id,
                record.captured_at,
            )
            .map_err(|err| err.to_string())?;
        if updated > 0 {
            return Ok(());
        }
    }

    storage
        .insert_usage_snapshot(record)
        .map_err(|e| e.to_string())?;
    let retain = usage_snapshots_retain_per_account();
    if retain > 0 {
        let _ = storage.prune_usage_snapshots_for_account(&record.account_id, retain);
    }
    Ok(())
}

/// 函数 `apply_status_from_snapshot`
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
pub(crate) fn apply_status_from_snapshot(
    storage: &Storage,
    record: &UsageSnapshotRecord,
) -> Availability {
    let availability = evaluate_snapshot(record);
    let current_status = storage
        .find_account_by_id(&record.account_id)
        .ok()
        .flatten()
        .map(|account| account.status)
        .unwrap_or_default();

    if usage_status_updates_blocked(storage, &record.account_id, &current_status) {
        return availability;
    }

    match availability {
        Availability::Available => {
            set_account_status(storage, &record.account_id, "active", "usage_ok");
        }
        Availability::Unavailable("usage_exhausted_primary" | "usage_exhausted_secondary") => {
            set_account_status(
                storage,
                &record.account_id,
                "limited",
                "usage_limit_exhausted",
            );
        }
        Availability::Unavailable(_) => {}
    }
    availability
}

/// 函数 `store_usage_snapshot`
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
pub(crate) fn store_usage_snapshot(
    storage: &Storage,
    account_id: &str,
    value: serde_json::Value,
) -> Result<(), String> {
    // 解析并写入用量快照
    let parsed = parse_usage_snapshot(&value);
    let record = UsageSnapshotRecord {
        account_id: account_id.to_string(),
        used_percent: parsed.used_percent,
        window_minutes: parsed.window_minutes,
        resets_at: parsed.resets_at,
        secondary_used_percent: parsed.secondary_used_percent,
        secondary_window_minutes: parsed.secondary_window_minutes,
        secondary_resets_at: parsed.secondary_resets_at,
        credits_json: parsed.credits_json,
        captured_at: now_ts(),
    };
    persist_usage_snapshot(storage, &record)?;
    let _ = apply_status_from_snapshot(storage, &record);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use codexmanager_core::storage::Account;
    use serde_json::json;
    use std::thread;
    use std::time::Duration;

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(previous) = &self.previous {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    fn usage_payload(used_percent: f64, credits: Value) -> Value {
        json!({
            "rate_limit": {
                "primary_window": {
                    "used_percent": used_percent,
                    "limit_window_seconds": 18_000,
                    "reset_at": 1_800_000_000
                },
                "secondary_window": {
                    "used_percent": 40.0,
                    "limit_window_seconds": 604_800,
                    "reset_at": 1_800_604_800
                }
            },
            "credits": credits
        })
    }

    fn insert_account(storage: &Storage, account_id: &str) {
        let now = now_ts();
        storage
            .insert_account(&Account {
                id: account_id.to_string(),
                label: account_id.to_string(),
                issuer: "test".to_string(),
                chatgpt_account_id: None,
                workspace_id: None,
                group_name: None,
                sort: 0,
                status: "limited".to_string(),
                created_at: now,
                updated_at: now,
            })
            .expect("insert account");
    }

    fn open_storage_with_account(account_id: &str) -> Storage {
        let storage = Storage::open_in_memory().expect("open storage");
        storage.init().expect("init storage");
        insert_account(&storage, account_id);
        storage
    }

    fn wait_until_next_captured_at(previous: i64) {
        while now_ts() <= previous {
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn store_usage_snapshot_skips_insert_for_unchanged_key_fields() {
        let _env_lock = crate::test_env_guard();
        let _retain = EnvVarGuard::set(USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT_ENV, "10");
        let storage = open_storage_with_account("acc-same");
        let payload = usage_payload(12.5, json!({"grant": "alpha", "remaining": 8}));

        store_usage_snapshot(&storage, "acc-same", payload.clone()).expect("store first snapshot");
        store_usage_snapshot(&storage, "acc-same", payload).expect("store unchanged snapshot");

        assert_eq!(
            storage
                .usage_snapshot_count_for_account("acc-same")
                .expect("count snapshots"),
            1
        );
    }

    #[test]
    fn store_usage_snapshot_refreshes_latest_captured_at_for_unchanged_key_fields() {
        let _env_lock = crate::test_env_guard();
        let _retain = EnvVarGuard::set(USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT_ENV, "10");
        let storage = open_storage_with_account("acc-captured");
        let payload = usage_payload(12.5, json!({"grant": "alpha", "remaining": 8}));

        store_usage_snapshot(&storage, "acc-captured", payload.clone()).expect("store first");
        let first = storage
            .latest_usage_snapshot_for_account("acc-captured")
            .expect("read first latest")
            .expect("first latest exists");
        wait_until_next_captured_at(first.captured_at);

        store_usage_snapshot(&storage, "acc-captured", payload).expect("store unchanged");
        let second = storage
            .latest_usage_snapshot_for_account("acc-captured")
            .expect("read second latest")
            .expect("second latest exists");

        assert_eq!(
            storage
                .usage_snapshot_count_for_account("acc-captured")
                .expect("count snapshots"),
            1
        );
        assert!(second.captured_at > first.captured_at);
        assert_eq!(second.used_percent, first.used_percent);
    }

    #[test]
    fn store_usage_snapshot_inserts_when_key_field_changes() {
        let _env_lock = crate::test_env_guard();
        let _retain = EnvVarGuard::set(USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT_ENV, "10");
        let storage = open_storage_with_account("acc-change");
        let first_payload = usage_payload(12.5, json!({"grant": "alpha", "remaining": 8}));
        let changed_payload = usage_payload(20.0, json!({"grant": "alpha", "remaining": 8}));

        store_usage_snapshot(&storage, "acc-change", first_payload).expect("store first");
        let first = storage
            .latest_usage_snapshot_for_account("acc-change")
            .expect("read first latest")
            .expect("first latest exists");
        wait_until_next_captured_at(first.captured_at);
        store_usage_snapshot(&storage, "acc-change", changed_payload).expect("store changed");

        assert_eq!(
            storage
                .usage_snapshot_count_for_account("acc-change")
                .expect("count snapshots"),
            2
        );
        let latest = storage
            .latest_usage_snapshot_for_account("acc-change")
            .expect("read latest")
            .expect("latest exists");
        assert_eq!(latest.used_percent, Some(20.0));
    }

    #[test]
    fn store_usage_snapshot_treats_reordered_credits_json_as_unchanged() {
        let _env_lock = crate::test_env_guard();
        let _retain = EnvVarGuard::set(USAGE_SNAPSHOTS_RETAIN_PER_ACCOUNT_ENV, "10");
        let storage = open_storage_with_account("acc-credits");
        let first_payload = usage_payload(
            12.5,
            json!({
                "grant": "alpha",
                "remaining": 8,
                "nested": {
                    "b": 2,
                    "a": 1
                }
            }),
        );
        let reordered_payload = usage_payload(
            12.5,
            json!({
                "nested": {
                    "a": 1,
                    "b": 2
                },
                "remaining": 8,
                "grant": "alpha"
            }),
        );

        store_usage_snapshot(&storage, "acc-credits", first_payload).expect("store first");
        store_usage_snapshot(&storage, "acc-credits", reordered_payload).expect("store reordered");

        assert_eq!(
            storage
                .usage_snapshot_count_for_account("acc-credits")
                .expect("count snapshots"),
            1
        );
    }
}
