# 2026-06-06 配额来源刷新显式范围保护

## 状态

✅ 已完成

## 背景

`quota/refreshSources` 的旧行为会把缺省 `sourceIds` 解析为空数组，而服务端空集合表示不过滤。裸 RPC 或旧脚本调用会同时遍历聚合 API 与 OpenAI 账号来源，并触发余额查询或账号用量刷新。在几千账号场景下，这会放大 CPU、网络请求、上游超时和 RPC 超时重试风险。

## 修改

- `QuotaRefreshSourcesInput` 增加 `refreshAll` 显式全量开关。
- `quota/refreshSources` 默认要求提供 `sourceIds`，否则立即返回错误。
- 只有传入 `refreshAll=true` 时才允许无 `sourceIds` 的全量刷新。
- 指定 `sourceIds` 时，聚合 API 通过 `find_aggregate_api_by_id` 按 ID 读取，账号通过 `list_accounts_by_ids` 批量读取，避免先全量列表再过滤。
- Web / Tauri / TypeScript 客户端同步透传 `refreshAll`。

## 验证

- 新增纯函数测试覆盖空 `sourceIds` 默认拒绝、`refreshAll=true` 允许、指定 ID 允许。
- 更新 Web 命令映射测试，确认 `refreshAll` 与 `sourceIds/source_ids` 参数透传。
