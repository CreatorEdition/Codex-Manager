use codexmanager_core::rpc::types::{JsonRpcRequest, JsonRpcResponse};

/// 处理出口网络诊断 RPC。
pub(super) fn try_handle(req: &JsonRpcRequest) -> Option<JsonRpcResponse> {
    let result = match req.method.as_str() {
        "networkDiagnostics/get" => super::as_json(crate::network_diagnostics_get()),
        "networkDiagnostics/refresh" => super::as_json(crate::network_diagnostics_refresh()),
        _ => return None,
    };
    Some(super::response(req, result))
}
