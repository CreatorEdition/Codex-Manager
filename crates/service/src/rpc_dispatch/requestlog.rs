use codexmanager_core::rpc::types::{JsonRpcRequest, JsonRpcResponse, RequestLogListParams};

use crate::RpcActor;
use crate::{
    requestlog_clear, requestlog_error_summary, requestlog_list, requestlog_summary,
    requestlog_today_summary,
};

fn actor_key_ids(actor: &RpcActor) -> Result<Vec<String>, String> {
    if actor.is_admin() {
        return Ok(Vec::new());
    }
    let user_id = actor
        .user_id
        .as_deref()
        .ok_or_else(|| "permission_denied: requestlog requires user session".to_string())?;
    crate::list_api_key_ids_for_user(user_id)
}

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
        "requestlog/list" => {
            let params = req
                .params
                .clone()
                .map(serde_json::from_value::<RequestLogListParams>)
                .transpose()
                .map(|params| params.unwrap_or_default())
                .map(RequestLogListParams::normalized)
                .map_err(|err| format!("invalid requestlog/list params: {err}"));
            super::value_or_error(params.and_then(|params| {
                if actor.is_admin() {
                    requestlog_list::read_request_log_page(params)
                } else {
                    let key_ids = actor_key_ids(actor)?;
                    requestlog_list::read_request_log_page_for_key_ids(params, &key_ids)
                }
            }))
        }
        "requestlog/summary" => {
            let params = req
                .params
                .clone()
                .map(serde_json::from_value::<RequestLogListParams>)
                .transpose()
                .map(|params| params.unwrap_or_default())
                .map(RequestLogListParams::normalized)
                .map_err(|err| format!("invalid requestlog/summary params: {err}"));
            super::value_or_error(params.and_then(|params| {
                if actor.is_admin() {
                    requestlog_summary::read_request_log_filter_summary(params)
                } else {
                    let key_ids = actor_key_ids(actor)?;
                    requestlog_summary::read_request_log_filter_summary_for_key_ids(
                        params, &key_ids,
                    )
                }
            }))
        }
        "requestlog/errorSummary" => {
            // 中文注释：错误去重汇总跨全部账号/来源聚合，属管理面诊断数据，仅管理员可调用，
            // 避免成员会话拿到全局错误分布。非管理员直接返回 permission_denied。
            if !actor.is_admin() {
                super::value_or_error::<serde_json::Value>(Err(
                    "permission_denied: requestlog/errorSummary requires admin".to_string(),
                ))
            } else {
                let start_ts = super::i64_param(req, "startTs");
                let end_ts = super::i64_param(req, "endTs");
                let limit = super::i64_param(req, "limit");
                super::value_or_error(requestlog_error_summary::read_request_log_error_summary(
                    start_ts, end_ts, limit,
                ))
            }
        }
        "requestlog/clear" => super::ok_or_error(requestlog_clear::clear_request_logs()),
        "requestlog/today_summary" => {
            let day_start_ts = super::i64_param(req, "dayStartTs");
            let day_end_ts = super::i64_param(req, "dayEndTs");
            super::value_or_error(if actor.is_admin() {
                requestlog_today_summary::read_requestlog_today_summary(day_start_ts, day_end_ts)
            } else {
                actor_key_ids(actor).and_then(|key_ids| {
                    requestlog_today_summary::read_requestlog_today_summary_for_key_ids(
                        day_start_ts,
                        day_end_ts,
                        &key_ids,
                    )
                })
            })
        }
        _ => return None,
    };

    Some(super::response(req, result))
}
