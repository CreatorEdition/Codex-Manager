# 聚合 API 后端分页进度

## 范围

- `aggregateApi/list` 新增后端分页与筛选：
  - `page`
  - `pageSize`
  - `query`
  - `providerType`
  - `statusFilter`
- Storage 层新增聚合 API 过滤计数与分页读取，排序保持 `sort ASC, updated_at DESC, id ASC`。
- 聚合 API 管理页改为读取后端分页结果，provider/status/pageSize 下推到后端。
- 模型池来源装饰只按当前页聚合 API ID 调用 `quota/modelPoolSources`。
- 批量测试和批量刷新余额改为当前页范围，避免分页后误操作全部数据。

## 非范围

- 本次不新增聚合 API 文本搜索输入框；后端已预留 `query`。
- 本次不处理模型管理页的聚合 API 名称字典全量读取。
- 本次不拆 `dashboard/adminUsageSummary`。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service rpc_aggregate_api_list_supports_backend_filters_and_pagination`
- ✅ `corepack pnpm -C apps run test:runtime`
- ✅ `corepack pnpm -C apps run build`
- ✅ `git diff --check`
