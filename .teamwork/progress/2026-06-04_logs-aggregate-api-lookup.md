# 日志页聚合 API lookup 进度

## 范围

- 新增 `aggregateApi/lookup` RPC，按传入聚合 API ID 批量返回聚合 API 摘要。
- `aggregateApi/lookup` 不加入成员 allowlist，日志页仅在管理员模式下启用。
- Tauri 与 Web command map 同步注册 `service_aggregate_api_lookup`。
- 日志页从当前页请求日志和当前页 API Key 摘要中提取聚合 API ID，按需查询展示信息，不再调用无参 `listAggregateApis()` 全量加载。
- `aggregateApi/list` 的模型分配装饰复用按 source id 查询，避免额外全量扫描模型分配表。

## 非范围

- 本次不改启动快照中的全量预载。
- 本次不改聚合 API 管理页分页。
- 本次不实现按上游 URL 反查聚合 API 的批量 lookup，日志页缺失 ID 时继续使用既有 URL/ID fallback。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service aggregate_api_lookup_is_admin_only_and_filters_requested_ids`
- ✅ `corepack pnpm -C apps run test:runtime`
- ✅ `corepack pnpm -C apps run build`
- ✅ `cargo check --manifest-path apps/src-tauri/Cargo.toml`
