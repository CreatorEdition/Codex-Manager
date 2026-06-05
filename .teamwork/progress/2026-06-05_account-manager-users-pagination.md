# 2026-06-05 账号体系用户列表分页

## 负责人

- 【CodeX-GPT】

## 背景

- 账号体系管理页直接调用 `appClient.listAppUsers()`，后端无分页时会一次性返回全部 Web 登录用户及钱包信息。
- 在多成员场景下，这类全量 list 会增加 `/api/rpc` 响应体和数据库装饰成本，也会让顶部“可分发成员”在分页改造后容易误读当前页。

## 变更

- `accountManager/users/list` 支持 `page/pageSize`，传分页参数时返回 `items/total/page/pageSize`。
- 保留无参数数组返回和 `ids` lookup 兼容路径，避免影响模型组等旧调用。
- 账号管理页改用 `appClient.listAppUserPage()`，表格增加分页大小和上一页/下一页控制。
- `accountManager/status` 增加 `memberUserCount`，顶部“可分发成员”不再从当前页用户数组推导。
- Tauri 命令补充 `page/page_size` 参数透传，Web 与桌面运行壳行为一致。

## 验证

- 通过 `cargo fmt --all --check`。
- 通过 `cargo test -p codexmanager-service --test rpc rpc_account_manager_lists_support_lookup_filters`。
- 通过 `corepack pnpm -C apps run test:runtime`。
- 通过 `corepack pnpm -C apps run build`。
- 通过 `git diff --check`。
