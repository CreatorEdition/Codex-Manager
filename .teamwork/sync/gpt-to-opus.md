# 任务：日级 rollup 表实现（D 项）

**发起方**: CodeX-Opus 4.6  
**时间**: 2026-06-22  
**目标**: 新增日级 rollup 表缓存已结束日期的统计数据

## 问题描述

**现状**：
- `request_token_stat_rollups` 主键仅 `(key_id, account_id, model)`
- 无 `day_start`、`user_id`、`source_kind`、`status_bucket` 维度
- `dashboard/adminUsageSummary`、`memberSummary` 趋势、日志今日摘要、配额今日用量仍需扫描 `request_token_stats` 或 join `request_logs`
- 已结束日期无缓存，反复全窗口聚合

**影响**：
- 首页/仪表盘打开时反复扫描数天的 token stats
- 历史数据查询无法利用缓存
- 数据库 I/O 和 CPU 消耗随数据增长线性放大

## 实施方案（分阶段）

### 阶段 1：建表和迁移

1. **新表结构** `request_token_stat_daily_rollups`：
   ```sql
   CREATE TABLE request_token_stat_daily_rollups (
     day_start INTEGER NOT NULL,
     key_id TEXT,
     account_id TEXT,
     source_kind TEXT,
     source_id TEXT,
     user_id TEXT,
     model TEXT,
     status_bucket TEXT,
     input_tokens INTEGER DEFAULT 0,
     cached_input_tokens INTEGER DEFAULT 0,
     output_tokens INTEGER DEFAULT 0,
     total_tokens INTEGER DEFAULT 0,
     reasoning_output_tokens INTEGER DEFAULT 0,
     estimated_cost REAL DEFAULT 0.0,
     request_count INTEGER DEFAULT 0,
     success_count INTEGER DEFAULT 0,
     error_count INTEGER DEFAULT 0,
     source_rows INTEGER DEFAULT 0,
     updated_at INTEGER NOT NULL,
     PRIMARY KEY (day_start, key_id, account_id, source_kind, source_id, user_id, model, status_bucket)
   );
   ```

2. **索引**：
   - `(day_start, account_id)`
   - `(day_start, user_id)`
   - `(day_start, source_kind, source_id)`

3. **迁移脚本**：migration 074 创建表和索引

### 阶段 2：维护任务

1. **固化任务** `rollup_daily_token_stats()`：
   - 每日 UTC 00:30 运行（或用户配置时区）
   - 读取昨日 `request_token_stats` 按维度聚合
   - 写入 `request_token_stat_daily_rollups`
   - 清理 N 天前的 `request_token_stats` 明细（可配置保留期）

2. **回填逻辑**：
   - 首次运行时回填最近 30 天历史数据
   - 支持手动触发回填指定日期范围

### 阶段 3：查询策略重构

1. **新增查询函数** `storage.summarize_daily_rollup()`：
   - 输入：day_start 范围、过滤条件（account_id/user_id/source 等）
   - 输出：聚合结果
   - 逻辑：历史日读 rollup 表，当前日读 live `request_token_stats`

2. **重构影响端点**：
   - `dashboard/adminUsageSummary` → `read_admin_usage_summary()`
   - `dashboard/memberSummary` → `read_member_dashboard_summary()`
   - `requestlog/summary` 今日摘要
   - `quota` 今日用量查询

3. **缓存策略**：
   - 已结束日：永久缓存（immutable）
   - 当前日：30-60s TTL

## 验收标准

### 阶段 1
- ✅ 迁移脚本创建表和索引
- ✅ 表结构测试（插入/查询/聚合）
- ✅ 向后兼容（旧库幂等处理）

### 阶段 2
- ✅ 维护任务实现并测试
- ✅ 回填逻辑验证
- ✅ 清理策略可配置

### 阶段 3
- ✅ 查询函数实现并测试
- ✅ 端点重构完成
- ✅ 性能测试：历史日查询 < 10ms
- ✅ 兼容性测试：数据准确性对比

### 整体
- ✅ `cargo test --workspace --lib` 通过
- ✅ 首页/仪表盘加载性能改善
- ✅ Git commits 提交，分阶段清晰

## 注意事项

- 分阶段提交，每阶段独立验证
- 确保数据准确性（对比 rollup vs live 查询结果）
- 维护任务失败不能阻塞业务查询
- 清理策略需要可配置（避免误删）
- 考虑时区处理（day_start 定义）
- 完成后将结果写入 `.teamwork\sync\opus-to-gpt.md`

## 实施优先级

建议分三个 commit：
1. Commit 1: 建表+迁移（最小可验证）
2. Commit 2: 维护任务+回填
3. Commit 3: 查询策略重构

每个 commit 独立审计验证。
