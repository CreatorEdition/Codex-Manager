# 模型池来源按需查询进度

## 范围

- 新增 RPC：`quota/modelPoolSources`。
- 支持参数：
  - `sourceKind`: `aggregate_api` 或 `openai_account`，默认 `aggregate_api`。
  - `sourceIds`: 指定来源 ID；空数组明确返回空结果，避免误触发全量扫描。
  - `page` / `pageSize`: 未指定 `sourceIds` 时按页读取，`pageSize` 上限为 500。
- 聚合 API 来源查询使用当前页/指定 ID 的 assignment 装饰，不再通过 `quota/modelPools(includeSources:true)` 构造模型池全量 sources。
- 聚合 API 管理页改为按 API ID 调用 `modelPoolSources`，用返回的每来源一条明细展示额度模型和折算 token。

## 非范围

- 本次不完整重构 `aggregateApi/list` 后端分页；该列表仍需下一条独立提交处理。
- 本次不重构 `dashboard/adminUsageSummary`；该首页慢路径已由子代理确认并记录到 `task.md`。
- 本次不移除旧 `quota/modelPools` sources 兼容能力。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service rpc_quota_model_pool_sources_filters_and_pages_sources`
- ✅ `corepack pnpm -C apps run test:runtime`
- ✅ `corepack pnpm -C apps run build`
- ✅ `git diff --check`
