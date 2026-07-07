# Official Responses 标准化 Value 复用

## 状态

✅ 已完成

## 背景

`task.md` 剩余 P2 性能观察项包含请求体 JSON parse 深水区。`normalize_official_responses_http_body` 会把 Responses body 解析成强类型结构再序列化；本地校验随后还会重新解析标准化 body 做 metadata 提取和文本长度校验。

## 本轮处理内容

- 保留既有 `normalize_official_responses_http_body` API。
- 新增 `normalize_official_responses_http_body_with_value`，在标准化 body 的同时返回标准化后的 JSON `Value`。
- `apply_passthrough_request_overrides` 使用该 `Value` 提取 request metadata，避免标准化后立即重 parse。
- 主本地校验路径复用该 `Value` 执行文本长度校验与 metadata 提取，失败或非 Responses 路径时保留原解析兜底。

## 验证

已通过：

- `CARGO_TARGET_DIR=target\codex-test-stream-default cargo test -p codexmanager-service responses_http_normalizer`
- `CARGO_TARGET_DIR=target\codex-test-stream-default cargo test -p codexmanager-service aggregate_passthrough_openai_responses_defaults_omitted_stream_to_sse`
- `CARGO_TARGET_DIR=target\codex-test-stream-default cargo test -p codexmanager-service hybrid_passthrough_fallback_body_uses_aggregate_override_shape`
- `CARGO_TARGET_DIR=target\codex-test-stream-default cargo test -p codexmanager-service openai_responses_api_body`
- `cargo fmt --all --check`
- `git diff --check`

备注：本轮不改 README，Linux.do 认可社区入口必须继续保留。
