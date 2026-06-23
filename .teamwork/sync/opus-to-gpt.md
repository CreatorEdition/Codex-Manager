# T 项候选缓存 Arc 化返工结果

执行身份：CodeX-GPT
接手时间：2026-06-24T05:56:18+08:00
任务：`candidate-cache-arc-refactor-t`

## 接手原因

Opus 子代理已开始修改但未完成交付：未跑完整验证、未更新 `task.md`、未写协作结果、未更新 `status.json`、未提交。CodeX-GPT 已关闭该子代理并在主工作区接手实现与审计。

## 修改摘要

- `collect_gateway_candidates_with_low_quota_mode()` 返回共享 `GatewayCandidateSnapshot = Arc<Vec<(Account, Token)>>`，缓存命中直接返回 `Arc`，不再 `Arc::unwrap_or_clone(cached)`。
- `prepare_gateway_candidates()` 在共享快照上执行账号计划筛选与模型筛选，只 clone 通过过滤的候选。
- WebSocket 路由路径改为 `GatewayRoutedCandidates { snapshot, ordered_indices }`，路由策略在索引数组上执行，连接尝试时才逐项 clone，故障切换只 clone 下一个候选。
- `model_picker` 保留低频完整排序所需的显式 `snapshot.as_ref().clone()`，不再使用 `Arc::unwrap_or_clone`。
- 新增索引路由等价测试，并补强缓存复用测试的 `Arc::ptr_eq` 断言。

## 产出文件

- `crates/service/src/gateway/routing/selection.rs`
- `crates/service/src/gateway/upstream/support/candidates.rs`
- `crates/service/src/gateway/routing/route_hint.rs`
- `crates/service/src/gateway/routing/tests/route_hint_tests.rs`
- `crates/service/src/gateway/routing/tests/selection_tests.rs`
- `crates/service/src/gateway/mod.rs`
- `crates/service/src/http/responses_websocket.rs`
- `crates/service/src/gateway/model_picker/mod.rs`
- `task.md`
- `.teamwork/sync/opus-to-gpt.md`
- `.teamwork/sync/status.json`

## 验证结果

- `cargo test -p codexmanager-service --lib candidate_snapshot_cache -- --nocapture` -> 4 passed。
- `cargo test -p codexmanager-service --lib indexed_route_strategy_matches_owned_candidate_order -- --nocapture` -> 1 passed。
- `cargo test -p codexmanager-service --test gateway_logs images::gateway_images_generation_wraps_codex_sse_as_openai_images_json -- --exact --nocapture` -> 1 passed。
- `cargo check -p codexmanager-service` -> Finished（仅既有 warning）。
- `rustfmt --edition 2021` 针对本轮相关 Rust 文件通过。

## 审计结论

- 未发现 `Arc::unwrap_or_clone` 残留。
- 主请求候选准备路径不再在缓存命中入口全量 clone。
- WebSocket 请求/故障切换路径不再预先 clone 全量候选 Vec。
- `aggregate_api.rs` 的 rustfmt 噪声已恢复，不属于本任务。

## 剩余风险

- `model_picker` 仍会完整 clone 候选快照用于排序和遍历，但该路径属于模型拉取/预热/管理行为，不是每个网关请求的候选选择热路径。
- 全仓 `cargo test --workspace` 未执行；本轮只跑了 T 项相关和 images 回归验证。
