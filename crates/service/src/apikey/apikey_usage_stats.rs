use chrono::{Duration, Local, LocalResult, TimeZone};
use codexmanager_core::{
    rpc::types::ApiKeyUsageStatSummary,
    storage::{ApiKeyModelTokenUsageSummary, ApiKeyTokenUsageSummary},
};
use std::collections::{HashMap, HashSet};

use crate::storage_helpers::open_storage;
use crate::RpcActor;

const DAY_SECONDS: i64 = 86_400;
const MAX_API_KEY_USAGE_STAT_IDS: usize = 500;

/// 函数 `read_api_key_usage_stats`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - key_ids: 限定统计的平台密钥 ID 列表
///
/// # 返回
/// 返回函数执行结果
pub(crate) fn read_api_key_usage_stats(
    key_ids: Option<Vec<String>>,
) -> Result<Vec<ApiKeyUsageStatSummary>, String> {
    let storage = open_storage().ok_or_else(|| "open storage failed".to_string())?;
    let (today_start, today_end) = local_day_bounds_ts()?;
    let (total_items, today_items) = match key_ids {
        Some(key_ids) => {
            let key_ids = normalize_key_ids(key_ids);
            if key_ids.is_empty() {
                return Ok(Vec::new());
            }
            let total_items = storage
                .summarize_request_token_stats_by_key_ids(&key_ids)
                .map_err(|err| format!("summarize api key token stats failed: {err}"))?;
            let today_items = storage
                .summarize_request_token_stats_by_key_ids_and_model(
                    &key_ids,
                    Some(today_start),
                    Some(today_end),
                )
                .map_err(|err| format!("summarize api key today token stats failed: {err}"))?;
            (total_items, today_items)
        }
        None => {
            let total_items = storage
                .summarize_request_token_stats_by_key()
                .map_err(|err| format!("summarize api key token stats failed: {err}"))?;
            let today_items = storage
                .summarize_request_token_stats_by_key_and_model(Some(today_start), Some(today_end))
                .map_err(|err| format!("summarize api key today token stats failed: {err}"))?;
            (total_items, today_items)
        }
    };

    Ok(map_api_key_usage_stats(total_items, today_items))
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

fn map_api_key_usage_stats(
    total_items: Vec<ApiKeyTokenUsageSummary>,
    today_items: Vec<ApiKeyModelTokenUsageSummary>,
) -> Vec<ApiKeyUsageStatSummary> {
    let total_by_key = merge_total_usage_by_key(total_items);
    let today_by_key = merge_model_usage_by_key(today_items);
    let mut key_ids = total_by_key
        .keys()
        .chain(today_by_key.keys())
        .cloned()
        .collect::<Vec<_>>();
    key_ids.sort();
    key_ids.dedup();

    let mut items = key_ids
        .into_iter()
        .map(|key_id| {
            let (total_tokens, estimated_cost_usd) =
                total_by_key.get(&key_id).copied().unwrap_or((0, 0.0));
            let (today_tokens, today_estimated_cost_usd) =
                today_by_key.get(&key_id).copied().unwrap_or((0, 0.0));
            ApiKeyUsageStatSummary {
                key_id,
                today_tokens: today_tokens.max(0),
                today_estimated_cost_usd: today_estimated_cost_usd.max(0.0),
                total_tokens: total_tokens.max(0),
                estimated_cost_usd: estimated_cost_usd.max(0.0),
            }
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        right
            .total_tokens
            .cmp(&left.total_tokens)
            .then_with(|| left.key_id.cmp(&right.key_id))
    });
    items
}

fn merge_total_usage_by_key(items: Vec<ApiKeyTokenUsageSummary>) -> HashMap<String, (i64, f64)> {
    let mut by_key = HashMap::new();
    for item in items {
        merge_usage_entry(
            &mut by_key,
            item.key_id,
            item.total_tokens,
            item.estimated_cost_usd,
        );
    }
    by_key
}

fn merge_model_usage_by_key(
    items: Vec<ApiKeyModelTokenUsageSummary>,
) -> HashMap<String, (i64, f64)> {
    let mut by_key = HashMap::new();
    for item in items {
        merge_usage_entry(
            &mut by_key,
            item.key_id,
            item.total_tokens,
            item.estimated_cost_usd,
        );
    }
    by_key
}

fn merge_usage_entry(
    by_key: &mut HashMap<String, (i64, f64)>,
    key_id: String,
    total_tokens: i64,
    estimated_cost_usd: f64,
) {
    let key_id = key_id.trim().to_string();
    if key_id.is_empty() {
        return;
    }
    let entry = by_key.entry(key_id).or_insert((0, 0.0));
    entry.0 = entry.0.saturating_add(total_tokens.max(0));
    entry.1 += estimated_cost_usd.max(0.0);
}

fn local_day_bounds_ts() -> Result<(i64, i64), String> {
    let now = Local::now();
    let today = now.date_naive();
    let start_naive = today
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| "build local start-of-day failed".to_string())?;
    let tomorrow_naive = (today + Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| "build local end-of-day failed".to_string())?;
    let start = match Local.from_local_datetime(&start_naive) {
        LocalResult::Single(value) => value.timestamp(),
        LocalResult::Ambiguous(a, b) => a.timestamp().min(b.timestamp()),
        LocalResult::None => now.timestamp(),
    };
    let end = match Local.from_local_datetime(&tomorrow_naive) {
        LocalResult::Single(value) => value.timestamp(),
        LocalResult::Ambiguous(a, b) => a.timestamp().max(b.timestamp()),
        LocalResult::None => start + DAY_SECONDS,
    };
    Ok((start, end.max(start)))
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
