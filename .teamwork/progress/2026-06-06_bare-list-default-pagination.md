# 2026-06-06 裸列表 RPC 默认分页

## 背景

- `account/list` 与 `apikey/list` 已支持分页，但公共 RPC 只有收到 `page` / `pageSize` 等参数时才走分页。
- 旧前端、脚本或直接 JSON-RPC 裸调用 `{}` 时仍会返回全量账号或 API Key，在几千数据下会造成内存、网络和后端装饰查询放大。

## 变更

- `account/list` 公共 RPC 无参数时默认按 `AccountListParams::default()` 返回第一页，默认 `pageSize=5`。
- `apikey/list` 公共 RPC 无参数时默认按 `ApiKeyListParams::default()` 返回第一页，默认 `pageSize=20`。
- 内部启动快照需要全量时仍可直接调用 `read_accounts(..., false)` 或 `read_api_keys_for_actor()`，避免破坏内部显式全量路径。

## 验证

- `cargo test -p codexmanager-service account_list_bare_rpc_defaults_to_first_page`
- `cargo test -p codexmanager-service member_api_key_list_supports_backend_pagination_and_filters`
- `cargo fmt --all --check`
- `git diff --check`

## 风险

- 直接依赖 `account/list {}` 或 `apikey/list {}` 全量返回的外部脚本需要改为分页拉取，或改用专用 lookup/内部能力。
