# 给 Claude/Opus 的执行任务：J 项 usage_snapshots 无变化写入去重

发起方：CodeX-GPT  
任务时间：2026-06-24T16:31:02+08:00
工作目录：`C:\code\CodeX\Codex-Manager-CE`  
目标分支：`hardening/main`

## 背景

`task.md` 的 J 项仍真实存在：生产写入路径 `crates/service/src/usage/usage_snapshot_store.rs::store_usage_snapshot()` 每次刷新都会：

1. `parse_usage_snapshot()`
2. 构造 `UsageSnapshotRecord`
3. `storage.insert_usage_snapshot(&record)`
4. `prune_usage_snapshots_for_account(account_id, retain)`

即使服务端返回的用量关键字段完全没变化，也会 INSERT 新行再 prune。大账号池 + 高频刷新时会造成 `usage_snapshots` 和 WAL 写放大。

## 目标

完成 J 项：用量快照在关键字段未变化时不要 append 新行。

## 约束

1. 不要改变对外用量状态语义：`apply_status_from_snapshot()` 仍应基于本次解析结果执行。
2. 不要因为 skipped insert 而丢失“最近刷新时间”的表达能力。推荐方案是更新最新行 `captured_at`，或者通过等价轻量路径维持 latest 语义。
3. 比较关键字段时应覆盖：
   - `used_percent`
   - `window_minutes`
   - `resets_at`
   - `secondary_used_percent`
   - `secondary_window_minutes`
   - `secondary_resets_at`
   - `credits_json`
4. `credits_json` 比较要避免简单字符串格式差异造成误判。如果当前项目没有规范化 JSON helper，可以使用 `serde_json::Value` 语义比较；若决定做字符串比较，必须解释风险并补测试覆盖稳定序列化。
5. 不要 `git add .`；只暂存本任务相关文件。
6. 新增注释必须使用简体中文。
7. 不要把密钥、cookie、token、身份证号、手机号写入协作文件。

## 推荐实现方向

小步方案：

1. 在 storage 层新增一个方法，例如：
   - `update_latest_usage_snapshot_captured_at_for_account(account_id, captured_at) -> Result<bool/usize>`
   - 或 `upsert_usage_snapshot_if_changed(&record) -> Result<UsageSnapshotStoreOutcome>`
2. 在 service 层 `store_usage_snapshot()` 中读取 `latest_usage_snapshot_for_account(account_id)`，比较关键字段。
3. 如果字段未变：
   - 不调用 `insert_usage_snapshot()`
   - 更新最新快照 `captured_at` 为本次时间
   - 可跳过 `prune_usage_snapshots_for_account()`，因为没有新增行
   - 仍调用 `apply_status_from_snapshot(storage, &record)`
4. 如果字段变化：
   - 保持原 insert + prune 行为

更优方案：

- 把“比较 + insert/update”尽量收口到 storage，service 只处理解析和状态更新。
- 但不要做大范围表结构迁移，除非你证明必须。

## 必须验证

至少运行并记录：

```powershell
cargo test -p codexmanager-core --lib usage_snapshot
cargo test -p codexmanager-service --lib usage_snapshot
cargo check -p codexmanager-service
```

如果 filter 不匹配导致 `0 tests`，必须换成真实能跑到新增/修改测试的命令，不能把 `0 tests` 当通过。

建议新增测试覆盖：

- 相同关键字段连续 store 两次，`usage_snapshot_count_for_account(account_id)` 不增加。
- 相同关键字段第二次 store 后，最新行 `captured_at` 更新为新时间或 latest 语义能体现新刷新。
- 任一关键字段变化时会新增快照。
- `credits_json` 语义相同但字段顺序不同不应新增快照（如果采用语义比较）。

## 交付要求

完成后写入 `.teamwork/sync/opus-to-gpt.md`，至少包含：

- 修改摘要
- 产出文件
- 关键设计取舍
- 实际运行的验证命令与结果
- git commit hash
- 未验证项或剩余风险

然后将 `.teamwork/sync/status.json` 更新为 `waiting_for_gpt`，`last_actor` 写你的身份。

## 审计提醒

CodeX-GPT 会独立复核：

1. 是否真的避免相同快照 append。
2. 是否保留 latest/captured_at 语义。
3. 是否仍在字段变化时新增记录。
4. 是否没有把测试 fixture 的 `insert_usage_snapshot()` 当生产路径误改。
5. diff 是否只包含 J 项相关文件。
