# 任务：审计架构优化清单的准确性

## 📌 背景
Claude 已对仓库做持续架构审计，将 7 类优化点（A-G）写入 task.md（commit 2f54f41a）。其中 B（首页 hook 叠加）和 C（网关聚合 API 热路径全表扫描）是本轮新发现。

## 🎯 GPT 审计职责
独立用 git/grep 复核以下新发现的证据点是否准确，不盲从 Claude 结论：

### 必须验证的证据
1. **C 项热路径全表扫描**：
   - `crates/service/src/gateway/upstream/protocol/aggregate_api.rs:726` 的 `resolve_aggregate_api_rotation_candidates()` 是否确实 `list_aggregate_apis()` 全量读后 Rust 过滤
   - 该函数是否在请求转发热路径被调用（grep 调用方）

2. **B 项首页 hook 叠加**：
   - `apps/src/app/page.tsx` 是否同时挂载 useDashboardStats / useDashboardAdminUsageSummary / useMemberDashboardSummary
   - 三者 staleTime 是否分别为 15s/30s/30s

3. **E 项索引缺口**：
   - `request_logs` 是否确实无 error/success 字段索引

4. **F 项 Unknown401 边界**：
   - `usage_http.rs:453` 是否把未匹配 401 fallback 成 Unknown401
   - `tokens.rs:101` 是否用 `refresh_token_invalid:` 前缀排除轮询

## 📤 交付
在 opus-to-gpt.md 写审计结论：每个证据点标注 ✅确认 / ❌不符 / ⚠️部分。
- 全部确认 → status 改 completed
- 有不符 → status 改 needs_revision，列出需修正的清单项

## ⚠️ 约束
- 只审计 task.md 的准确性，不修改代码
- 用证据（行号/grep 输出）支撑结论
