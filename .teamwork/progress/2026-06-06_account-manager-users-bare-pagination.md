# 2026-06-06 账号体系用户列表裸调用默认分页

## 角色

- 【CodeX-GPT】

## 背景

`accountManager/users/list` 虽然已经支持 `page/pageSize` 和 `ids` lookup，但裸调用仍回退到 `list_app_users()` 全量读取，并逐用户查询钱包。API Key 弹窗和模型组页面仍通过旧 `listAppUsers()` 入口触发无参调用。

## 变更

- `accountManager/users/list` 无 `ids/page/pageSize` 时默认走分页接口，返回第一页 20 条。
- 前端 `appClient.listAppUsers()` 兼容数组与分页对象，读取分页对象的 `items`。
- `ids` lookup 路径保持数组返回，不影响当前页 owner lookup。

## 验证

- 已通过 `cargo fmt --all --check`
- 已通过 `cargo test -p codexmanager-service account_manager_users_list_bare_call_defaults_to_page`
- 已通过 `cargo test -p codexmanager-service member_actor_cannot_call_admin_only_rpc`
- 已通过 `cargo test -p codexmanager-service password_mode_can_call_admin_and_model_source_rpcs`
- 已通过 `git diff --check`
- 未执行前端 `pnpm` 校验：当前 PowerShell PATH 中没有 `pnpm`

## 后续

- API Key owner 选择器和模型组成员分配仍应改为搜索/分页选择器；本提交先阻断裸 RPC 全量读取。
