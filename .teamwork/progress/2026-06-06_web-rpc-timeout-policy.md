# 2026-06-06 Web RPC 超时与重试治理

## 背景

- 用户抓包显示 `/api/rpc` 调用 `startup/snapshot` 与 `quota/modelPools` 时约 10 秒后出现 `net::ERR_ABORTED`。
- 前端 Web 模式所有 `/api/rpc` 默认使用 `fetchWithRetry` 的 `timeoutMs=10000` 与 `retries=3`。
- 浏览器 abort 不等于 service 后端停止执行；默认重试会让重 RPC 被重复提交，放大 CPU、SQLite 连接和 WAL 压力。

## 变更

- `fetchWithRetry` 新增 `timeoutMessage`，并把自身超时转换为 `TimeoutError`。
- 区分自身超时与调用方 `AbortSignal` 取消；调用方取消继续抛出 `AbortError`，不伪装成超时。
- Web command descriptor 支持 `requestOptions`。
- `startup/snapshot` 与 `quota/modelPools` 在 Web 模式下默认：
  - `timeoutMs=30000`
  - `retries=0`
  - 使用带 RPC method 的中文超时信息。
- 普通轻 RPC 保持 transport 默认策略，避免一刀切拉长所有请求。

## 验证

- `node --test apps/tests/request.test.mjs apps/tests/transport-web-commands.test.mjs`
- `node --test apps/tests/runtime-capabilities.test.mjs apps/tests/gateway-endpoints.test.mjs apps/tests/transport-errors.test.mjs apps/tests/gateway-settings.test.mjs apps/tests/transport-web-commands.test.mjs apps/tests/rpc-http.test.mjs apps/tests/request.test.mjs apps/tests/startup-snapshot.test.mjs apps/tests/timeout.test.mjs`
- `git diff --check`

## 已知情况

- `pnpm` 不在当前 PowerShell PATH 中，改用本地 `apps/node_modules/.bin` 工具。
- `tsc --project apps/tsconfig.json --noEmit` 当前失败在既有 `apps/tests/navigation-cache.spec.ts:123`，与本次修改文件无关。
- 本次只是治理客户端超时/重试放大效应；后端重查询仍需继续通过分页、下推和后台化修复。
