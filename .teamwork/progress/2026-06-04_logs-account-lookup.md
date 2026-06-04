# 日志页账号 lookup 进度

## 范围

- 新增 `account/lookup` RPC，按传入账号 ID 批量返回账号摘要。
- `account/lookup` 不加入成员 allowlist，日志页仅在管理员模式下启用，避免成员通过日志反查账号标签。
- Tauri 与 Web command map 同步注册 `service_account_lookup`。
- 日志页从当前页请求日志中提取唯一 `accountId`，按需查询账号名称，不再调用无参 `accountClient.list()` 全量加载账号。

## 非范围

- 本次不改日志页聚合 API lookup，仍需后续按当前页聚合 API ID 批量查询。
- 本次不改启动快照中的全量账号预载。
- 本次不改账号页前端分页接入。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service account_lookup_is_admin_only_and_filters_requested_ids`
- ✅ `corepack pnpm -C apps run test:runtime`
- ✅ `corepack pnpm -C apps run build`
- ✅ `cargo check --manifest-path apps/src-tauri/Cargo.toml`
