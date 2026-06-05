# 2026-06-05 观测数据保留与 WAL 截断

## 负责人

- 【CodeX-GPT】

## 背景

- 运行版 `D:\Apps\CodexManager` 主库约 455MB，WAL 约 717MB。
- 只读 `dbstat` 显示主库体积主要来自 `events` 与 `usage_snapshots`，其中 `usage_refresh_failed` 事件约 81.9 万条。
- 请求日志本体约 6.8MB，不是当前主库膨胀主因。

## 变更

- 新增 `CODEXMANAGER_EVENTS_RETENTION_DAYS`，默认保留 14 天高频事件；账号状态事件保留，避免破坏状态原因判断。
- 维护任务会同时清理过期 events、过量 usage snapshots、过期 request logs 和 request token stats。
- 维护任务在实际删除/rollup 后执行 `PRAGMA wal_checkpoint(TRUNCATE)`，避免连续运行时 WAL 长期不截断。
- 新增 events 时间/类型索引，降低状态原因查询和保留清理的扫描成本。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-core events`
- ✅ `cargo test -p codexmanager-core observability_storage_compaction`
- ✅ `cargo test -p codexmanager-core init_tracks_schema_migrations_and_is_idempotent`
- ✅ `git diff --check`
