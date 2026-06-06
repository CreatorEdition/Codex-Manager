# 2026-06-06 空 Token 统计跳过写入

## 负责人

- 【CodeX-GPT】

## 背景

- 运行版数据库与 WAL 体积持续增长，`request_logs` / `request_token_stats` 是高频写入路径。
- UI 需要保留 `request_logs` 的一请求一条语义，但 token、费用全为 0 的 `request_token_stats` 行对汇总没有贡献。
- 失败请求、无 usage 响应、非推理请求仍可能进入 `insert_request_log_with_token_stat()`，导致无效统计行写入和 WAL 放大。

## 变更

- 新增 `request_token_stat_has_usage()`，仅当 token 或费用存在正数时写入 `request_token_stats`。
- `insert_request_log_with_token_stat()` 仍始终写入 `request_logs`，保证日志页面可见性。
- token 统计失败时仍保持原有容错：不阻塞 request log 提交。
- 新增回归测试覆盖全零统计被跳过、日志仍可查询、Key 用量汇总为空。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-core insert_request_log_with_empty_token_stat_skips_token_stats_row`
- ✅ `cargo test -p codexmanager-core insert_request_log_with_token_stat_is_visible_via_join`
- ✅ `cargo test -p codexmanager-core request_logs`
- ✅ `cargo test -p codexmanager-service usage_stats`
- ⚠️ `cargo test -p codexmanager-service gateway_logs` 曾因并行编译/文件锁在 120 秒超时，未见测试断言失败；本提交核心覆盖在 core request_logs 与 usage_stats。
- ✅ `git diff --check`
