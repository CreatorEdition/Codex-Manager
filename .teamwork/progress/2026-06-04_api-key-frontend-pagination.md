# API Key 前端分页进度

## 范围

- 平台 Key 页面新增搜索、状态筛选、每页数量和翻页控件。
- 新增 `accountClient.listApiKeyPage()`，保留无参 `listApiKeys()` 供日志页和旧调用兼容。
- `useApiKeys()` 接收分页参数，并将 query key 拆成分页维度。
- 桌面 Tauri `service_apikey_list` 同步透传分页参数，避免桌面端退回无参全量查询。

## 非范围

- 本次不改日志页 API Key lookup，全量 lookup 后续独立处理。
- 本次不改启动快照 `apiKeys` 预载，避免破坏现有启动占位行为。
- 本次不改 API Key 用量统计接口，Token/费用列仍读取全量 `usageStats`。

## 验证

- ✅ `corepack pnpm -C apps run test:runtime`
- ✅ `corepack pnpm -C apps run build`
- ✅ `cargo fmt --all --check`
- ✅ `cargo check --manifest-path apps/src-tauri/Cargo.toml`

说明：`cargo check` 首次在沙箱内访问 crates.io 时出现 `schannel: AcquireCredentialsHandle failed: SEC_E_NO_CREDENTIALS`，提升权限联网拉取依赖后通过。
