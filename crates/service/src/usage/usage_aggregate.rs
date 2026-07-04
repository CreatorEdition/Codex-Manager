use codexmanager_core::rpc::types::UsageAggregateSummaryResult;

use crate::storage_helpers::open_storage;

/// 函数 `read_usage_aggregate_summary`
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
pub(crate) fn read_usage_aggregate_summary() -> Result<UsageAggregateSummaryResult, String> {
    let storage = open_storage().ok_or_else(|| "open storage failed".to_string())?;
    storage
        .usage_aggregate_summary()
        .map_err(|err| format!("usage aggregate summary failed: {err}"))
}
