# 任务：HTTP Bridge 重复逻辑重构

## 任务目标

对 `crates/service/src/gateway/observability/http_bridge/delivery.rs` 进行等价重构，消除 `respond_with_upstream` 与 `respond_with_stream_upstream` 中重复的元数据提取、响应头准备和 Content-Type 判断逻辑。

## 执行范围

- 目标文件：`crates/service/src/gateway/observability/http_bridge/delivery.rs`
- 允许修改：仅限重复逻辑的等价重构
- 禁止修改：协议行为、响应体转换逻辑、错误语义、header 过滤策略、权限边界和无关文件

## 预期提交

1. 添加 `UpstreamMetadata` 与 `extract_upstream_metadata()`。
2. 添加 `should_skip_response_header()` 与 `prepare_response_headers()`。
3. 添加 `ContentTypeInfo` 与 `analyze_content_type()`。
4. 在 `respond_with_upstream()` 中使用上述 helper。
5. 在 `respond_with_stream_upstream()` 中使用上述 helper。

## 验收要求

- 每个逻辑步骤单独中文 commit。
- 完成后写入 `opus-to-gpt.md`，列出 commit、变更摘要和自检结果。
- CodeX-GPT 必须独立使用 git diff / cargo / rustfmt 复核，不得直接采纳 Claude/Opus 结论。

## 当前完成状态

Claude Opus 已完成重构并生成 5 个 commit；CodeX-GPT 已完成独立审计。最终结论见：

- `.teamwork/sync/opus-to-gpt.md`
- `.teamwork/sync/gpt-audit-result.md`
- `.teamwork/sync/status.json`
