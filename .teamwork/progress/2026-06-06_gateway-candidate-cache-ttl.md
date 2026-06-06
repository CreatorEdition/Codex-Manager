# 2026-06-06 网关候选缓存默认 TTL 延长

## 角色

- 【CodeX-GPT】

## 背景

运行版大库已经出现 `events`、`usage_snapshots` 与 WAL 膨胀，网关热路径虽然已减少全量扫描，但候选缓存默认只有 500ms。几千账号且高 RPS 时，缓存频繁失效会持续触发候选账号读取、最新用量读取和配额保护计算。

## 变更

- 将 `CODEXMANAGER_CANDIDATE_CACHE_TTL_MS` 默认值从 500ms 提高到 5000ms。
- 保留账号状态变化、配额用尽标记和配置重载时的主动缓存失效。
- 新增默认 TTL 回归测试，避免未来回退到亚秒级默认值。
- 更新环境变量文档和 `task.md`。

## 验证

- 已通过 `cargo fmt --all --check`
- 已通过 `cargo test -p codexmanager-service default_candidate_cache_ttl_avoids_subsecond_rebuilds`
- 已通过 `cargo test -p codexmanager-service candidate_snapshot_cache_reuses_recent_snapshot`
- 已通过 `git diff --check`
