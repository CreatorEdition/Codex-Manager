# J 项 usage_snapshots 无变化写入去重结果

执行身份：CodeX-GPT
接手时间：2026-06-24T17:28:09+08:00
任务：`usage-snapshot-dedup-j`

## 接手原因

Claude/Opus 执行方完成了核心半成品，但未完成协议交付、未确认 service 测试、未运行 `cargo check`、未写协作结果、未更新状态、未提交，并带入了多处 unrelated rustfmt/行尾噪声。CodeX-GPT 已关闭执行方、清理无关 diff 并完成审计收口。

## 修改摘要

- storage 层新增 `update_latest_usage_snapshot_captured_at_for_account(account_id, captured_at)`，用于在相同快照去重时维护最新刷新时间语义。
- service 层 `store_usage_snapshot()` 先读取账号最新快照，比较关键字段：
  - `used_percent`
  - `window_minutes`
  - `resets_at`
  - `secondary_used_percent`
  - `secondary_window_minutes`
  - `secondary_resets_at`
  - `credits_json`
- 如果关键字段未变化：不再 `insert_usage_snapshot()`，只更新最新行 `captured_at`，并继续基于本次解析结果执行 `apply_status_from_snapshot()`。
- 如果关键字段变化：保持原 insert + prune 行为。
- `credits_json` 使用 `serde_json::Value` 语义比较，避免对象字段顺序不同导致误判为变化。

## 产出文件

- `crates/core/src/storage/usage.rs`
- `crates/service/src/usage/usage_snapshot_store.rs`
- `task.md`
- `.teamwork/sync/opus-to-gpt.md`
- `.teamwork/sync/status.json`

## 验证结果

- `cargo test -p codexmanager-core --lib usage_snapshot -- --nocapture` -> 6 passed。
- `cargo test -p codexmanager-service --lib usage_snapshot -- --nocapture` -> 5 passed。
- `cargo check -p codexmanager-service` -> Finished（仅既有 warning）。
- `git diff --check` -> 通过。
- `rustfmt --edition 2021 crates/core/src/storage/usage.rs crates/service/src/usage/usage_snapshot_store.rs` -> 通过。

## 审计结论

- 相同关键字段连续写入不会增加 `usage_snapshots` 行数。
- 相同关键字段第二次写入会更新 latest `captured_at`。
- 任一关键字段变化仍新增快照。
- `credits_json` 语义相同但字段顺序不同不会新增快照。
- 执行方误带入的 unrelated rustfmt/行尾噪声已清理，未纳入提交。

## 剩余风险

- latest 快照的比较与更新时间是两步操作，极端同账号并发刷新下仍可能存在竞态窗口；当前 token/usage 刷新路径已有批次调度和账号级刷新语义，风险可接受。若后续观测到同账号并发写入，可进一步把 compare/update/insert 收口到 storage transaction。
