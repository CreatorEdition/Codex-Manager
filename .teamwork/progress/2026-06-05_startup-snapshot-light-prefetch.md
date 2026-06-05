# 启动快照轻量预取进度

## 范围

- `startup/snapshot` 新增可选 include 开关：
  - `includeUsageAggregate`
  - `includeTodaySummary`
  - `includeRecentLogs`
  - `includeApiModels`
- 不传 include 参数时保留旧完整快照语义。
- 应用启动预取显式关闭用量聚合、今日摘要和最近日志，避免启动路径触发重聚合 RPC。
- 启动预取使用独立 `prefetch` query key，避免污染首页完整统计快照缓存。
- 账号页和平台 Key 页可继续从轻量预取缓存读取首屏账号/API Key 占位数据。

## 非范围

- 本次不重构首页 `dashboard/adminUsageSummary`。
- 本次不拆 `quota/modelPools` sources 分页。
- 本次不改日志页自身分页查询。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service rpc_startup_snapshot_limits_prefetch_sections_and_returns_totals`
- ✅ `corepack pnpm -C apps run test:runtime`
- ✅ `corepack pnpm -C apps run build`
