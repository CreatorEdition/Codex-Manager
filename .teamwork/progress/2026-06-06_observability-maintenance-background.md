# 2026-06-06 观测维护后台化

## 背景

- 运行版 `D:\Apps\CodexManager` 只读诊断显示 `codexmanager.db` 约 460MB、`codexmanager.db-wal` 约 1.09GB，用户反馈 XPS15 i7 上 CPU 常驻 30% 甚至峰值 60%。
- 网关请求日志写入后会同步调用 `maybe_run_observability_maintenance(created_at)`。
- 该维护函数会执行 token stats rollup、请求日志清理、events 清理、usage snapshots 剪枝，并在有变更时执行 `PRAGMA wal_checkpoint(TRUNCATE);`。

## 变更

- 新增 `gateway/observability/maintenance.rs`，把观测维护改为后台线程执行。
- 请求线程只做原子调度，不再同步执行 rollup、剪枝和 WAL checkpoint。
- 调度状态使用 `AtomicI64` 与 `AtomicBool` 控制：
  - 默认 `900` 秒窗口内只调度一次；
  - 后台任务运行期间不重复调度；
  - 后台失败或线程创建失败时恢复上次调度时间，避免维护永久停摆。
- 后台线程通过 `storage_helpers::open_storage()` 获取独立 storage handle，再调用 `prune_observability_history(now)`。
- 环境变量文档补充观测维护间隔与日志/统计/快照保留策略。

## 验证

- `cargo fmt --all --check`
- `cargo test -p codexmanager-service gateway::maintenance::tests`
- `cargo test -p codexmanager-service request_log`
- `cargo test -p codexmanager-core observability`

## 风险与后续

- 本次只改变维护执行时机，不改变保留策略本身。
- 运行版需要重新构建替换可执行文件后才会生效。
- 后续仍需继续审计 `request_logs` 留存策略与线上 WAL 收缩效果。
