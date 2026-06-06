# 2026-06-06 Token refresh 失败退避与永久无效过滤

## 状态

✅ 已完成

## 背景

运行版只读诊断显示大量账号处于不可用状态，`events` 中 `usage_refresh_failed` 和 refresh token 401 类错误占主要体积。前序提交已降低用量轮询失败事件写入与失败账号重试，但令牌刷新后台任务仍按 60 秒默认周期扫描 due token。

当 refresh token 已复用、吊销、过期或 invalid_grant 时，服务端会把账号标记为 `refresh_token_invalid:*`。旧查询只排除了停用类状态与 `refresh_token_region_blocked`，这类永久无效 token 仍可能继续进入后台令牌刷新候选。

## 修改

- `list_tokens_due_for_refresh()` 排除最新状态为 `refresh_token_invalid:*` 的账号。
- 用量轮询候选状态过滤同步排除 `refresh_token_invalid:*`。
- 令牌刷新失败后只更新 `next_refresh_at` 到失败冷却时间，默认 6 小时。
- 新增 `CODEXMANAGER_TOKEN_REFRESH_FAILURE_COOLDOWN_SECS` 覆盖项。
- 新增 `update_token_next_refresh_at()`，避免失败退避覆盖 `access_token_exp`。
- 更新后台任务跳过说明与环境变量文档。

## 验证

- 新增 storage 测试覆盖 `refresh_token_invalid:*` 过滤。
- 新增 storage 测试确认失败退避只改 `next_refresh_at`。
- 新增 service 测试确认默认失败冷却覆盖普通轮询周期。

## 后续

- 后续可以把账号最新状态从 `events` 流水表迁移到状态表，进一步降低大库下的状态查询成本。
- 运行版需要重编译替换 exe 才能吃到本提交；旧 DB/WAL 收缩仍需单独运维步骤。
