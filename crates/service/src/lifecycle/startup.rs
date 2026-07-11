use std::io;
use std::thread;

pub struct ServerHandle {
    pub addr: String,
    join: thread::JoinHandle<()>,
}

impl ServerHandle {
    /// 函数 `join`
    ///
    /// 作者: gaohongshun
    ///
    /// 时间: 2026-04-02
    ///
    /// # 参数
    /// - self: 参数 self
    ///
    /// # 返回
    /// 无
    pub fn join(self) {
        let _ = self.join.join();
    }
}

/// 函数 `start_one_shot_server`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// 无
///
/// # 返回
/// 返回函数执行结果
pub fn start_one_shot_server() -> std::io::Result<ServerHandle> {
    crate::portable::bootstrap_current_process();
    crate::gateway::reload_runtime_config_from_env();
    if let Err(err) = crate::storage_helpers::initialize_storage() {
        log::warn!("storage startup init skipped: {}", err);
    }
    crate::sync_runtime_settings_from_storage();
    let server = tiny_http::Server::http("127.0.0.1:0")
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    let addr = server
        .server_addr()
        .to_ip()
        .map(|a| a.to_string())
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "server addr missing"))?;
    let join = thread::spawn(move || {
        if let Some(request) = server.incoming_requests().next() {
            crate::http::backend_router::handle_backend_request(request);
        }
    });
    Ok(ServerHandle { addr, join })
}

/// 函数 `start_server`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - addr: 参数 addr
///
/// # 返回
/// 返回函数执行结果
pub fn start_server(addr: &str) -> std::io::Result<()> {
    crate::portable::bootstrap_current_process();
    crate::gateway::reload_runtime_config_from_env();
    crate::gateway::upstream::protocol::aggregate_api::init_aggregate_api_candidate_cache_config();
    if let Err(err) = crate::storage_helpers::initialize_storage() {
        log::warn!("storage startup init skipped: {}", err);
    }
    crate::sync_runtime_settings_from_storage();
    // 版本信息已从持久化存储恢复（sync_runtime_settings_from_storage），
    // 网络拉取完全交给后台线程异步首刷，避免阻塞启动主路径。
    crate::app_settings::ensure_codex_latest_version_sync();
    crate::network_diagnostics::ensure_network_diagnostics();
    crate::usage_refresh::ensure_usage_polling();
    crate::usage_refresh::ensure_gateway_keepalive();
    crate::usage_refresh::ensure_token_refresh_polling();
    crate::usage_refresh::ensure_warmup_cron();
    crate::plugin::ensure_plugin_scheduler();
    crate::gateway::ensure_idle_db_maintenance_scheduler();
    crate::http::server::start_http(addr)
}
