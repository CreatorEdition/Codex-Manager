# 2026-06-05 用量轮询候选按批次读取

## 负责人

- 【CodeX-GPT】

## 背景

- 运行版 `D:\Apps\CodexManager` 已有 3028 个账号，`events` 约 83.7 万行，`usage_refresh_failed` 约 82 万行。
- 原 `refresh_usage_for_polling_batch()` 每轮先 `list_accounts()`、`list_tokens()`，再对所有账号读取最新状态原因，最后只取默认 100 个任务执行。
- 在 30 秒轮询间隔下，该路径会把后台任务成本固定成 O(账号总数 + 状态事件扫描)，容易放大 CPU 占用并拖慢 `/api/rpc`。

## 变更

- 新增 storage 层 `usage_refresh_candidate_count()` 和 `list_usage_refresh_candidates_paginated()`。
- 用 SQL 过滤禁用/封禁账号、空 refresh token、以及最新状态原因为账号停用、workspace 停用或地区阻断的账号。
- `refresh_usage_for_polling_batch()` 改为按游标读取本轮候选，尾部不足时最多再从头部补一段，不再构造全量任务数组。
- 手动 `refresh_usage_for_all_accounts()` 保留原显式全量刷新语义，避免混改交互入口。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-core list_usage_refresh_candidates_paginated_filters_blocked_accounts`
- ✅ `cargo test -p codexmanager-service usage_poll`
- ✅ `cargo test -p codexmanager-service usage_refresh`
- ✅ `git diff --check`
