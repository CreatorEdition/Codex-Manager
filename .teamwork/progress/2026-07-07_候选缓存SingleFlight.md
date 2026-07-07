# 候选缓存 Single-Flight

## 状态

✅ 已完成

## 背景

`task.md` 中剩余 P2 性能观察项包含候选缓存 single-flight / stale-while-revalidate。当前候选快照已有 TTL 缓存，但 cache miss 或过期后多个并发请求可能同时查询账号候选、patch 账号 meta，并批量读取 usage snapshots。

## 处理内容

- 在候选快照缓存 miss 重建阶段加入 single-flight 协调。
- 同一缓存窗口内只允许一个线程执行候选池重建，其他线程等待刷新结束后重新读取缓存。
- `CODEXMANAGER_CANDIDATE_CACHE_TTL_MS=0` 时保持原有无缓存语义，不启用 single-flight。
- 重建失败时会释放等待线程，后续请求可按原逻辑继续重试。

## 暂不处理

- `stale-while-revalidate` 未直接启用。原因是候选快照包含低额度、封禁与账号状态信号，SWR 可能延长旧快照使用窗口；需要单独设计状态变更失效优先级后再做。

## 验证

已通过：
- `cargo test -p codexmanager-service candidate_cache_refresh_is_single_flight_per_cache_window`
- `cargo test -p codexmanager-service gateway::selection::tests::`

备注：由于默认 `target/debug/deps/gateway_logs-*.exe` 有旧进程锁，验证命令使用独立 `CARGO_TARGET_DIR=target\codex-test-json-parse` 执行。
