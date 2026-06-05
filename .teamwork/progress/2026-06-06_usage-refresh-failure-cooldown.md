# 2026-06-06 后台用量轮询失败账号冷却

## 负责人

- 【CodeX-GPT】

## 背景

- 用户反馈几千账号场景下 CPU 长期偏高，运行库 `events` / `usage_snapshots` / WAL 膨胀明显。
- 前序提交已把 `usage_refresh_failed` 写入节流窗口提高到 6 小时，但后台轮询候选仍可能在每轮把最近刚失败的账号继续选入批次。
- 如果大量账号长期 401、限额、网络失败或上游异常，仅降低事件写入不足以降低上游请求和 CPU 消耗。

## 变更

- `usage_refresh_candidate_count()` 与 `list_usage_refresh_candidates_paginated()` 新增失败冷却 cutoff 参数。
- 后台用量轮询使用 `CODEXMANAGER_USAGE_REFRESH_FAILURE_EVENT_WINDOW_SECS` 计算冷却窗口，默认 6 小时。
- 候选 SQL 通过 `events(type, account_id, created_at, id)` 索引跳过窗口内存在 `usage_refresh_failed` 的账号。
- `CODEXMANAGER_USAGE_REFRESH_FAILURE_EVENT_WINDOW_SECS=0` 可关闭冷却；手动“刷新全部账号”不受冷却影响。
- 环境变量目录与中文运行配置文档同步默认值和语义。

## 验证

- ✅ `cargo fmt --all`
- ✅ `cargo test -p codexmanager-core list_usage_refresh_candidates_paginated_skips_recent_failures`
- ✅ `cargo test -p codexmanager-core list_usage_refresh_candidates_paginated_filters_blocked_accounts`
- ✅ `cargo test -p codexmanager-service usage_refresh_failure_cooldown_cutoff_uses_failure_window`
- ✅ `cargo test -p codexmanager-core usage_refresh_candidates`
- ✅ `cargo test -p codexmanager-service usage_refresh`
- ✅ `cargo test -p codexmanager-service app_settings`
- ✅ `cargo fmt --all --check`
- ✅ `git diff --check`
