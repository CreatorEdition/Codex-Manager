# 2026-06-05 用量列表裸调用限载

## 负责人

- 【CodeX-GPT】

## 背景

- 前端账号页已按当前页账号 ID 调用 `account/usage/list`，但裸 RPC 或旧调用仍会无参读取全部账号最新用量。
- 运行版 `usage_snapshots` 约 22.5 万行，无参接口不应继续把全库最新快照搬到 `/api/rpc` 响应里。

## 变更

- 新增 `latest_usage_snapshots_by_account_limited(limit)`，用于兼容性无参读取。
- `account/usage/list` 无 `accountIds` 时默认 `limit=100`，显式 `limit` 限制在 `0..=500`。
- 有 `accountIds` 的路径保持精确按账号 ID 查询，不受默认 limit 影响。
- RPC 返回结构保持 `{ items }`，避免这次提交混入前端契约调整。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-core latest_usage_snapshots_by_account_limited_returns_recent_unique_accounts`
- ✅ `cargo test -p codexmanager-service normalize_usage_list_limit_clamps_unscoped_reads`
- ✅ `cargo test -p codexmanager-service rpc_usage_list_unscoped_respects_limit`
- ✅ `git diff --check`
