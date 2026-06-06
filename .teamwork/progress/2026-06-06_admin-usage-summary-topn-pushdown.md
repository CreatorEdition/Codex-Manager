# 2026-06-06 管理员用量排行 SQL 下推

## 背景

- `dashboard/adminUsageSummary` 默认只返回 Top 8 排行，但此前仍分别读取今日和区间的完整用户、OpenAI 账号、聚合 API 分组结果。
- 几千账号场景下，响应体已被 TopN 限制，但 Rust 层仍要构造完整 HashMap、合并 ID 集合并排序，首页打开时会放大 CPU 峰值。

## 变更

- storage 层新增用户与来源的双窗口排行查询，同时聚合今日和区间用量，并在 SQL 内按 `today total_tokens DESC, range total_tokens DESC, id ASC` 排序后 `LIMIT`。
- `dashboard/adminUsageSummary` 默认 TopN 路径改用新排行查询，只按 TopN ID 读取用户、钱包、账号和聚合 API 元数据。
- 显式 `rankingLimit < 0` 的全量兼容路径继续使用旧聚合方法，避免破坏已有全量读取语义。

## 验证

- `cargo test -p codexmanager-core request_token_stats_rollups_use_owner_and_actual_source_precedence`
- `cargo test -p codexmanager-service admin_usage_summary`

## 后续

- 当前 SQL 下推减少 Rust 侧全量合并排序和元数据读取，但数据库仍需扫描请求日志窗口做分组聚合。
- 后续应考虑为首页排行建立日级 rollup 或分页排行 RPC，进一步降低大库首页查询成本。
