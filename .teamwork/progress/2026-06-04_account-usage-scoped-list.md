# 账号页 usage 按需读取进度

## 范围

- `account/usage/list` 新增可选 `accountIds` 参数。
- 无参数调用保持旧行为，继续返回全部账号最新 usage 快照，兼容启动快照、仪表盘和旧调用。
- 显式传空数组返回空列表，避免当前页无账号时误退回全量读取。
- Tauri `service_usage_list` 透传 `accountIds`。
- 前端 `accountClient.listUsage({ accountIds })` 支持按账号 ID 查询。
- `useAccounts()` 从当前账号页结果提取账号 ID，只读取当前页账号 usage，再用现有 `attachUsagesToAccounts` 装饰当前页。
- 新增 RPC 回归测试覆盖按 ID 过滤、去重、空数组不回全量。

## 非范围

- 本次不删除启动快照里的 usage 全量预载。
- 本次不改变 usage 聚合接口；仪表盘和统计页仍可按原全量语义工作。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service rpc_usage_list_filters_requested_account_ids`
- ✅ `corepack pnpm -C apps run test:runtime`
- ✅ `corepack pnpm -C apps run build`
