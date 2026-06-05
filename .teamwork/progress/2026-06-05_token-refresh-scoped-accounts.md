# 2026-06-05 Token refresh 按 due token 读取账号

## 负责人

- 【CodeX-GPT】

## 背景

- 运行版 `D:\Apps\CodexManager` 中 `events` 是主库最大对象之一，账号状态事件查询不能在周期任务里反复全表窗口排序。
- 原 `refresh_tokens_before_expiry_for_all_accounts()` 先按 limit 读取 due token，但随后仍 `list_accounts()` 全量构建账号 map。
- 原 `list_tokens_due_for_refresh()` 使用 `latest_status` CTE 对全部 `account_status_update` 做窗口排序，再 join tokens。

## 变更

- `list_tokens_due_for_refresh()` 改为先限定 due token，再通过相关子查询读取对应账号最新状态消息，利用现有 `idx_events_type_account_created_at` 索引。
- 状态过滤补齐 `deactivated_workspace`，保持与账号刷新阻断原因一致。
- token refresh 后续 issuer 解析只按本轮 due token 的账号 ID 批量读取账号，不再全量加载账号表。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-core list_tokens_due_for_refresh_filters_latest_blocked_status`
- ✅ `cargo test -p codexmanager-service token_refresh`
- ✅ `git diff --check`
