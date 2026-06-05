use codexmanager_core::rpc::types::UsageSnapshotResult;

use crate::storage_helpers::open_storage;
use crate::usage_read::usage_snapshot_result_from_record;

const MAX_USAGE_LIST_ACCOUNT_IDS: usize = 500;
const DEFAULT_USAGE_LIST_LIMIT: i64 = 100;
const MAX_USAGE_LIST_LIMIT: i64 = 500;

/// 函数 `read_usage_snapshots`
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
pub(crate) fn read_usage_snapshots(
    account_ids: Option<Vec<String>>,
    limit: Option<i64>,
) -> Result<Vec<UsageSnapshotResult>, String> {
    let storage = open_storage().ok_or_else(|| "open storage failed".to_string())?;
    let items = match account_ids {
        None => {
            // 无账号范围时只保留兼容性限载，避免裸 RPC 搬运全库快照。
            storage
                .latest_usage_snapshots_by_account_limited(normalize_usage_list_limit(limit))
                .map_err(|err| format!("list usage snapshots failed: {err}"))?
        }
        Some(account_ids) => {
            let account_ids = normalize_account_ids(account_ids);
            if account_ids.is_empty() {
                Vec::new()
            } else {
                storage
                    .latest_usage_snapshots_by_account_ids(&account_ids)
                    .map_err(|err| format!("list usage snapshots failed: {err}"))?
            }
        }
    };
    Ok(items
        .into_iter()
        .map(usage_snapshot_result_from_record)
        .collect())
}

fn normalize_account_ids(account_ids: Vec<String>) -> Vec<String> {
    let mut ids = account_ids
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    ids.truncate(MAX_USAGE_LIST_ACCOUNT_IDS);
    ids
}

fn normalize_usage_list_limit(limit: Option<i64>) -> i64 {
    limit
        .unwrap_or(DEFAULT_USAGE_LIST_LIMIT)
        .clamp(0, MAX_USAGE_LIST_LIMIT)
}

#[cfg(test)]
mod tests {
    use super::normalize_usage_list_limit;

    #[test]
    fn normalize_usage_list_limit_clamps_unscoped_reads() {
        assert_eq!(normalize_usage_list_limit(None), 100);
        assert_eq!(normalize_usage_list_limit(Some(-1)), 0);
        assert_eq!(normalize_usage_list_limit(Some(20)), 20);
        assert_eq!(normalize_usage_list_limit(Some(10_000)), 500);
    }
}
