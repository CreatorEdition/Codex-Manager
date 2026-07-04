use codexmanager_core::{
    rpc::types::{AccountListParams, AccountListResult, AccountPlanTypeSummary, AccountSummary},
    storage::{
        Account, AccountMetadata, AccountQuotaCapacityOverride, AccountSubscription, Token,
        UsageSnapshotRecord,
    },
};
use std::{cmp::Ordering, collections::HashMap};

use crate::account_plan::resolve_effective_account_plan;
use crate::storage_helpers::open_storage;

const DEFAULT_ACCOUNT_PAGE_SIZE: i64 = 5;
const MAX_ACCOUNT_PAGE_SIZE: i64 = 500;
const MAX_ACCOUNT_LOOKUP_IDS: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccountFilter {
    All,
    Active,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccountStatusFilter {
    All,
    Active,
    Low,
    Exact(AccountStatusKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccountStatusKind {
    Banned,
    Disabled,
    Inactive,
    Limited,
    Unavailable,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccountSortMode {
    Manual,
    LargeFirst,
    SmallFirst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AccountSizeGroup {
    Large,
    Standard,
    Small,
}

/// 函数 `read_accounts`
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
pub(crate) fn read_accounts(
    params: AccountListParams,
    pagination_requested: bool,
) -> Result<AccountListResult, String> {
    // 中文注释：公共 RPC 默认走分页；只有内部调用明确传入 false 时才保留全量兼容路径。
    let params = params.normalized();
    let storage = open_storage().ok_or_else(|| "open storage failed".to_string())?;
    let query = normalize_optional_text(params.query);
    let group_filter = normalize_optional_text(params.group_filter);
    let filter = normalize_filter(params.filter);
    let status_filter = normalize_status_filter(params.status_filter);
    let effective_filter = merge_status_filter(filter, status_filter);
    let plan_filter = crate::account_plan::normalize_account_plan_filter(params.plan_filter)?;
    let sort_mode = normalize_sort_mode(params.sort_mode);
    let include_plan_types = params.include_plan_types;

    if plan_filter.is_some()
        || matches!(effective_filter, AccountStatusFilter::Exact(_))
        || sort_mode != AccountSortMode::Manual
    {
        return read_accounts_with_summary_filters(
            &storage,
            effective_filter,
            plan_filter.as_deref(),
            sort_mode,
            query.as_deref(),
            group_filter.as_deref(),
            pagination_requested,
            params.page,
            params.page_size,
            include_plan_types,
        );
    }

    let filter = match effective_filter {
        AccountStatusFilter::All => AccountFilter::All,
        AccountStatusFilter::Active => AccountFilter::Active,
        AccountStatusFilter::Low => AccountFilter::Low,
        AccountStatusFilter::Exact(_) => AccountFilter::All,
    };

    if filter == AccountFilter::All {
        if pagination_requested {
            let page_size = normalize_page_size(params.page_size);
            let total = storage
                .account_count_filtered(query.as_deref(), group_filter.as_deref())
                .map_err(|err| format!("count accounts failed: {err}"))?;
            let page = clamp_page(params.page, total, page_size);
            let offset = (page - 1) * page_size;
            let accounts = storage
                .list_accounts_paginated(
                    query.as_deref(),
                    group_filter.as_deref(),
                    offset,
                    page_size,
                )
                .map_err(|err| format!("list accounts failed: {err}"))?;
            let items = to_account_summaries(&storage, accounts)?;
            return Ok(AccountListResult {
                items,
                total,
                page,
                page_size,
                plan_types: account_plan_type_options_if_requested(
                    &storage,
                    include_plan_types,
                    effective_filter,
                    query.as_deref(),
                    group_filter.as_deref(),
                )?,
            });
        }

        let accounts = storage
            .list_accounts_filtered(query.as_deref(), group_filter.as_deref())
            .map_err(|err| format!("list accounts failed: {err}"))?;
        let total = accounts.len() as i64;
        let items = to_account_summaries(&storage, accounts)?;
        return Ok(AccountListResult {
            items,
            total,
            page: 1,
            page_size: if total > 0 {
                total
            } else {
                DEFAULT_ACCOUNT_PAGE_SIZE
            },
            plan_types: account_plan_type_options_if_requested(
                &storage,
                include_plan_types,
                effective_filter,
                query.as_deref(),
                group_filter.as_deref(),
            )?,
        });
    }

    if pagination_requested {
        let total =
            filtered_account_count(&storage, filter, query.as_deref(), group_filter.as_deref())?;
        let page_size = normalize_page_size(params.page_size);
        let page = clamp_page(params.page, total, page_size);
        let offset = (page - 1) * page_size;
        let paged = filtered_accounts(
            &storage,
            filter,
            query.as_deref(),
            group_filter.as_deref(),
            Some((offset, page_size)),
        )?;
        let items = to_account_summaries(&storage, paged)?;
        return Ok(AccountListResult {
            items,
            total,
            page,
            page_size,
            plan_types: account_plan_type_options_if_requested(
                &storage,
                include_plan_types,
                effective_filter,
                query.as_deref(),
                group_filter.as_deref(),
            )?,
        });
    }

    let accounts = filtered_accounts(
        &storage,
        filter,
        query.as_deref(),
        group_filter.as_deref(),
        None,
    )?;
    let total = accounts.len() as i64;
    let items = to_account_summaries(&storage, accounts)?;

    Ok(AccountListResult {
        items,
        total,
        page: 1,
        page_size: if total > 0 {
            total
        } else {
            DEFAULT_ACCOUNT_PAGE_SIZE
        },
        plan_types: account_plan_type_options_if_requested(
            &storage,
            include_plan_types,
            effective_filter,
            query.as_deref(),
            group_filter.as_deref(),
        )?,
    })
}

pub(crate) fn lookup_accounts(ids: Vec<String>) -> Result<Vec<AccountSummary>, String> {
    let ids = normalize_lookup_ids(ids);
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let storage = open_storage().ok_or_else(|| "open storage failed".to_string())?;
    let accounts = storage
        .list_accounts_by_ids(&ids)
        .map_err(|err| format!("lookup accounts failed: {err}"))?;
    to_account_summaries(&storage, accounts)
}

/// 函数 `normalize_optional_text`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - value: 参数 value
///
/// # 返回
/// 返回函数执行结果
fn normalize_optional_text(value: Option<String>) -> Option<String> {
    let trimmed = value.unwrap_or_default().trim().to_string();
    if trimmed.is_empty() || trimmed == "all" {
        return None;
    }
    Some(trimmed)
}

/// 函数 `normalize_filter`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - value: 参数 value
///
/// # 返回
/// 返回函数执行结果
fn normalize_filter(value: Option<String>) -> AccountFilter {
    match value
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "active" => AccountFilter::Active,
        "low" => AccountFilter::Low,
        _ => AccountFilter::All,
    }
}

fn normalize_status_filter(value: Option<String>) -> AccountStatusFilter {
    match value
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "" | "all" => AccountStatusFilter::All,
        "active" | "available" => AccountStatusFilter::Active,
        "low" | "low_quota" => AccountStatusFilter::Low,
        "banned" => AccountStatusFilter::Exact(AccountStatusKind::Banned),
        "disabled" => AccountStatusFilter::Exact(AccountStatusKind::Disabled),
        "inactive" => AccountStatusFilter::Exact(AccountStatusKind::Inactive),
        "limited" => AccountStatusFilter::Exact(AccountStatusKind::Limited),
        "unavailable" => AccountStatusFilter::Exact(AccountStatusKind::Unavailable),
        "unknown" => AccountStatusFilter::Exact(AccountStatusKind::Unknown),
        _ => AccountStatusFilter::All,
    }
}

fn merge_status_filter(
    legacy_filter: AccountFilter,
    status_filter: AccountStatusFilter,
) -> AccountStatusFilter {
    if status_filter != AccountStatusFilter::All {
        return status_filter;
    }
    match legacy_filter {
        AccountFilter::All => AccountStatusFilter::All,
        AccountFilter::Active => AccountStatusFilter::Active,
        AccountFilter::Low => AccountStatusFilter::Low,
    }
}

fn normalize_sort_mode(value: Option<String>) -> AccountSortMode {
    match value
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .replace('_', "-")
        .as_str()
    {
        "large-first" | "large" | "business-first" => AccountSortMode::LargeFirst,
        "small-first" | "small" | "free-first" => AccountSortMode::SmallFirst,
        _ => AccountSortMode::Manual,
    }
}

fn normalize_lookup_ids(ids: Vec<String>) -> Vec<String> {
    let mut normalized = ids
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized.truncate(MAX_ACCOUNT_LOOKUP_IDS);
    normalized
}

/// 函数 `normalize_page_size`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - value: 参数 value
///
/// # 返回
/// 返回函数执行结果
fn normalize_page_size(value: i64) -> i64 {
    value.clamp(1, MAX_ACCOUNT_PAGE_SIZE)
}

/// 函数 `clamp_page`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - page: 参数 page
/// - total: 参数 total
/// - page_size: 参数 page_size
///
/// # 返回
/// 返回函数执行结果
fn clamp_page(page: i64, total: i64, page_size: i64) -> i64 {
    let normalized_page = page.max(1);
    let total_pages = if total <= 0 {
        1
    } else {
        ((total + page_size - 1) / page_size).max(1)
    };
    normalized_page.min(total_pages)
}

/// 函数 `filtered_account_count`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - storage: 参数 storage
/// - filter: 参数 filter
/// - query: 参数 query
/// - group_filter: 参数 group_filter
///
/// # 返回
/// 返回函数执行结果
fn filtered_account_count(
    storage: &codexmanager_core::storage::Storage,
    filter: AccountFilter,
    query: Option<&str>,
    group_filter: Option<&str>,
) -> Result<i64, String> {
    match filter {
        AccountFilter::All => storage
            .account_count_filtered(query, group_filter)
            .map_err(|err| format!("count accounts failed: {err}")),
        AccountFilter::Active => storage
            .account_count_active_available(query, group_filter)
            .map_err(|err| format!("count active accounts failed: {err}")),
        AccountFilter::Low => storage
            .account_count_low_quota(query, group_filter)
            .map_err(|err| format!("count low quota accounts failed: {err}")),
    }
}

/// 函数 `filtered_accounts`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - storage: 参数 storage
/// - filter: 参数 filter
/// - query: 参数 query
/// - group_filter: 参数 group_filter
/// - pagination: 参数 pagination
///
/// # 返回
/// 返回函数执行结果
fn filtered_accounts(
    storage: &codexmanager_core::storage::Storage,
    filter: AccountFilter,
    query: Option<&str>,
    group_filter: Option<&str>,
    pagination: Option<(i64, i64)>,
) -> Result<Vec<Account>, String> {
    match filter {
        AccountFilter::All => match pagination {
            Some((offset, limit)) => storage
                .list_accounts_paginated(query, group_filter, offset, limit)
                .map_err(|err| format!("list accounts failed: {err}")),
            None => storage
                .list_accounts_filtered(query, group_filter)
                .map_err(|err| format!("list accounts failed: {err}")),
        },
        AccountFilter::Active => storage
            .list_accounts_active_available(query, group_filter, pagination)
            .map_err(|err| format!("list active accounts failed: {err}")),
        AccountFilter::Low => storage
            .list_accounts_low_quota(query, group_filter, pagination)
            .map_err(|err| format!("list low quota accounts failed: {err}")),
    }
}

fn read_accounts_with_summary_filters(
    storage: &codexmanager_core::storage::Storage,
    filter: AccountStatusFilter,
    plan_filter: Option<&str>,
    sort_mode: AccountSortMode,
    query: Option<&str>,
    group_filter: Option<&str>,
    pagination_requested: bool,
    requested_page: i64,
    requested_page_size: i64,
    include_plan_types: bool,
) -> Result<AccountListResult, String> {
    let broad_filter = match filter {
        AccountStatusFilter::Active => AccountFilter::Active,
        AccountStatusFilter::Low => AccountFilter::Low,
        AccountStatusFilter::All | AccountStatusFilter::Exact(_) => AccountFilter::All,
    };
    let accounts = filtered_accounts(storage, broad_filter, query, group_filter, None)?;
    let summaries = to_account_summaries(storage, accounts)?;
    let plan_types =
        account_plan_type_options_from_summaries(include_plan_types, filter, &summaries);
    let mut items = summaries
        .into_iter()
        .filter(|item| account_summary_matches_status(item, filter))
        .filter(|item| account_summary_matches_plan(item, plan_filter))
        .collect::<Vec<_>>();
    if sort_mode != AccountSortMode::Manual {
        sort_account_summaries(&mut items, sort_mode);
    }

    let total = items.len() as i64;
    if !pagination_requested {
        return Ok(AccountListResult {
            items,
            total,
            page: 1,
            page_size: if total > 0 {
                total
            } else {
                DEFAULT_ACCOUNT_PAGE_SIZE
            },
            plan_types,
        });
    }

    let page_size = normalize_page_size(requested_page_size);
    let page = clamp_page(requested_page, total, page_size);
    let start = ((page - 1) * page_size).max(0) as usize;
    let end = (start + page_size as usize).min(items.len());
    let page_items = if start < items.len() {
        items.into_iter().skip(start).take(end - start).collect()
    } else {
        Vec::new()
    };
    Ok(AccountListResult {
        items: page_items,
        total,
        page,
        page_size,
        plan_types,
    })
}

fn account_plan_type_options_if_requested(
    storage: &codexmanager_core::storage::Storage,
    include_plan_types: bool,
    filter: AccountStatusFilter,
    query: Option<&str>,
    group_filter: Option<&str>,
) -> Result<Vec<AccountPlanTypeSummary>, String> {
    if !include_plan_types {
        return Ok(Vec::new());
    }
    let broad_filter = match filter {
        AccountStatusFilter::Active => AccountFilter::Active,
        AccountStatusFilter::Low => AccountFilter::Low,
        AccountStatusFilter::All | AccountStatusFilter::Exact(_) => AccountFilter::All,
    };
    let accounts = filtered_accounts(storage, broad_filter, query, group_filter, None)?;
    let summaries = to_account_summaries(storage, accounts)?;
    Ok(account_plan_type_options_from_summaries(
        include_plan_types,
        filter,
        &summaries,
    ))
}

fn account_plan_type_options_from_summaries(
    include_plan_types: bool,
    filter: AccountStatusFilter,
    summaries: &[AccountSummary],
) -> Vec<AccountPlanTypeSummary> {
    if !include_plan_types {
        return Vec::new();
    }
    let mut counts: HashMap<String, i64> = HashMap::new();
    for item in summaries
        .iter()
        .filter(|item| account_summary_matches_status(item, filter))
    {
        let value = account_summary_plan_filter_value(item);
        *counts.entry(value).or_insert(0) += 1;
    }
    let mut options = counts
        .into_iter()
        .map(|(value, count)| AccountPlanTypeSummary { value, count })
        .collect::<Vec<_>>();
    options.sort_by(|left, right| {
        account_plan_sort_index(left.value.as_str())
            .cmp(&account_plan_sort_index(right.value.as_str()))
            .then_with(|| left.value.cmp(&right.value))
    });
    options
}

fn account_summary_matches_status(item: &AccountSummary, filter: AccountStatusFilter) -> bool {
    match filter {
        AccountStatusFilter::All | AccountStatusFilter::Active | AccountStatusFilter::Low => true,
        AccountStatusFilter::Exact(kind) => {
            normalize_account_status_kind(item.status.as_str()) == Some(kind)
        }
    }
}

fn normalize_account_status_kind(value: &str) -> Option<AccountStatusKind> {
    match value.trim().to_ascii_lowercase().as_str() {
        "banned" => Some(AccountStatusKind::Banned),
        "disabled" => Some(AccountStatusKind::Disabled),
        "inactive" => Some(AccountStatusKind::Inactive),
        "limited" => Some(AccountStatusKind::Limited),
        "unavailable" => Some(AccountStatusKind::Unavailable),
        "unknown" => Some(AccountStatusKind::Unknown),
        _ => None,
    }
}

fn account_summary_matches_plan(item: &AccountSummary, plan_filter: Option<&str>) -> bool {
    let Some(filter) = plan_filter else {
        return true;
    };
    let normalized = item
        .plan_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_ascii_lowercase();
    let raw = item
        .plan_type_raw
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase());
    if filter == "unknown" {
        return normalized == "unknown" && raw.as_deref().map_or(true, |value| value == "unknown");
    }
    normalized == filter || raw.as_deref() == Some(filter)
}

fn account_summary_plan_filter_value(item: &AccountSummary) -> String {
    let normalized = item
        .plan_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_ascii_lowercase();
    let raw = item
        .plan_type_raw
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase());
    if normalized == "unknown" && raw.as_deref().is_some_and(|value| value != "unknown") {
        return raw.unwrap();
    }
    normalized
}

fn account_plan_sort_index(value: &str) -> usize {
    match value {
        "free" => 0,
        "go" => 1,
        "plus" => 2,
        "pro" => 3,
        "team" => 4,
        "business" => 5,
        "enterprise" => 6,
        "edu" => 7,
        "unknown" => 8,
        _ => 9,
    }
}

fn sort_account_summaries(items: &mut [AccountSummary], sort_mode: AccountSortMode) {
    items.sort_by(|left, right| {
        let size_order = match sort_mode {
            AccountSortMode::Manual => Ordering::Equal,
            AccountSortMode::LargeFirst => account_size_group(left).cmp(&account_size_group(right)),
            AccountSortMode::SmallFirst => account_size_group(right).cmp(&account_size_group(left)),
        };
        size_order
            .then_with(|| left.sort.cmp(&right.sort))
            .then_with(|| left.label.cmp(&right.label))
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn account_size_group(item: &AccountSummary) -> AccountSizeGroup {
    match item
        .plan_type
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "plus" | "pro" | "team" | "business" | "enterprise" => AccountSizeGroup::Large,
        "free" => AccountSizeGroup::Small,
        _ => AccountSizeGroup::Standard,
    }
}

/// 函数 `to_account_summary_with_reason`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - acc: 参数 acc
/// - status_reason: 参数 status_reason
/// - plan_type: 参数 plan_type
/// - plan_type_raw: 参数 plan_type_raw
/// - note: 参数 note
/// - tags: 参数 tags
///
/// # 返回
/// 返回函数执行结果
fn to_account_summary_with_reason(
    acc: Account,
    preferred: bool,
    status_reason: Option<String>,
    has_token: bool,
    plan_type: Option<String>,
    plan_type_raw: Option<String>,
    has_subscription: Option<bool>,
    subscription_plan: Option<String>,
    subscription_expires_at: Option<i64>,
    subscription_renews_at: Option<i64>,
    note: Option<String>,
    tags: Option<String>,
    model_slugs: Vec<String>,
    quota_capacity_primary_window_tokens: Option<i64>,
    quota_capacity_secondary_window_tokens: Option<i64>,
) -> AccountSummary {
    AccountSummary {
        id: acc.id,
        label: acc.label,
        group_name: acc.group_name,
        preferred,
        sort: acc.sort,
        status: acc.status,
        status_reason,
        has_token,
        plan_type,
        plan_type_raw,
        has_subscription,
        subscription_plan,
        subscription_expires_at,
        subscription_renews_at,
        note,
        tags,
        model_slugs,
        quota_capacity_primary_window_tokens,
        quota_capacity_secondary_window_tokens,
    }
}

/// 函数 `to_account_summaries`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - storage: 参数 storage
/// - accounts: 参数 accounts
///
/// # 返回
/// 返回函数执行结果
fn to_account_summaries(
    storage: &codexmanager_core::storage::Storage,
    accounts: Vec<Account>,
) -> Result<Vec<AccountSummary>, String> {
    let account_ids = accounts
        .iter()
        .map(|account| account.id.clone())
        .collect::<Vec<_>>();
    let preferred_account_id = storage
        .preferred_account_id()
        .map_err(|err| format!("load preferred account failed: {err}"))?;
    let status_reasons = storage
        .latest_account_status_reasons(&account_ids)
        .map_err(|err| format!("load account status reasons failed: {err}"))?;
    let tokens = storage
        .list_tokens_by_account_ids(&account_ids)
        .map_err(|err| format!("load account tokens failed: {err}"))?
        .into_iter()
        .map(|token| (token.account_id.clone(), token))
        .collect::<HashMap<String, Token>>();
    let usages = storage
        .latest_usage_snapshots_by_account_ids(&account_ids)
        .map_err(|err| format!("load account usage snapshots failed: {err}"))?
        .into_iter()
        .map(|snapshot| (snapshot.account_id.clone(), snapshot))
        .collect::<HashMap<String, UsageSnapshotRecord>>();
    let metadata = storage
        .list_account_metadata_by_account_ids(&account_ids)
        .map_err(|err| format!("load account metadata failed: {err}"))?
        .into_iter()
        .map(|item| (item.account_id.clone(), item))
        .collect::<HashMap<String, AccountMetadata>>();
    let subscriptions = storage
        .list_account_subscriptions_by_account_ids(&account_ids)
        .map_err(|err| format!("load account subscriptions failed: {err}"))?
        .into_iter()
        .map(|item| (item.account_id.clone(), item))
        .collect::<HashMap<String, AccountSubscription>>();
    let source_assignments = storage
        .list_quota_source_model_assignments_for_source_ids("openai_account", &account_ids)
        .map_err(|err| format!("load quota source assignments failed: {err}"))?;
    let mut model_slugs_by_account: HashMap<String, Vec<String>> = HashMap::new();
    for assignment in source_assignments {
        if assignment.source_kind == "openai_account" {
            model_slugs_by_account
                .entry(assignment.source_id)
                .or_default()
                .push(assignment.model_slug);
        }
    }
    let quota_overrides = storage
        .list_account_quota_capacity_overrides_by_account_ids(&account_ids)
        .map_err(|err| format!("load account quota capacity overrides failed: {err}"))?
        .into_iter()
        .map(|item| (item.account_id.clone(), item))
        .collect::<HashMap<String, AccountQuotaCapacityOverride>>();
    Ok(accounts
        .into_iter()
        .map(|account| {
            map_account_summary(
                account,
                preferred_account_id.as_deref(),
                &status_reasons,
                &tokens,
                &usages,
                &metadata,
                &subscriptions,
                &model_slugs_by_account,
                &quota_overrides,
            )
        })
        .collect())
}

/// 函数 `map_account_summary`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - account: 参数 account
/// - status_reasons: 参数 status_reasons
/// - tokens: 参数 tokens
/// - usages: 参数 usages
/// - metadata: 参数 metadata
///
/// # 返回
/// 返回函数执行结果
fn map_account_summary(
    account: Account,
    preferred_account_id: Option<&str>,
    status_reasons: &HashMap<String, String>,
    tokens: &HashMap<String, Token>,
    usages: &HashMap<String, UsageSnapshotRecord>,
    metadata: &HashMap<String, AccountMetadata>,
    subscriptions: &HashMap<String, AccountSubscription>,
    model_slugs_by_account: &HashMap<String, Vec<String>>,
    quota_overrides: &HashMap<String, AccountQuotaCapacityOverride>,
) -> AccountSummary {
    let account_id = account.id.clone();
    let status_reason = status_reasons.get(&account_id).cloned();
    let preferred = preferred_account_id.is_some_and(|id| id == account_id);
    let subscription = subscriptions.get(&account_id);
    let plan = resolve_effective_account_plan(
        tokens.get(&account_id),
        usages.get(&account_id),
        subscription,
    );
    let has_token = tokens.contains_key(&account_id);
    let account_metadata = metadata.get(&account_id);
    let model_slugs = model_slugs_by_account
        .get(&account_id)
        .cloned()
        .unwrap_or_default();
    let quota_override = quota_overrides.get(&account_id);
    let (fallback_plan_type, plan_type_raw) = match plan {
        Some(value) => (Some(value.normalized), value.raw),
        None => (None, None),
    };
    let subscription_plan = subscription.and_then(|value| value.plan_type.clone());
    let plan_type = fallback_plan_type;
    to_account_summary_with_reason(
        account,
        preferred,
        status_reason,
        has_token,
        plan_type,
        plan_type_raw,
        subscription.map(|value| value.has_subscription),
        subscription_plan,
        subscription.and_then(|value| value.expires_at),
        subscription.and_then(|value| value.renews_at),
        account_metadata.and_then(|value| value.note.clone()),
        account_metadata.and_then(|value| value.tags.clone()),
        model_slugs,
        quota_override.and_then(|value| value.primary_window_tokens),
        quota_override.and_then(|value| value.secondary_window_tokens),
    )
}
