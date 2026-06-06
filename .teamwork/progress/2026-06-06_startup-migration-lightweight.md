# 2026-06-06 启动迁移轻量化

## 背景

- 运行版只读诊断显示 `codexmanager.db` 约 462MB、`codexmanager.db-wal` 约 1.09GB。
- `events` 约 84.8 万行，占用约 285MB；`usage_snapshots` 约 22.5 万行，连同索引占用约 124MB。
- 旧库升级时如果在 `init()` 迁移路径直接执行历史清理、rollup、`VACUUM`，会把 CPU、磁盘 I/O 和 WAL 峰值集中到应用启动阶段。

## 变更

- `062_observability_storage_compaction` 和 `067_observability_retention_compaction` 兼容迁移改为只确保观测相关表/列存在，并记录迁移版本。
- 不再在启动迁移路径执行 request logs / token stats / events / usage snapshots 的历史清理。
- 不再在启动迁移路径执行 `PRAGMA wal_checkpoint(TRUNCATE); VACUUM;`。
- 历史数据清理继续由后台观测维护负责，避免阻塞桌面端启动和 Web RPC 首屏。

## 验证

- 新增迁移回归测试：确认旧观测数据在兼容准备函数执行后仍保留，迁移准备只确保结构、不做清理。

## 后续

- 后台维护仍需要继续观察首次清理大库时的耗时；如仍有尖峰，应改成分批清理并限制每轮删除量。
