# 管理员用量排行限载进度

## 范围

- `dashboard/adminUsageSummary` 新增 `rankingLimit` 参数。
- 未传 `rankingLimit` 时默认返回 Top 8 用户、OpenAI 账号和聚合 API。
- `rankingLimit:0` 可返回空排行；`rankingLimit<0` 保留旧的完整排行语义，供需要全量导出或调试时显式使用。
- 用户排行在限载模式下只按 Top 用户 ID 读取用户信息和钱包。
- OpenAI 账号与聚合 API 排行在限载模式下只按 Top 来源 ID 批量读取元数据。
- 前端首页 hook 默认传 `rankingLimit:8`，Web 命令映射和 Tauri 命令均透传该参数。

## 非范围

- 本次不新增分页 UI。
- 本次不拆 `dashboard/adminOverview`、`dashboard/adminUserUsagePage`、`dashboard/adminSourceUsagePage`。
- 本次不把 TopN 聚合完全下推到 SQL；当前仍会计算区间排行 rollup，但不再返回和装饰全量元数据。
- 本次不处理模型页来源选择器的全量账号/聚合 API 问题。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service admin_usage_summary`
- ✅ `corepack pnpm -C apps run test:runtime`
- ✅ `corepack pnpm -C apps run build`
- ✅ `git diff --check`
