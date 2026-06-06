# 2026-06-06 成员最近日志降载

## 角色

- 【CodeX-GPT】
- 【Aquinas】只读复核

## 背景

`dashboard/memberSummary` 首页只展示成员最近 8 条请求日志，但旧路径调用分页接口，会先对成员全部 Key 的日志执行 `COUNT`，再读取第一页。大日志表和成员持有大量 Key 时，打开成员概览会被无意义 COUNT 放大。

同时 request logs 的 Key ID 过滤条件使用 `IFNULL(r.key_id, '') IN (...)`，普通 `(key_id, created_at)` 复合索引不能稳定服务于被函数包裹的列。

## 变更

- `dashboard/memberSummary` 最近日志改用非分页读取，只取 `MEMBER_RECENT_LOG_LIMIT` 条，不再计算 total。
- request logs 的 Key ID 条件改为 `r.key_id IN (...)`，保持空 Key 列表返回空结果。
- 新增 query plan 回归测试，确认 `key_id IN (...)` 能匹配 `idx_request_logs_key_id_created_at`。

## 验证

- 已通过 `cargo fmt --all --check`
- 已通过 `cargo test -p codexmanager-core key_in_query_matches_composite_index`
- 已通过 `cargo check -p codexmanager-service`
- 已通过 `git diff --check`
- 说明：`cargo test -p codexmanager-service member_summary` 当前无匹配用例，未作为覆盖性验证记录。
