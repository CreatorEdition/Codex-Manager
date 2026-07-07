# Compact Transport JSON 解析收敛

## 状态

✅ 已完成

## 背景

请求体 JSON parse 深水区继续观察中，compact transport 路径此前会先解析 body 剥离 `service_tier`，随后再次解析 body 提取 `prompt_cache_key` 参与 session affinity。

## 处理内容

- 新增 compact transport body 准备 helper，同时返回处理后的 body 与 `prompt_cache_key`。
- 在需要剥离 `service_tier` 的 compact 请求路径中复用同一次 JSON parse。
- 保持旧限制：超过 64 KiB 的请求体不提取 `prompt_cache_key`，避免改变大请求体 affinity 语义。
- 保留旧 `strip_compact_service_tier_for_transport` 测试入口，验证原有剥离/保留行为不回归。

## 验证

已通过：
- `cargo test -p codexmanager-service compact_transport_body_prepares_prompt_cache_key_with_single_parse_path`
- `cargo test -p codexmanager-service compact_transport_strips_service_tier_without_chatgpt_account_header`
- `cargo test -p codexmanager-service compact_transport_preserves_service_tier_with_chatgpt_account_header`
- `cargo test -p codexmanager-service gateway::upstream::attempt_flow::transport::tests::`

备注：由于默认 `target/debug/deps/gateway_logs-*.exe` 有旧进程锁，验证命令使用独立 `CARGO_TARGET_DIR=target\codex-test-json-parse` 执行。
