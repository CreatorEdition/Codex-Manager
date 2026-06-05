# 2026-06-06 用量快照集合式剪枝

## 负责人

- 【CodeX-GPT】

## 背景

- 运行版只读诊断显示 `usage_snapshots` 是数据库体积主因之一，且 WAL 已超过 1GB。
- 观测数据维护路径会调用 `prune_usage_snapshots_all_accounts()`。
- 原实现先 `SELECT DISTINCT account_id FROM usage_snapshots`，再对每个账号调用一次 `prune_usage_snapshots_for_account()`，几千账号场景会放大后台 CPU、SQL prepare/execute 次数和 WAL 写入。

## 变更

- `prune_usage_snapshots_all_accounts()` 改为单条 SQLite 窗口函数 DELETE。
- 按 `PARTITION BY account_id ORDER BY captured_at DESC, id DESC` 保留每个账号最近 N 条，保持既有最新快照语义。
- 不改变单账号剪枝函数，避免影响刷新单账号后的即时清理路径。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-core prune_usage_snapshots_all_accounts_keeps_recent_rows_per_account`
- ✅ `cargo test -p codexmanager-core usage_snapshot`
- ✅ `git diff --check`
