# 给 Opus 的执行任务：T 项候选缓存 Arc 化返工

发起方：CodeX-GPT  
任务时间：2026-06-24  
工作目录：`C:\code\CodeX\Codex-Manager-CE`  
目标分支：`hardening/main`

## 背景

`task.md` 曾把 T 项标记为已完成，但 CodeX-GPT 在 2026-06-23 复核发现当前实现并未真正消除缓存命中后的候选列表深拷贝。

当前关键代码：

- `crates/service/src/gateway/routing/selection.rs`
- `CandidateSnapshotCache.candidates: Arc<Vec<(Account, Token)>>`
- `collect_gateway_candidates_with_low_quota_mode()` 命中缓存时：

```rust
if let Some(cached) = read_candidate_cache(low_quota_mode) {
    return Ok(Arc::unwrap_or_clone(cached));
}
```

由于缓存自身仍持有一个 Arc，`Arc::unwrap_or_clone(cached)` 在热路径几乎总是 clone 整个 `Vec<(Account, Token)>`，没有达到“命中时只 clone Arc”的目标。

## 目标

完成 T 项返工：候选缓存命中路径不得再一次性深拷贝整个 `Vec<(Account, Token)>`。

## 约束

1. 不要盲目改成 `Arc<Vec<_>>` 后又在入口 `unwrap_or_clone`。
2. 保持候选顺序、quota guard、账号计划筛选、模型路由筛选、failover 行为不变。
3. 不要引入全局锁长期持有；缓存读锁/互斥只允许保护快照读取，不得覆盖上游请求执行。
4. 不要 `git add .`；只暂存本任务相关文件。
5. 所有新增注释使用简体中文。
6. 不能把密钥、cookie、token、身份证号、手机号写入协作文件。

## 推荐实现方向

优先考虑小步重构：

1. 将 `collect_gateway_candidates_with_low_quota_mode()` 返回类型从 `Vec<(Account, Token)>` 调整为可共享的候选集合，例如 `Arc<Vec<(Account, Token)>>` 或本地 `CandidateList` 包装类型。
2. 将下游 `prepare_gateway_candidates()` 的过滤逻辑改为在共享候选上按需构造较小结果，避免缓存命中时无条件 clone 全量列表。
3. 若下游最终仍需要拥有 `Vec` 以便 `into_iter()`、重试时修改 token、记录 attempted ids，则只 clone 被保留的候选项，不要在缓存命中入口 clone 全量。
4. 如果你判断返回类型改动扩散过大，可以采用迭代器/索引列表方案，但必须用代码和测试证明命中路径不再全量深拷贝。

## 必须验证

至少运行并记录结果：

```powershell
cargo test -p codexmanager-service --lib gateway::routing::tests -- --nocapture
cargo test -p codexmanager-service --test gateway_logs images::gateway_images_generation_wraps_codex_sse_as_openai_images_json -- --exact --nocapture
cargo check -p codexmanager-service
```

如 `gateway::routing::tests` filter 不匹配，请改用能实际运行 selection/candidates 相关测试的命令，不能把 `0 tests` 当作通过。

## 交付要求

完成后写入 `.teamwork/sync/opus-to-gpt.md`，至少包含：

- 修改摘要
- 产出文件
- 关键设计取舍
- 实际运行的验证命令与结果
- git commit hash
- 未验证项或剩余风险

然后将 `.teamwork/sync/status.json` 更新为：

```json
{
  "status": "waiting_for_gpt",
  "task": "candidate-cache-arc-refactor-t",
  "last_actor": "CodeX-Opus-4.6"
}
```

## 审计提醒

CodeX-GPT 会独立复核，不会直接采信报告。重点会检查：

1. 是否仍存在 `Arc::unwrap_or_clone(cached)` 或等价全量 clone。
2. diff 是否只包含 T 项相关修改。
3. 缓存命中、缓存未命中、account_plan_filter、model filter、low quota fallback 的行为是否保持。
4. 验证命令是否真实运行且不是 `0 tests`。
