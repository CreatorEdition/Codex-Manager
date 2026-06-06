# 2026-06-06 配额概览 SQL 汇总

## 角色

- 【CodeX-GPT】

## 背景

`quota/overview` 旧实现会全量读取 API Key、quota limits、按 Key 用量、聚合 API、账号和最新 usage snapshots，再在 Rust 层构造 HashMap 并聚合。几千账号/来源和大 usage 表场景下，该 RPC 会造成不必要对象搬运和 CPU 峰值。

## 变更

- 新增 storage 层 API Key 配额概览汇总 SQL，直接返回 Key 数、限额 Key 数、总限额、总用量、剩余额和费用。
- 新增 storage 层聚合 API 概览汇总 SQL，直接统计来源数、余额查询启用数、成功/失败状态、余额合计和最后刷新时间。
- 新增 storage 层 OpenAI 账号概览汇总 SQL，按账号索引读取最新用量并统计账号数、可用数、低配额数、平均剩余百分比和最后刷新时间。
- `quota/overview` 响应结构不变，只改为装配 storage 汇总结果。

## 验证

- 已通过 `cargo fmt --all --check`
- 已通过 `cargo test -p codexmanager-core quota_api_key_overview_summary_aggregates_limits_and_usage`
- 已通过 `cargo test -p codexmanager-core quota_aggregate_api_overview_summary_parses_balance_and_status`
- 已通过 `cargo test -p codexmanager-core quota_openai_account_overview_summary_uses_latest_usage_per_account`
- 已通过 `cargo test -p codexmanager-core overview_summary`
- 已执行 `cargo test -p codexmanager-service quota_overview`，当前无匹配用例，用于确认 service 测试目标可编译
- 已通过 `cargo check -p codexmanager-service`
- 已通过 `git diff --check`
