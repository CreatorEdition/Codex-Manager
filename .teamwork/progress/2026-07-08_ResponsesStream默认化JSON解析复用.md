# Responses stream 默认化 JSON 解析复用

## 状态

✅ 已完成

## 背景

`task.md` 剩余 P2 性能观察项包含请求体 JSON parse 深水区。非原生 OpenAI `/v1/responses` 请求在本地校验阶段会默认补齐 `stream=true`，随后再执行文本长度校验；此前这两个步骤会分别解析同一份 JSON body。

## 本轮处理内容

- 新增 `default_omitted_responses_stream_to_true_with_value`，在补齐 `stream=true` 时同步返回解析后的 JSON 值。
- aggregate 直连与 hybrid fallback 的 passthrough body 在默认化后复用该解析结果执行文本长度校验。
- 保留原 `default_omitted_responses_stream_to_true` 入口，避免影响既有调用与测试语义。

## 验证

已通过：

- `CARGO_TARGET_DIR=target\codex-test-stream-default cargo test -p codexmanager-service openai_responses_default_stream_helper_can_reuse_parsed_payload`
- `CARGO_TARGET_DIR=target\codex-test-stream-default cargo test -p codexmanager-service aggregate_passthrough_openai_responses_defaults_omitted_stream_to_sse`
- `CARGO_TARGET_DIR=target\codex-test-stream-default cargo test -p codexmanager-service hybrid_passthrough_fallback_body_uses_aggregate_override_shape`
- `CARGO_TARGET_DIR=target\codex-test-stream-default cargo test -p codexmanager-service openai_responses_api_body`
- `cargo fmt --all --check`
- `git diff --check`

备注：本轮继续保留 README Linux.do 认可社区入口，未改动 README。
