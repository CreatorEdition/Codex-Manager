# 日志页 API Key lookup 进度

## 范围

- 新增 `apikey/lookup` RPC，按传入 Key ID 批量返回平台 Key 摘要。
- 成员模式下 lookup 只返回当前用户拥有的 Key，不泄露其他用户 Key。
- Tauri 与 Web command map 同步注册 `service_apikey_lookup`。
- 日志页从当前页请求日志中提取唯一 `keyId`，按需查询展示信息，不再调用无参 `listApiKeys()` 全量加载。

## 非范围

- 本次不改日志页账号 lookup，仍需后续按当前页 `accountId` 批量查询。
- 本次不改日志页聚合 API lookup，仍需后续按当前页聚合 API ID 批量查询。
- 本次不改启动快照中的全量 `apiKeys` 预载。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service member_api_key_lookup_filters_to_owned_ids`
- ✅ `corepack pnpm -C apps run test:runtime`
- ✅ `corepack pnpm -C apps run build`
- ✅ `cargo check --manifest-path apps/src-tauri/Cargo.toml`
