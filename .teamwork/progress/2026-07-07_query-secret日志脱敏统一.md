# 2026-07-07 query-secret 日志脱敏统一

## 状态

- ✅ 已确认聚合 API `query-secret` / `username/password query pair` 会把密钥拼进真实上游 URL，真实转发必须保留。
- ✅ 已新增共享 `log_redaction` URL 脱敏工具，日志侧统一移除 query 与 fragment。
- ✅ DB 请求日志写入前会脱敏 `upstream_url` 与 `aggregate_api_url`。
- ✅ 请求日志列表展示继续复用同一脱敏规则，历史异常数据即使带 query 也不会在 UI 继续暴露。
- ✅ 失败 trace 的 `ATTEMPT_RESULT` 与 `FAILED_REQUEST` 上游 URL 也复用同一脱敏规则。

## 变更范围

- `crates/service/src/log_redaction.rs`
- `crates/service/src/lib.rs`
- `crates/service/src/requestlog/requestlog_list.rs`
- `crates/service/src/gateway/observability/request_log.rs`
- `crates/service/src/gateway/observability/trace_log.rs`
- `crates/service/src/gateway/observability/tests/request_log_tests.rs`
- `docs/zh-CN/CHANGELOG.md`
- `task.md`

## 验证

- ✅ `cargo test -p codexmanager-service query_secret`
- ✅ `cargo test -p codexmanager-service redacts_query`
- ✅ `cargo fmt --all --check`
