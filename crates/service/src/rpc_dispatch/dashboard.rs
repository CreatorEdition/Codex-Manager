use codexmanager_core::rpc::types::{JsonRpcRequest, JsonRpcResponse};

use crate::{dashboard, RpcActor};

pub(super) fn try_handle(req: &JsonRpcRequest, actor: &RpcActor) -> Option<JsonRpcResponse> {
    let result = match req.method.as_str() {
        "dashboard/adminUsageSummary" => {
            let start_ts = super::i64_param(req, "startTs");
            let end_ts = super::i64_param(req, "endTs");
            let ranking_limit = super::i64_param(req, "rankingLimit");
            super::value_or_error(dashboard::read_admin_usage_summary(
                actor,
                start_ts,
                end_ts,
                ranking_limit,
            ))
        }
        "dashboard/adminOverview" => {
            let request_log_limit = super::i64_param(req, "requestLogLimit");
            let day_start_ts = super::i64_param(req, "dayStartTs");
            let day_end_ts = super::i64_param(req, "dayEndTs");
            let account_limit = super::i64_param(req, "accountLimit");
            super::value_or_error(dashboard::read_admin_overview(
                actor,
                request_log_limit,
                day_start_ts,
                day_end_ts,
                account_limit,
            ))
        }
        "dashboard/memberSummary" => {
            let user_id = super::string_param(req, "userId");
            let day_start_ts = super::i64_param(req, "dayStartTs");
            let day_end_ts = super::i64_param(req, "dayEndTs");
            super::value_or_error(dashboard::read_member_dashboard_summary(
                actor,
                user_id,
                day_start_ts,
                day_end_ts,
            ))
        }
        _ => return None,
    };

    Some(super::response(req, result))
}
