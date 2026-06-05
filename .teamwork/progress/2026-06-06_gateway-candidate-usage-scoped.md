# 2026-06-06 网关候选配额保护按候选读取用量

## 负责人

- 【CodeX-GPT】
- 旁路只读审计：【Rawls】已派发，等待回报

## 背景

- 用户反馈运行版 `D:\Apps\CodexManager` 数据库约 458MB、WAL 约 1.09GB，XPS15 i7 CPU 常驻约 30%，峰值可到 60%。
- 源码审计发现 `collect_gateway_candidates_uncached()` 在候选缓存失效后会调用 `apply_quota_guard()`。
- 原 `load_usage_snapshots()` 使用 `latest_usage_snapshots_by_account()` 读取全部账号最新 usage 快照，再只按候选账号 ID 判断低额度。

## 变更

- quota guard 改为 `load_usage_snapshots_for_candidates()`，只读取当前候选账号 ID 对应的最新 usage 快照。
- 账号 ID 查询按 500 个一批执行，避免几千账号时撞 SQLite 绑定变量上限。
- 保留旧容错语义：usage 快照读取失败时记录 warning 并跳过本轮配额过滤，不阻断网关候选返回。
- 不改变候选排序、低额度分组、低额度 fallback、模型路由或 failover 语义。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service quota_guard_usage_lookup_scopes_to_candidate_accounts`
- ✅ `cargo test -p codexmanager-service selection`
- ✅ `git diff --check`
