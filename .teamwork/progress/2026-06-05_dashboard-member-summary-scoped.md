# 2026-06-05 成员仪表盘按归属 Key 聚合

## 负责人

- 【CodeX-GPT】

## 背景

- `dashboard/memberSummary` 旧实现先读取成员名下 Key ID，但随后仍全量读取平台 Key 列表并本地过滤。
- 成员用量排行还会全量聚合所有 Key/模型用量，再按成员 Key ID 本地过滤；几千账号和大量日志场景下会放大 `/api/rpc` 的 10 秒超时风险。

## 变更

- `crates/service/src/apikey/apikey_list.rs` 新增按 ID 批量读取 Key 元数据的内部 helper，并移除已无调用的裸全量 `read_api_keys`。
- `crates/core/src/storage/request_token_stats.rs` 新增 `summarize_request_token_stats_by_key_ids_and_model`，支持按 Key ID 范围聚合模型用量。
- `crates/service/src/dashboard.rs` 将成员 summary 的 Key 元数据、今日用量、累计 Key 用量和 7 日模型用量都限制在成员归属 Key ID 内，并以 250 个 Key 为一批规避 SQLite 参数上限。
- `crates/service/src/tests/lib_tests.rs` 增加成员模型排行隔离断言，并覆盖 251 个成员 Key 跨批次计数。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service member_dashboard`
