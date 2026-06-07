use codexmanager_core::rpc::types::{
    AccountListParams, ApiKeyListParams, RequestLogTodaySummaryResult, StartupSnapshotResult,
    UsageAggregateSummaryResult,
};

use crate::{
    account_list, apikey_list, apikey_models, gateway, requestlog_list, requestlog_today_summary,
    storage_helpers::open_storage, usage_aggregate, usage_list, RpcActor,
};

const MAX_STARTUP_SECTION_LIMIT: i64 = 500;

#[derive(Debug, Clone, Copy)]
pub(crate) struct StartupSnapshotOptions {
    pub include_usage_aggregate: bool,
    pub include_today_summary: bool,
    pub include_recent_logs: bool,
    pub include_api_models: bool,
}

impl Default for StartupSnapshotOptions {
    fn default() -> Self {
        Self {
            include_usage_aggregate: false,
            include_today_summary: false,
            include_recent_logs: false,
            include_api_models: false,
        }
    }
}

/// 函数 `read_startup_snapshot`
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
pub(crate) fn read_startup_snapshot(
    request_log_limit: Option<i64>,
    day_start_ts: Option<i64>,
    day_end_ts: Option<i64>,
    account_limit: Option<i64>,
    api_key_limit: Option<i64>,
    options: StartupSnapshotOptions,
) -> Result<StartupSnapshotResult, String> {
<<<<<<< HEAD
    let storage = open_storage().ok_or_else(|| "open storage failed".to_string())?;
    let account_total = storage
        .account_count_filtered(None, None)
        .map_err(|err| format!("count accounts failed: {err}"))?;
    let account_available = storage
        .account_count_active_available(None, None)
        .map_err(|err| format!("count available accounts failed: {err}"))?;
    let api_key_total = storage
        .api_key_count_filtered(None, None, None)
        .map_err(|err| format!("count api keys failed: {err}"))?;
    drop(storage);

    let accounts = read_startup_accounts(account_limit)?;
    let usage_snapshots = match account_limit.and_then(normalize_startup_section_limit) {
        Some(_) => {
            let account_ids = accounts
                .iter()
                .map(|account| account.id.clone())
                .collect::<Vec<_>>();
            usage_list::read_usage_snapshots(Some(account_ids), None)?
        }
        None => usage_list::read_usage_snapshots(None, account_limit)?,
    };
    let usage_aggregate_summary = if options.include_usage_aggregate {
        usage_aggregate::read_usage_aggregate_summary()?
    } else {
        UsageAggregateSummaryResult::default()
    };
    let api_keys = read_startup_api_keys(&RpcActor::system_admin(), api_key_limit)?.items;
    let api_models = if options.include_api_models {
        apikey_models::read_model_options(false)?
    } else {
        Default::default()
    };
=======
    let storage =
        storage_helpers::open_storage().ok_or_else(|| "open storage failed".to_string())?;
    let accounts = storage
        .list_accounts()
        .map_err(|err| format!("list accounts failed: {err}"))?;
    let db_path = std::env::var("CODEXMANAGER_DB_PATH").unwrap_or_else(|_| "<unset>".to_string());
    log::info!(
        "startup/snapshot read: db_path={} account_count={}",
        db_path,
        accounts.len()
    );
    let account_context = account_list::build_account_summary_context(&storage, &accounts)?;
    let usage_aggregate_summary = usage_aggregate::compute_usage_aggregate_summary(
        &accounts,
        &account_context.usage_snapshots,
    );
    let usage_snapshots = account_context
        .usage_snapshots
        .into_iter()
        .map(crate::usage_read::usage_snapshot_result_from_record)
        .collect();
    let api_keys = apikey_list::read_api_keys()?;
    let api_models = apikey_models::read_model_options(false)?;
>>>>>>> cf306b11 (修复未注册的插件)
    let manual_preferred_account_id = gateway::manual_preferred_account();
    let request_log_today_summary = if options.include_today_summary {
        requestlog_today_summary::read_requestlog_today_summary(day_start_ts, day_end_ts)?
    } else {
        empty_today_summary()
    };
    let request_logs = if options.include_recent_logs {
        requestlog_list::read_request_logs(None, request_log_limit)?
    } else {
        Vec::new()
    };

    Ok(StartupSnapshotResult {
        account_total,
        account_available,
        api_key_total,
        accounts,
        usage_snapshots,
        usage_aggregate_summary,
        api_keys,
        api_models,
        manual_preferred_account_id,
        request_log_today_summary,
        request_logs,
    })
}

pub(crate) fn read_startup_snapshot_for_actor(
    actor: &RpcActor,
    request_log_limit: Option<i64>,
    day_start_ts: Option<i64>,
    day_end_ts: Option<i64>,
    account_limit: Option<i64>,
    api_key_limit: Option<i64>,
    options: StartupSnapshotOptions,
) -> Result<StartupSnapshotResult, String> {
    if actor.is_admin() {
        return read_startup_snapshot(
            request_log_limit,
            day_start_ts,
            day_end_ts,
            account_limit,
            api_key_limit,
            options,
        );
    }
    let user_id = actor
        .user_id
        .as_deref()
        .ok_or_else(|| "permission_denied: startup requires user session".to_string())?;
    let key_ids = crate::list_api_key_ids_for_user(user_id)?;
    let api_key_total = key_ids.len() as i64;
    let api_keys = read_startup_api_keys(actor, api_key_limit)?.items;
    let api_models = if options.include_api_models {
        apikey_models::read_model_options(false)?
    } else {
        Default::default()
    };
    let request_log_today_summary = if options.include_today_summary {
        requestlog_today_summary::read_requestlog_today_summary_for_key_ids(
            day_start_ts,
            day_end_ts,
            &key_ids,
        )?
    } else {
        empty_today_summary()
    };
    let request_logs = if options.include_recent_logs {
        requestlog_list::read_request_logs_for_key_ids(None, request_log_limit, &key_ids)?
    } else {
        Vec::new()
    };

    Ok(StartupSnapshotResult {
        account_total: 0,
        account_available: 0,
        api_key_total,
        accounts: Vec::new(),
        usage_snapshots: Vec::new(),
        usage_aggregate_summary: UsageAggregateSummaryResult::default(),
        api_keys,
        api_models,
        manual_preferred_account_id: None,
        request_log_today_summary,
        request_logs,
    })
}

fn read_startup_accounts(
    account_limit: Option<i64>,
) -> Result<Vec<codexmanager_core::rpc::types::AccountSummary>, String> {
    match account_limit.and_then(normalize_startup_section_limit) {
        Some(0) => Ok(Vec::new()),
        Some(limit) => {
            let params = AccountListParams {
                page: 1,
                page_size: limit,
                ..AccountListParams::default()
            };
            account_list::read_accounts(params, true).map(|result| result.items)
        }
        None => account_list::read_accounts(AccountListParams::default(), false)
            .map(|result| result.items),
    }
}

fn read_startup_api_keys(
    actor: &RpcActor,
    api_key_limit: Option<i64>,
) -> Result<codexmanager_core::rpc::types::ApiKeyListResult, String> {
    match api_key_limit.and_then(normalize_startup_section_limit) {
        Some(0) => Ok(codexmanager_core::rpc::types::ApiKeyListResult {
            items: Vec::new(),
            total: 0,
            page: 1,
            page_size: 0,
        }),
        Some(limit) => apikey_list::read_api_key_list_for_actor(
            actor,
            ApiKeyListParams {
                page: 1,
                page_size: limit,
                ..ApiKeyListParams::default()
            },
            true,
        ),
        None => {
            let items = apikey_list::read_api_keys_for_actor(actor)?;
            let total = items.len() as i64;
            Ok(codexmanager_core::rpc::types::ApiKeyListResult {
                items,
                total,
                page: 1,
                page_size: total,
            })
        }
    }
}

fn normalize_startup_section_limit(value: i64) -> Option<i64> {
    if value < 0 {
        return None;
    }
    Some(value.clamp(0, MAX_STARTUP_SECTION_LIMIT))
}

fn empty_today_summary() -> RequestLogTodaySummaryResult {
    RequestLogTodaySummaryResult {
        input_tokens: 0,
        cached_input_tokens: 0,
        output_tokens: 0,
        reasoning_output_tokens: 0,
        today_tokens: 0,
        estimated_cost: 0.0,
    }
}
