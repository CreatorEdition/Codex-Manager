# 2026-06-06 日志页轮询降载

## 背景

- 运行版 request logs 约 1.3 万行，`/v1/models?...` 成功探测占比较高；成功模型列表日志已默认跳过。
- 日志页仍每 5 秒同时触发 `requestlog/list` 和 `requestlog/summary`，大库下会重复执行分页 COUNT、过滤摘要和 token 聚合。

## 变更

- 请求日志列表自动刷新间隔从 5 秒调整为 30 秒。
- 日志列表查询禁用后台刷新，页面不可见时不继续轮询。
- 日志摘要查询不再定时轮询，只随过滤条件变化、清空日志或手动 invalidation 重新请求。
- Web RPC `requestlog/list` 与 `requestlog/summary` 使用 30 秒超时、0 次重试，避免 10 秒默认超时后重复打慢查询。

## 验证

- 更新 `transport-web-commands` runtime 测试，固定 requestlog/list 与 requestlog/summary 的 Web 请求超时和重试策略。

## 后续

- 后续仍应把日志摘要做缓存或拆成手动刷新，日志列表可进一步改成增量读取最新 ID。
