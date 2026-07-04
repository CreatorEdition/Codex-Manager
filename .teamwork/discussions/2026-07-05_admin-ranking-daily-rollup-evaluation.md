# 管理员排行迁移到日级 rollup 评估

时间：2026-07-05
执行：CodeX-GPT

## 结论

不应在当前状态下直接把 `dashboard/adminUsageSummary` 的用户、OpenAI 账号和聚合 API 排行切换到 `request_token_stat_daily_rollups`。

原因是当前仓库虽然已经有日级 rollup 表结构和插入/查询辅助函数，但没有生产维护链路写入该表。直接切换会让历史区间排行返回空数据或与实时明细不一致。

置信度：高。评估基于当前工作树代码路径检索和关键函数读取。

## 证据

- `dashboard/adminUsageSummary` 当前仍调用 core storage 的实时聚合：
  - `summarize_request_token_stats_daily`
  - `summarize_request_token_stats_user_ranking_between`
  - `summarize_request_token_stats_source_ranking_between`
- 上述聚合当前直接扫描 `request_token_stats` / `request_logs`，没有读取 `request_token_stat_daily_rollups`。
- `request_token_stat_daily_rollups` 表存在，migration `074_request_token_stat_daily_rollups.sql` 明确设计为“缓存已结束日期的统计数据，当前日数据仍从 live 表查询”。
- core 只有：
  - `insert_request_token_stat_daily_rollup`
  - `query_request_token_stat_daily_rollups`
  - 相关表结构测试
- 实际观测维护链路 `Storage::prune_observability_history()` 只调用 `rollup_request_token_stats_before_limited()`，写入的是旧的 `request_token_stat_rollups`，不是日级 rollup 表。
- service 后台维护 `schedule_observability_maintenance()` 最终只调用 `storage.prune_observability_history(now)`，因此不会产生日级 rollup 数据。

## 推荐实施顺序

1. 先补 core 层日级 rollup 生产函数，例如按 `request_token_stats.created_at` 的本地日边界聚合已结束日期，写入 `request_token_stat_daily_rollups`。
2. 将观测维护任务接入日级 rollup 生产，并确保只处理已结束日期，避免当前日与 live 表双算。
3. 为 dashboard 增加“日级 rollup + 当前未结束区间 live stats”的混合查询函数。
4. 最后再把 `dashboard/adminUsageSummary` 的排行和趋势切到混合查询，并保留回归测试证明：
   - 历史日来自日级 rollup；
   - 当前日来自 live stats；
   - 跨日区间不会重复计算；
   - user/source 维度排行与现有实时查询结果一致。

## 本轮处理决定

本轮只完成迁移评估和风险收敛，不做半成品代码切换。后续任务应改为“实现日级 rollup 生产链路与 dashboard 混合查询”，而不是继续把“评估”作为未完成项。
