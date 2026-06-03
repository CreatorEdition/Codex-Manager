# 2026-06-04 账号列表后端装饰优化进度

## 来源

子代理 D 初稿 + 主窗口【CodeX-GPT】补齐实现与审计。

## 已处理范围

- 新增按账号 ID 批量读取的 storage helper：
  - `list_tokens_by_account_ids`
  - `latest_usage_snapshots_by_account_ids`
  - `list_account_metadata_by_account_ids`
  - `list_account_subscriptions_by_account_ids`
  - `list_quota_source_model_assignments_for_source_ids`
  - `list_account_quota_capacity_overrides_by_account_ids`
- `account_list::to_account_summaries` 改为只读取当前 accounts 参数对应的装饰数据。
- 新增 core storage 回归测试，确认 scoped helpers 不返回未请求账号的数据，并保持 usage 最新快照语义。

## 验证

- 已执行 `cargo test -p codexmanager-core storage_account_scoped_list_helpers_only_return_requested_ids`，通过。
- 已执行 `cargo test -p codexmanager-service rpc_account_list`，6 个账号列表 RPC 测试通过。
- 已执行 `cargo check -p codexmanager-service`，通过。
- 已执行 `cargo fmt --all --check`，通过。
