# 启动快照瘦身进度

## 范围

- `startup/snapshot` 新增 `accountLimit` 与 `apiKeyLimit` 参数。
- 不传限制参数时保留旧行为，兼容旧客户端和外部直接 RPC 调用。
- 前端 `serviceClient.getStartupSnapshot()` 默认只预载 20 个账号与 20 个平台 Key，避免启动阶段全量搬运几千条管理数据。
- `StartupSnapshot` 新增 `accountTotal`、`accountAvailable`、`apiKeyTotal` 元数据，首页统计使用元数据而不是当前预载数组长度。
- 桌面 Tauri `service_startup_snapshot` 补齐 `dayStartTs`、`dayEndTs`、`accountLimit`、`apiKeyLimit` 透传。
- 新增 RPC 回归测试覆盖限制预载数组但保留总数元数据。

## 非范围

- 本次不处理 `quota/modelPools` 全量 sources 构建。
- 本次不重构 `usageAggregateSummary` 的聚合 SQL。
- 本次不改变请求日志分页和摘要接口。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service rpc_startup_snapshot_limits_prefetch_sections_and_returns_totals`
- ✅ `corepack pnpm -C apps run test:runtime`
- ✅ `corepack pnpm -C apps run build`
