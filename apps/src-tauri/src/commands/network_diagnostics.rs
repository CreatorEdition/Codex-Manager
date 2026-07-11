use crate::commands::shared::rpc_call_in_background;

/// 读取服务端缓存的出口网络诊断。
#[tauri::command]
pub async fn service_network_diagnostics_get(
    addr: Option<String>,
) -> Result<serde_json::Value, String> {
    rpc_call_in_background("networkDiagnostics/get", addr, None).await
}

/// 异步请求服务端刷新出口网络诊断。
#[tauri::command]
pub async fn service_network_diagnostics_refresh(
    addr: Option<String>,
) -> Result<serde_json::Value, String> {
    rpc_call_in_background("networkDiagnostics/refresh", addr, None).await
}
