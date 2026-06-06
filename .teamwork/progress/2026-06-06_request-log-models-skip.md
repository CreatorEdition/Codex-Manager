# 2026-06-06 请求日志模型列表降载

## 背景

- 运行版只读诊断显示 `request_logs` 约 13,589 行，其中 `/v1/models?...0.136.0` 约 5,261 行。
- `GET /v1/models` 是 Codex CLI / 客户端高频模型探测路径，成功响应通常没有 token、费用、钱包扣费和推理排障价值。
- 继续把这些成功请求写入 SQLite 会增加 `request_logs` 行数、索引写入、WAL 写入以及维护任务负担。

## 变更

- 默认跳过成功且无用量的 `GET /v1/models` 请求日志写入。
- 保留失败模型列表请求、非 GET 请求、非模型列表请求和所有推理请求日志。
- 跳过发生在 trace final 之后，Prometheus 路由指标和失败 trace 行为不受影响。
- 新增 `CODEXMANAGER_SKIP_SUCCESS_MODEL_LIST_REQUEST_LOGS`，设为 `0` / `false` / `off` / `no` 可关闭跳过。

## 验证

- 新增 request_log 判定测试：
  - 默认跳过成功模型列表日志；
  - 失败、非模型列表、非 GET 不跳过；
  - 环境变量可关闭跳过。

## 后续

- 继续审计 request_logs 的其他高频低价值路径，例如健康检查或客户端探测类请求。
- 继续观察运行版 WAL 收缩效果，避免通过单纯延长保留期掩盖写放大。
