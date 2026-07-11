# Request Rewrite Value 旁路

## 状态

✅ 已完成

## 背景

`task.md` 剩余 P2 性能观察项包含请求体 JSON parse 深水区。`request_rewrite` 会解析请求体并执行模型、reasoning、service tier、prompt cache key 和 allowlist 改写；本地校验随后还会把改写后的 body 交给 Official Responses 标准化与 metadata / 文本长度校验再次解析。

## 本轮处理内容

- 保留既有 `request_rewrite` 旧 API，外部调用仍可只拿 `Vec<u8>`。
- 新增 `RequestRewriteOutput` 与带 `_with_value` 的 scope 入口，JSON 路径返回与 body 对应的重写后 `Value`。
- Official Responses 标准化新增可接收已解析 `Value` 的入口，非 Responses 路径会原样透传该 `Value`。
- 本地校验在 passthrough 和最终请求体路径中复用 rewrite 输出 Value；遇到 compat service tier 二次改写时丢弃旧 Value 并回退解析，避免复用过期结构。

## 验证

已通过：

- `CARGO_TARGET_DIR=target\codex-test-stream-default cargo test -p codexmanager-service request_rewrite_output_value_matches_rewritten_body`
- `CARGO_TARGET_DIR=target\codex-test-stream-default cargo test -p codexmanager-service responses_http_normalizer`
- `CARGO_TARGET_DIR=target\codex-test-stream-default cargo test -p codexmanager-service aggregate_passthrough_openai_responses_defaults_omitted_stream_to_sse`
- `CARGO_TARGET_DIR=target\codex-test-stream-default cargo test -p codexmanager-service hybrid_passthrough_fallback_body_uses_aggregate_override_shape`
- `CARGO_TARGET_DIR=target\codex-test-stream-default cargo test -p codexmanager-service openai_responses_api_body`
- `cargo fmt --all --check`
- `git diff --check`

备注：本轮不改 README，Linux.do 认可社区入口必须继续保留。
