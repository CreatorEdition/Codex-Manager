# 平台 Key 归属按需查询进度

## 范围

- `accountManager/apiKeyOwners/list` 新增可选 `keyIds` 参数。
- `accountManager/users/list` 新增可选 `ids` 参数。
- 无参数调用保留旧全量语义，兼容账号管理页和旧客户端。
- 显式传空数组返回空列表，避免当前页无 Key 时退回全量查询。
- 平台 Key 页只按当前页 Key 查询归属，只按归属 userIds 查询用户展示信息。
- 创建/编辑平台 Key 弹窗打开时才拉完整成员列表，用于选择归属成员。
- `list_api_key_ids_for_user` 改为数据库条件查询，不再全量读取 owner 后内存过滤。

## 非范围

- 本次不重构账号管理页用户列表分页。
- 本次不新增用户搜索/分页下拉。
- 本次不修改钱包、账单规则和模型组的业务语义。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service rpc_account_manager_lists_support_lookup_filters`
- ✅ `corepack pnpm -C apps run test:runtime`
- ✅ `corepack pnpm -C apps run build`
