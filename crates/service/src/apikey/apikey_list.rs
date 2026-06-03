use codexmanager_core::rpc::types::{ApiKeyListParams, ApiKeyListResult, ApiKeySummary};

use crate::storage_helpers::open_storage;
use crate::RpcActor;

const DEFAULT_API_KEY_PAGE_SIZE: i64 = 20;
const MAX_API_KEY_PAGE_SIZE: i64 = 200;

/// 函数 `read_api_keys`
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
pub(crate) fn read_api_keys() -> Result<Vec<ApiKeySummary>, String> {
    // 读取平台 Key 列表
    let storage = open_storage().ok_or_else(|| "open storage failed".to_string())?;
    let keys = storage
        .list_api_keys()
        .map_err(|err| format!("list api keys failed: {err}"))?;
    to_api_key_summaries(&storage, keys)
}

pub(crate) fn read_api_keys_for_actor(actor: &RpcActor) -> Result<Vec<ApiKeySummary>, String> {
    let storage = open_storage().ok_or_else(|| "open storage failed".to_string())?;
    let owner_user_id = owner_user_id_for_actor(actor)?;
    let keys = storage
        .list_api_keys_filtered(None, None, owner_user_id.as_deref())
        .map_err(|err| format!("list api keys failed: {err}"))?;
    to_api_key_summaries(&storage, keys)
}

pub(crate) fn read_api_key_list_for_actor(
    actor: &RpcActor,
    params: ApiKeyListParams,
    pagination_requested: bool,
) -> Result<ApiKeyListResult, String> {
    let params = params.normalized();
    let storage = open_storage().ok_or_else(|| "open storage failed".to_string())?;
    let owner_user_id = owner_user_id_for_actor(actor)?;
    let query = normalize_optional_text(params.query);
    let status_filter = normalize_status_filter(params.status_filter);

    if pagination_requested {
        let page_size = normalize_page_size(params.page_size);
        let total = storage
            .api_key_count_filtered(
                query.as_deref(),
                status_filter.as_deref(),
                owner_user_id.as_deref(),
            )
            .map_err(|err| format!("count api keys failed: {err}"))?;
        let page = clamp_page(params.page, total, page_size);
        let offset = (page - 1) * page_size;
        let keys = storage
            .list_api_keys_paginated(
                query.as_deref(),
                status_filter.as_deref(),
                owner_user_id.as_deref(),
                offset,
                page_size,
            )
            .map_err(|err| format!("list api keys failed: {err}"))?;
        let items = to_api_key_summaries(&storage, keys)?;
        return Ok(ApiKeyListResult {
            items,
            total,
            page,
            page_size,
        });
    }

    let keys = storage
        .list_api_keys_filtered(
            query.as_deref(),
            status_filter.as_deref(),
            owner_user_id.as_deref(),
        )
        .map_err(|err| format!("list api keys failed: {err}"))?;
    let total = keys.len() as i64;
    let items = to_api_key_summaries(&storage, keys)?;
    Ok(ApiKeyListResult {
        items,
        total,
        page: 1,
        page_size: if total > 0 {
            total
        } else {
            DEFAULT_API_KEY_PAGE_SIZE
        },
    })
}

fn to_api_key_summaries(
    storage: &codexmanager_core::storage::Storage,
    keys: Vec<codexmanager_core::storage::ApiKey>,
) -> Result<Vec<ApiKeySummary>, String> {
    let key_ids = keys.iter().map(|key| key.id.clone()).collect::<Vec<_>>();
    let quota_limits = storage
        .list_api_key_quota_limits_for_key_ids(&key_ids)
        .map_err(|err| format!("list api key quota limits failed: {err}"))?;
    Ok(keys
        .into_iter()
        .map(|key| ApiKeySummary {
            quota_limit_tokens: quota_limits.get(&key.id).copied(),
            id: key.id,
            name: key.name,
            model_slug: key.model_slug,
            reasoning_effort: key.reasoning_effort,
            service_tier: key.service_tier,
            rotation_strategy: key.rotation_strategy,
            aggregate_api_id: key.aggregate_api_id,
            account_plan_filter: key.account_plan_filter,
            aggregate_api_url: key.aggregate_api_url,
            client_type: key.client_type,
            protocol_type: key.protocol_type,
            auth_scheme: key.auth_scheme,
            upstream_base_url: key.upstream_base_url,
            static_headers_json: key.static_headers_json,
            status: key.status,
            created_at: key.created_at,
            last_used_at: key.last_used_at,
        })
        .collect())
}

fn owner_user_id_for_actor(actor: &RpcActor) -> Result<Option<String>, String> {
    if actor.is_admin() {
        return Ok(None);
    }
    let user_id = actor
        .user_id
        .as_deref()
        .ok_or_else(|| "permission_denied: apikey requires user session".to_string())?;
    Ok(Some(user_id.trim().to_string()))
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    let trimmed = value.unwrap_or_default().trim().to_string();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("all") {
        return None;
    }
    Some(trimmed)
}

fn normalize_status_filter(value: Option<String>) -> Option<String> {
    match normalize_optional_text(value)
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "active" | "enabled" => Some("active".to_string()),
        "disabled" => Some("disabled".to_string()),
        _ => None,
    }
}

fn normalize_page_size(value: i64) -> i64 {
    value.clamp(1, MAX_API_KEY_PAGE_SIZE)
}

fn clamp_page(page: i64, total: i64, page_size: i64) -> i64 {
    let normalized_page = page.max(1);
    let total_pages = if total <= 0 {
        1
    } else {
        ((total + page_size - 1) / page_size).max(1)
    };
    normalized_page.min(total_pages)
}
