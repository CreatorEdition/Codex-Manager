use codexmanager_core::rpc::types::{RequestLogErrorCodeSummaryItem, RequestLogErrorSummaryResult};

use crate::storage_helpers::open_storage;

/// 默认返回的错误类别上限。
const DEFAULT_ERROR_SUMMARY_LIMIT: i64 = 50;
/// 错误去重汇总允许查询的最大时间窗口（秒），默认 7 天，避免一次扫描过宽。
const MAX_ERROR_SUMMARY_RANGE_SECS: i64 = 7 * 24 * 60 * 60;

/// 函数 `resolve_window_bounds`
///
/// 中文注释：校验并归一化错误去重汇总的时间窗口。
/// - 两端都给：要求 end>start，且跨度不超过 7 天上限。
/// - 都不给：返回 (None, None)，由 storage 层按全量（受 retention 约束）聚合。
/// - 只给一端：拒绝，避免半开窗口语义歧义。
///
/// # 参数
/// - start_ts: 起始时间戳（含），秒
/// - end_ts: 结束时间戳（含），秒
///
/// # 返回
/// 归一化后的 (start, end) 选项对
fn resolve_window_bounds(
    start_ts: Option<i64>,
    end_ts: Option<i64>,
) -> Result<(Option<i64>, Option<i64>), String> {
    match (start_ts, end_ts) {
        (Some(start), Some(end)) => {
            if end <= start {
                return Err("endTs must be greater than startTs".to_string());
            }
            if end - start > MAX_ERROR_SUMMARY_RANGE_SECS {
                return Err("requested error summary range is too large".to_string());
            }
            Ok((Some(start), Some(end)))
        }
        (None, None) => Ok((None, None)),
        _ => Err("startTs and endTs must be provided together".to_string()),
    }
}

/// 函数 `read_request_log_error_summary`
///
/// 中文注释：按规范化 error_code 聚合错误请求日志，返回错误去重汇总。
/// 该结果跨全部账号/来源聚合，仅供管理员调用（在 RPC 分发层强制 admin 校验）。
///
/// # 参数
/// - start_ts: 起始时间戳（含），None 表示不限下界
/// - end_ts: 结束时间戳（含），None 表示不限上界
/// - limit: 返回的错误类别上限，<=0 时取默认 50（storage 层再夹紧到 500）
///
/// # 返回
/// 按出现次数降序的错误码聚合列表
pub(crate) fn read_request_log_error_summary(
    start_ts: Option<i64>,
    end_ts: Option<i64>,
    limit: Option<i64>,
) -> Result<RequestLogErrorSummaryResult, String> {
    let (start, end) = resolve_window_bounds(start_ts, end_ts)?;
    let normalized_limit = limit
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_ERROR_SUMMARY_LIMIT);
    let storage = open_storage().ok_or_else(|| "open storage failed".to_string())?;
    let rows = storage
        .summarize_request_log_error_codes(start, end, normalized_limit)
        .map_err(|err| format!("summarize request log error codes failed: {err}"))?;
    let items = rows
        .into_iter()
        .map(|row| RequestLogErrorCodeSummaryItem {
            error_code: row.error_code,
            count: row.count,
            last_seen: row.last_seen,
            sample_message: row.sample_message,
        })
        .collect();
    Ok(RequestLogErrorSummaryResult { items })
}
