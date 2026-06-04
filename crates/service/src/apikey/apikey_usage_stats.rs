use codexmanager_core::rpc::types::ApiKeyUsageStatSummary;
use std::collections::HashSet;

use crate::storage_helpers::open_storage;
use crate::RpcActor;

const MAX_API_KEY_USAGE_STAT_IDS: usize = 500;

/// 函数 `read_api_key_usage_stats`
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
pub(crate) fn read_api_key_usage_stats(
    key_ids: Option<Vec<String>>,
) -> Result<Vec<ApiKeyUsageStatSummary>, String> {
    let storage = open_storage().ok_or_else(|| "open storage failed".to_string())?;
    let items = match key_ids {
        Some(key_ids) => {
            let key_ids = normalize_key_ids(key_ids);
            if key_ids.is_empty() {
                Vec::new()
            } else {
                storage
                    .summarize_request_token_stats_by_key_ids(&key_ids)
                    .map_err(|err| format!("summarize api key token stats failed: {err}"))?
            }
        }
        None => storage
            .summarize_request_token_stats_by_key()
            .map_err(|err| format!("summarize api key token stats failed: {err}"))?,
    };

    Ok(items
        .into_iter()
        .map(|item| ApiKeyUsageStatSummary {
            key_id: item.key_id,
            total_tokens: item.total_tokens.max(0),
            estimated_cost_usd: item.estimated_cost_usd.max(0.0),
        })
        .collect())
}

pub(crate) fn read_api_key_usage_stats_for_actor(
    actor: &RpcActor,
    key_ids: Option<Vec<String>>,
) -> Result<Vec<ApiKeyUsageStatSummary>, String> {
    if actor.is_admin() {
        return read_api_key_usage_stats(key_ids);
    }
    let user_id = actor
        .user_id
        .as_deref()
        .ok_or_else(|| "permission_denied: apikey usage requires user session".to_string())?;
    let owned_key_ids = crate::list_api_key_ids_for_user(user_id)?
        .into_iter()
        .collect::<HashSet<_>>();
    let scoped_key_ids = match key_ids {
        Some(key_ids) => normalize_key_ids(key_ids)
            .into_iter()
            .filter(|key_id| owned_key_ids.contains(key_id))
            .collect::<Vec<_>>(),
        None => owned_key_ids.into_iter().collect::<Vec<_>>(),
    };
    read_api_key_usage_stats(Some(scoped_key_ids))
}

fn normalize_key_ids(key_ids: Vec<String>) -> Vec<String> {
    let mut ids = key_ids
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    ids.truncate(MAX_API_KEY_USAGE_STAT_IDS);
    ids
}
