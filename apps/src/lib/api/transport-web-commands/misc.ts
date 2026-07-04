import type { WebCommandDescriptor } from "./shared";
import { WEB_RPC_MAINTENANCE_TIMEOUT_MS, noRetryTimeoutOptions } from "./shared";
import { openExternalUrlDirect, openInBrowserDirect, showMainWindowDirect, unsupportedOpenInFileManager, unsupportedOpenUpdateLogsDir } from "./browser-direct";

export function createMiscWebCommands(): Record<string, WebCommandDescriptor> {
  return {
    service_initialize: { rpcMethod: "initialize" },
    service_startup_snapshot: {
      rpcMethod: "startup/snapshot",
      requestOptions: {
        timeoutMs: 30000,
        retries: 0,
        timeoutMessage: "RPC startup/snapshot 超时：启动快照查询超过 30 秒",
      },
    },
    app_settings_get: { rpcMethod: "appSettings/get" },
    app_settings_set: { rpcMethod: "appSettings/set", mapParams: (params) => params && typeof params.patch === "object" && params.patch !== null ? (params.patch as Record<string, unknown>) : {} },
    service_requestlog_list: {
      rpcMethod: "requestlog/list",
      requestOptions: {
        timeoutMs: 30000,
        retries: 0,
        timeoutMessage: "RPC requestlog/list 超时：请求日志查询超过 30 秒",
      },
    },
    service_requestlog_summary: {
      rpcMethod: "requestlog/summary",
      requestOptions: {
        timeoutMs: 30000,
        retries: 0,
        timeoutMessage: "RPC requestlog/summary 超时：请求日志摘要查询超过 30 秒",
      },
    },
    service_requestlog_clear: {
      rpcMethod: "requestlog/clear",
      requestOptions: noRetryTimeoutOptions(
        WEB_RPC_MAINTENANCE_TIMEOUT_MS,
        "RPC requestlog/clear 超时：请求日志清理超过 60 秒",
      ),
    },
    service_requestlog_today_summary: { rpcMethod: "requestlog/today_summary" },
    service_plugin_catalog_list: { rpcMethod: "plugin/catalog/list" },
    service_plugin_catalog_refresh: {
      rpcMethod: "plugin/catalog/refresh",
      requestOptions: noRetryTimeoutOptions(
        WEB_RPC_MAINTENANCE_TIMEOUT_MS,
        "RPC plugin/catalog/refresh 超时：插件目录刷新超过 60 秒",
      ),
    },
    service_plugin_install: {
      rpcMethod: "plugin/install",
      requestOptions: noRetryTimeoutOptions(
        WEB_RPC_MAINTENANCE_TIMEOUT_MS,
        "RPC plugin/install 超时：插件安装超过 60 秒",
      ),
    },
    service_plugin_update: {
      rpcMethod: "plugin/update",
      requestOptions: noRetryTimeoutOptions(
        WEB_RPC_MAINTENANCE_TIMEOUT_MS,
        "RPC plugin/update 超时：插件更新超过 60 秒",
      ),
    },
    service_plugin_uninstall: {
      rpcMethod: "plugin/uninstall",
      requestOptions: noRetryTimeoutOptions(
        WEB_RPC_MAINTENANCE_TIMEOUT_MS,
        "RPC plugin/uninstall 超时：插件卸载超过 60 秒",
      ),
    },
    service_plugin_list: { rpcMethod: "plugin/list" },
    service_plugin_enable: { rpcMethod: "plugin/enable" },
    service_plugin_disable: { rpcMethod: "plugin/disable" },
    service_plugin_tasks_update: { rpcMethod: "plugin/tasks/update" },
    service_plugin_tasks_list: { rpcMethod: "plugin/tasks/list" },
    service_plugin_tasks_run: {
      rpcMethod: "plugin/tasks/run",
      requestOptions: noRetryTimeoutOptions(
        WEB_RPC_MAINTENANCE_TIMEOUT_MS,
        "RPC plugin/tasks/run 超时：插件任务运行超过 60 秒",
      ),
    },
    service_plugin_logs_list: { rpcMethod: "plugin/logs/list" },
    service_listen_config_get: { rpcMethod: "service/listenConfig/get" },
    service_listen_config_set: { rpcMethod: "service/listenConfig/set" },
    open_in_browser: { direct: openInBrowserDirect },
    open_external_url: { direct: openExternalUrlDirect },
    open_in_file_manager: { direct: unsupportedOpenInFileManager },
    app_show_main_window: { direct: showMainWindowDirect },
    app_update_open_logs_dir: { direct: unsupportedOpenUpdateLogsDir },
  };
}
