# 2026-06-06 Token 用量按 Key 聚合下推过滤

## 负责人

- 【CodeX-GPT】
- 旁路只读审计来源：【Rawls】

## 背景

- Rawls 只读审计指出 `summarize_request_token_stats_by_key_ids()` 与 `summarize_request_token_stats_by_key_ids_and_model()` 在 include rollups 模式下先 UNION `request_token_stats` 与 `request_token_stat_rollups`，再在外层按 keyIds 过滤。
- 平台 Key 用量页和成员仪表盘只需要当前页/当前成员 Key 的统计，在大库下不应扫描全量 token_stats/rollups。

## 变更

- 新增 `repeated_sqlite_text_params()`，支持同一批 keyIds 参数绑定到两个 UNION 分支。
- `summarize_request_token_stats_by_key_ids()` 在热表与 rollup 分支内先 `WHERE key_id IN (...)`。
- `summarize_request_token_stats_by_key_ids_and_model()` 的 include rollups 分支同样先按 keyIds 下推过滤。
- 保留外层非空 key 过滤与原有排序/聚合口径。
- 新增回归测试覆盖热表 + rollup 合并统计，并确认未请求 Key 不会返回。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-core request_token_stats_key_id_summaries_merge_rollups_and_filter_keys`
- ✅ `cargo test -p codexmanager-core request_logs`
- ✅ `cargo test -p codexmanager-service usage_stats`
- ✅ `cargo test -p codexmanager-service member_summary`
- ✅ `git diff --check`
