import type { WebCommandDescriptor } from "./shared";
import { WEB_RPC_LONG_OPERATION_TIMEOUT_MS, noRetryTimeoutOptions } from "./shared";

export function createLoginWebCommands(): Record<string, WebCommandDescriptor> {
  return {
    service_login_start: { rpcMethod: "account/login/start", mapParams: (params) => ({ ...(params ?? {}), type: typeof params?.loginType === "string" && params.loginType.trim() ? params.loginType : "chatgpt", openBrowser: false }) },
    service_login_status: { rpcMethod: "account/login/status" },
    service_login_complete: { rpcMethod: "account/login/complete" },
    service_login_chatgpt_auth_tokens: { rpcMethod: "account/login/start", mapParams: (params) => ({ ...(params ?? {}), type: "chatgptAuthTokens" }) },
    service_account_read: { rpcMethod: "account/read" },
    service_account_logout: { rpcMethod: "account/logout" },
    service_chatgpt_auth_tokens_refresh: { rpcMethod: "account/chatgptAuthTokens/refresh" },
    service_chatgpt_auth_tokens_refresh_all: {
      rpcMethod: "account/chatgptAuthTokens/refreshAll",
      requestOptions: noRetryTimeoutOptions(
        WEB_RPC_LONG_OPERATION_TIMEOUT_MS,
        "RPC account/chatgptAuthTokens/refreshAll 超时：全量 Refresh Token 刷新超过 120 秒",
      ),
    },
  };
}
