use codexmanager_core::rpc::types::{JsonRpcRequest, JsonRpcResponse};

use crate::startup_snapshot::{self, StartupSnapshotOptions};
use crate::RpcActor;

/// 函数 `try_handle`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - super: 参数 super
///
/// # 返回
/// 返回函数执行结果
pub(super) fn try_handle(req: &JsonRpcRequest, actor: &RpcActor) -> Option<JsonRpcResponse> {
    let result = match req.method.as_str() {
        "startup/snapshot" => {
            let request_log_limit = super::i64_param(req, "requestLogLimit");
            let day_start_ts = super::i64_param(req, "dayStartTs");
            let day_end_ts = super::i64_param(req, "dayEndTs");
            let account_limit = super::i64_param(req, "accountLimit");
            let api_key_limit = super::i64_param(req, "apiKeyLimit");
            let options = StartupSnapshotOptions {
                include_usage_aggregate: super::bool_param(req, "includeUsageAggregate")
                    .unwrap_or(true),
                include_today_summary: super::bool_param(req, "includeTodaySummary")
                    .unwrap_or(true),
                include_recent_logs: super::bool_param(req, "includeRecentLogs").unwrap_or(true),
                include_api_models: super::bool_param(req, "includeApiModels").unwrap_or(true),
            };
            super::value_or_error(startup_snapshot::read_startup_snapshot_for_actor(
                actor,
                request_log_limit,
                day_start_ts,
                day_end_ts,
                account_limit,
                api_key_limit,
                options,
            ))
        }
        _ => return None,
    };

    Some(super::response(req, result))
}
