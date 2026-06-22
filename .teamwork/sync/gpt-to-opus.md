# 任务：首页统计 hook 叠加优化（B 项）

**发起方**: CodeX-Opus 4.6  
**时间**: 2026-06-22  
**目标**: 解决首页三个独立统计 hook 叠加导致的 CPU 峰值问题

## 问题描述

**位置**: `apps/src/app/page.tsx`

**现状**：首页同时挂载三个独立统计 hook，各自触发后端重聚合：
1. `useDashboardStats()` (line 1023, STALE 15s)
2. `useDashboardAdminUsageSummary()` (line 1070, STALE 30s)
3. `useMemberDashboardSummary(true)` (line 1476, STALE 30s)

**问题**：
- 三个 hook 的统计窗口高度重叠（今日/区间/账号）
- 分别打不同 RPC，导致首页打开瞬间触发 3 路独立聚合扫描
- 重复扫描同一时间窗口，浪费 CPU 和数据库查询

## 实施方案

### 方案 A（推荐）：合并为单一 adminOverview 端点

1. **后端**：新增 `dashboard/adminOverview` 轻量端点
   - 合并三个 hook 需要的所有数据
   - 单次查询返回完整首页数据
   - 使用统一的缓存策略（如 30s stale）

2. **前端**：重构首页数据获取
   - 创建统一的 `useDashboardOverview()` hook
   - 移除三个独立 hook
   - 从单一数据源派生各组件需要的数据

### 方案 B（备选）：共享缓存层

1. 在前端添加跨 hook 的共享缓存
2. 第一个 hook 触发查询，其他 hook 复用结果
3. 统一缓存失效时间

## 验收标准

1. ✅ 首页打开时只触发一次后端聚合查询
2. ✅ 所有组件数据正确显示
3. ✅ 缓存策略统一且合理
4. ✅ `cd apps && npx tsc --noEmit` 通过
5. ✅ 首页加载性能明显改善
6. ✅ Git commit 提交，消息格式：`性能: 首页统计 hook 合并避免重复聚合 (B)`

## 注意事项

- 不要破坏现有组件的数据展示
- 确保管理员和成员两种角色的数据分离
- 保持向后兼容，旧端点可标记 deprecated 但暂不删除
- 完成后将结果写入 `.teamwork\sync\opus-to-gpt.md`

## 实施优先级

- 建议先实施方案 A（更彻底的优化）
- 如果方案 A 改动过大，可先实施方案 B 作为过渡
