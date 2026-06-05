# RPC 默认轻量化防护进度

## 范围

- `startup/snapshot` 无 `accountLimit` / `apiKeyLimit` 参数时默认只预载 20 条账号和 20 条平台 Key。
- `startup/snapshot` 无 include 参数时默认关闭：
  - `includeUsageAggregate`
  - `includeTodaySummary`
  - `includeRecentLogs`
  - `includeApiModels`
- 前端 `serviceClient.getStartupSnapshot()` 显式传完整首页语义，避免后端默认变化导致首页统计静默丢失。
- 启动预取补充 `includeApiModels:false`，避免轻量预取仍读取模型目录。
- `quota/modelPools` 无参数时默认不返回 `sources` 和容量配置；需要完整来源明细时必须显式 `includeSources:true`，需要容量配置时必须显式 `includeConfig:true`。

## 背景

用户抓包显示外部/旧前端可直接发起仅包含 `addr` 的 RPC：

- `quota/modelPools`
- `startup/snapshot`

旧兼容语义是“无参数即完整快照/完整模型池”，在几千账号和大量日志场景会触发全量账号、API Key、usage、日志聚合和模型池来源构建，最终表现为 10 秒左右客户端中断。

## 非范围

- 本次不拆 `dashboard/adminUsageSummary`；该首页慢路径仍需单独提交处理。
- 本次不重写 `quota/modelPools` 汇总 SQL；默认仅减少返回明细和配置装饰，来源遍历优化留待后续 `quota/modelPoolSummary`。
- 本次不新增 `startup/bootstrap`；后续可用独立 RPC 替代首页完整 snapshot。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service --test rpc rpc_startup_snapshot_limits_prefetch_sections_and_returns_totals`
- ✅ `cargo test -p codexmanager-service --test rpc rpc_startup_snapshot_defaults_to_bounded_light_payload`
- ✅ `cargo test -p codexmanager-service --test rpc rpc_quota_model_pools_supports_lightweight_source_filters`
- ✅ `cargo test -p codexmanager-service --test rpc rpc_quota_model_pools_defaults_to_summary_without_sources_or_config`
- ✅ `corepack pnpm -C apps run test:runtime`
- ✅ `corepack pnpm -C apps run build`
- ✅ `git diff --check`
