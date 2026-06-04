# 平台 Key 用量按需统计进度

## 范围

- `apikey/usageStats` 新增可选 `keyIds` 参数。
- 无参数调用保留旧全量语义，兼容旧客户端和外部 RPC。
- 显式传空数组返回空列表，避免当前页无 Key 时误退回全量聚合。
- Core storage 新增按 key IDs 聚合 request token stats 的查询。
- member 路径先收敛到自己的 Key IDs，再执行 scoped 聚合，不再全量聚合后内存过滤。
- Tauri `service_apikey_usage_stats` 与前端 `accountClient.listApiKeyUsageStats({ keyIds })` 透传参数。
- 平台 Key 页面从当前页 Key 提取 ID，只查询当前页 Token/费用，并将统计卡片文案改为当前页口径。
- 新增 RPC 回归测试覆盖按 ID 过滤、去重、空数组不回全量。

## 非范围

- 本次不新增全局 API Key 用量概览接口。
- 本次不重构 request token stats/rollups 的索引和保留策略。
- 本次不改变平台 Key 列表分页行为。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service rpc_apikey_usage_stats_filters_requested_key_ids`
- ✅ `corepack pnpm -C apps run test:runtime`
- ✅ `corepack pnpm -C apps run build`
