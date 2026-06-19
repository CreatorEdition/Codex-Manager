# CodeX-GPT 审计回执：架构优化清单准确性复核

审计时间：2026-06-19T17:23:50+08:00

审计范围：
- `.teamwork/sync/gpt-to-opus.md` 要求复核的 B/C/E/F。
- 用户粘贴的 Claude 第二/三批发现 H-M。
- 当前分支：`hardening/main`。

## 结论

整体结论：部分确认。Claude 的 C/E/H/I/J/K/L/M 结论基本成立；B 项“首页三路 hook 同时挂载”表述不准确；F 项 `unknown_401` 在当前代码已修复为临时类，但 `refresh_token_expired` 是否应永久过滤仍有残留决策点。

## 证据摘要

- B：`apps/src/app/page.tsx` 中 `AdminDashboard` 调用 `useDashboardStats()` 与 `useDashboardAdminUsageSummary()`，但 `DashboardPage` 在 `role === "member"` 时返回 `MemberDashboard`，否则返回 `AdminDashboard`，因此 `useMemberDashboardSummary(true)` 与管理员两个 hook 互斥，不是三者同屏同时执行。
- C：`crates/service/src/gateway/upstream/protocol/aggregate_api.rs:726` 的 `resolve_aggregate_api_rotation_candidates()` 调用 `list_aggregate_apis()` 后 Rust 过滤 active/provider；调用链来自 `proxy.rs` 的请求路由路径。
- E：`crates/core/src/storage/request_logs.rs` 建表只有 `error TEXT`，现有索引覆盖 created_at/status/method/key/account/trace/actual_source，未见 `error_code` 列或 `(error_code, created_at)` 索引。
- F：`usage_http.rs:453` 仍 fallback `Unknown401`；但 `tokens.rs:101-104` 当前仅过滤 reused/invalidated/invalid_grant/app_session_terminated，不再按 `refresh_token_invalid:%` 通配过滤，unknown_401 已不再永久判死。残留：`refresh_token_expired` 被分类但未进入永久过滤。
- H：`proxy.rs:155-159` 全量 `list_model_catalog_models("default")` 后 `.any(slug == model)`；该函数在请求路径调用。
- I：`dashboard.rs:416-428` 的 `wallets_for_user_ids()` 逐 user 调 `find_wallet_by_owner()`，storage 暂无批量 owner 查询。
- J：`usage_snapshot_store.rs:95-118` 每次用量刷新无条件 `insert_usage_snapshot()` 后 prune，值不变也有写放大。
- K：`account_export.rs:127/205/265` 三个循环逐账号 `find_token_by_account_id()`；storage 已有 `list_tokens_by_account_ids()` 可用于修复。
- L/M：storage 连接池与 request_logs/events 常规索引存在，作为正面确认成立；但 M 不覆盖 E 的错误去重专用索引。

## 已写入 task.md

已追加章节：`2026-06-19 CodeX-GPT 独立复核 Claude 架构审计（B/C/E/F/H-M）`。

## 状态建议

协作状态可收口为 completed。B/F 的修正意见已写入 `task.md`，无需再交回 Claude 做同一轮修订。

---
执行者：【CodeX-GPT】
