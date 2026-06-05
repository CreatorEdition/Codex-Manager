# 2026-06-06 网关无候选诊断限载

## 负责人

- 【CodeX-GPT】

## 背景

- 用户反馈几千账号场景下 RPC 超时、数据库/WAL 膨胀、CPU 占用异常。
- 前序提交已降低网关候选查询、quota guard、用量刷新失败事件和 usage 快照维护成本。
- `log_no_candidates()` 仍在无候选异常分支执行 `list_accounts()`、`list_tokens()`、`latest_usage_snapshots_by_account()` 并逐账号写 warn。
- 该分支平时不应频繁触发，但一旦候选池失效或所有账号被过滤，会把故障放大成全库扫描和大量日志写入。

## 变更

- 新增 `Storage::usage_snapshot_count()`，用于诊断摘要只读取总行数。
- 新增无候选诊断 helper：读取账号总数、Token 总数、usage 快照总数，并只采样前 12 个账号。
- 采样账号的 Token 和 usage 只按样本账号 ID 批量查询，不再读取全量 Token 或全量最新 usage 快照。
- 日志摘要增加 `sample_limit`、`sampled_accounts`、`truncated_accounts`，保留排障可见性。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-core usage_snapshot_count_returns_total_rows`
- ✅ `cargo test -p codexmanager-service no_candidate_diagnostic_samples_accounts_without_full_detail_load`
- ✅ `cargo test -p codexmanager-service selection`
- ✅ `git diff --check`
