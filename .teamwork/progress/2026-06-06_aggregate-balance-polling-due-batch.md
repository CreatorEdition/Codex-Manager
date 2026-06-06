# 2026-06-06 聚合 API 余额后台轮询限载

## 背景

- 子代理只读审计确认：`refresh_aggregate_api_balances_for_polling_cycle()` 每轮通过 `list_aggregate_apis()` 全量读取聚合 API，再筛选 active + `balance_query_enabled` 并逐个刷新。
- 在几千聚合 API 或余额查询大量失败时，这会跟随 usage polling 周期重复打上游并写回 `last_balance_*` 字段。

## 变更

- storage 层新增 `list_aggregate_apis_balance_polling_due()`，只返回启用余额查询、active 且到期的聚合 API。
- `aggregate_apis` 新增 `idx_aggregate_apis_balance_due` 索引，辅助后台 due 查询。
- 后台轮询默认每轮最多刷新 20 个聚合 API；成功后默认 3600 秒再刷，失败后默认 21600 秒冷却。
- 新增环境变量：
  - `CODEXMANAGER_AGGREGATE_API_BALANCE_POLL_BATCH_LIMIT`
  - `CODEXMANAGER_AGGREGATE_API_BALANCE_POLL_SUCCESS_INTERVAL_SECS`
  - `CODEXMANAGER_AGGREGATE_API_BALANCE_POLL_FAILURE_COOLDOWN_SECS`

## 验证

- 待执行：`cargo test -p codexmanager-core aggregate_api_balance_polling_due_filters_and_limits_sources`
- 待执行：`cargo test -p codexmanager-service aggregate_api_balance_poll_config_uses_safe_bounds`

## 后续

- 手动刷新仍保持不受后台 due/cooldown 限制。
- 后续如仍有 CPU 峰值，可把聚合 API 余额刷新从 usage polling loop 拆成独立 loop。
