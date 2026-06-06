# 2026-06-06 账号状态事件历史剪枝

## 状态

✅ 已完成

## 背景

运行版只读诊断显示 `events` 是数据库体积主因，且 `account_status_update` 与 `usage_refresh_failed` 是高频事件。此前观测维护为了不破坏状态判断，永久保留所有 `account_status_update`，导致长期运行后状态流水无法随保留周期收缩。

当前状态查询只需要每个账号最新一条状态事件；历史状态流水可作为审计信息按保留周期清理。

## 修改

- `prune_events_by_retention_limited()` 不再无条件排除全部 `account_status_update`。
- 过期非状态事件继续按批清理。
- 过期状态事件只有在同账号存在更新的状态事件时才清理。
- 每个账号至少保留最新一条 `account_status_update`，保障 `latest_account_status_reasons()` 语义。
- 环境变量文档同步说明账号状态事件的保留策略。

## 验证

- 新增测试覆盖过期历史状态事件会被清理。
- 新增测试覆盖仅有一条状态事件的账号仍保留最新状态。
- 新增测试覆盖 `limit` 对历史状态事件清理生效，避免旧库首次维护一次性删除过多。
