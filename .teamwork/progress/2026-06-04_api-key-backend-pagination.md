# API Key 后端分页进度

## 范围

- 新增 `apikey/list` 后端分页参数：`page`、`pageSize`、`query`、`statusFilter`。
- 新增分页返回字段：`total`、`page`、`pageSize`，保留 `items` 兼容旧调用。
- 将 API Key 搜索、状态筛选、成员 owner 过滤下推到 SQLite 查询。
- API Key quota limit 装饰改为按当前结果 key IDs 批量读取。
- 新增 API Key 列表排序、状态筛选和 owner join 索引。

## 非范围

- 本次不改前端 API Key 页面分页控件。
- 本次不改启动快照 `api_keys` 全量返回。
- 本次不改日志页 API Key lookup。

## 风险记录

- 无参数 `apikey/list` 仍返回全量，供现有前端和启动快照兼容；真正消除大表启动成本需要后续单独提交。
- `statusFilter=enabled` 被规范为 `active`，`disabled` 保持原值，未知状态回退为不过滤。

## 验证

- 通过 `cargo fmt --all --check`
- 通过 `cargo test -p codexmanager-core storage_api_key_list_supports_backend_pagination_filters_and_scoped_quota`
- 通过 `cargo test -p codexmanager-core api_key_list_index_migration_adds_pagination_and_owner_indexes`
- 通过 `cargo test -p codexmanager-service member_api_key_list_supports_backend_pagination_and_filters`
- 通过 `cargo test -p codexmanager-service member_cannot_read_or_mutate_other_user_api_key`
- 通过 `cargo test -p codexmanager-service member_created_api_key_ignores_admin_only_routing_fields`
- 通过 `cargo test -p codexmanager-service rpc_account_list`
- 通过 `cargo check -p codexmanager-service`
