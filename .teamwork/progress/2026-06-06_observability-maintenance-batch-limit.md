# 2026-06-06 观测维护分批清理

## 背景

- 运行版只读诊断显示 `events` 约 84.8 万行、`usage_snapshots` 约 22.5 万行、WAL 约 1.09GB。
- 启动迁移已经轻量化，但后台维护如果单轮 rollup / delete 全部历史观测数据，仍可能在旧库首次升级后制造 CPU、磁盘 I/O 和 WAL 峰值。

## 变更

- 新增 `CODEXMANAGER_OBSERVABILITY_MAINTENANCE_BATCH_LIMIT`，默认 `5000`，最大按 `100000` 生效。
- `request_token_stats` 过期明细改为按批 rollup + delete。
- `request_logs`、`events`、`usage_snapshots` 保留清理改为每轮按批删除。
- 如果本轮 token stats rollup 达到批量上限，说明旧明细尚未滚完，本轮暂不删除 request logs，避免日志先删而 token 统计还未归档。

## 验证

- 新增 token stats 分批 rollup 测试。
- 新增观测维护在 token stats 未滚完时延迟删除 request logs 的测试。
- 新增 events 与 usage snapshots 分批清理测试。

## 后续

- 继续观察大库首次维护耗时；如单轮 5000 行仍偏重，可进一步按表拆独立维护游标或降低默认批量。
