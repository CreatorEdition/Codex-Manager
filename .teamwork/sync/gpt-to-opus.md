# 审计报告：日级 rollup 表实现（D 项 - 阶段 1）

**审计方**: CodeX-Opus-4.6  
**审计时间**: 2026-06-22  
**审计对象**: Commit c6ce98d7d5c7de1e6cc5a5db4646945ff54690a7

---

## 审计摘要

✅ **通过审计，阶段 1 实施正确**

D 项阶段 1（建表和迁移）成功完成。表结构设计合理，主键和索引覆盖查询场景，Storage 层方法实现正确，测试覆盖充分。向后兼容性良好，代码质量符合标准。

---

## 详细审计结果

### 1. 表结构设计 ✅

**迁移脚本**: `crates/core/migrations/074_request_token_stat_daily_rollups.sql`

**表定义验证**：
```sql
CREATE TABLE IF NOT EXISTS request_token_stat_daily_rollups (
  day_start INTEGER NOT NULL,
  key_id TEXT NOT NULL DEFAULT '',
  account_id TEXT NOT NULL DEFAULT '',
  source_kind TEXT NOT NULL DEFAULT '',
  source_id TEXT NOT NULL DEFAULT '',
  user_id TEXT NOT NULL DEFAULT '',
  model TEXT NOT NULL DEFAULT '',
  status_bucket TEXT NOT NULL DEFAULT '',
  -- ... 9 个聚合字段 ...
  PRIMARY KEY (day_start, key_id, account_id, source_kind, source_id, user_id, model, status_bucket)
);
```

**验证结论**：
- ✅ **主键设计**：8 维度组合主键正确覆盖所有聚合维度
  - `day_start`: 日期维度（必需）
  - `key_id`, `account_id`: API Key 和账户维度
  - `source_kind`, `source_id`: 来源类型和 ID
  - `user_id`: 用户维度
  - `model`: 模型维度
  - `status_bucket`: 状态桶（成功/失败）
  
- ✅ **NULL 处理**：使用 `NOT NULL DEFAULT ''` 处理空维度
  - 设计合理：避免主键包含 NULL（SQLite 主键不支持 NULL）
  - 空字符串表示"系统级聚合"（所有账户/所有用户）
  
- ✅ **聚合字段完整**：
  - Tokens: `input_tokens`, `cached_input_tokens`, `output_tokens`, `total_tokens`, `reasoning_output_tokens`
  - 成本: `estimated_cost` (REAL 类型)
  - 计数: `request_count`, `success_count`, `error_count`
  - 元数据: `source_rows` (来源行数), `updated_at` (更新时间)

- ✅ **向后兼容**：`CREATE TABLE IF NOT EXISTS` 幂等处理

### 2. 索引设计 ✅

**索引覆盖验证**：

1. **`idx_request_token_stat_daily_rollups_day_account`** - `(day_start, account_id)`
   - ✅ 覆盖场景：管理员概览、账户级报表、账户用量排行
   - ✅ 选择性高：day_start 过滤 + account_id 精确查找

2. **`idx_request_token_stat_daily_rollups_day_user`** - `(day_start, user_id)`
   - ✅ 覆盖场景：成员仪表盘、用户级报表、用户用量排行
   - ✅ 选择性高：day_start 过滤 + user_id 精确查找

3. **`idx_request_token_stat_daily_rollups_day_source`** - `(day_start, source_kind, source_id)`
   - ✅ 覆盖场景：来源级用量排行（聚合 API、账户来源）
   - ✅ 组合索引：支持 source_kind 过滤 + source_id 精确查找

**验证结论**：
- ✅ 索引数量适中（3 个），避免过度索引影响写入性能
- ✅ 索引列顺序正确（day_start 在前，过滤基数大）
- ✅ 覆盖主要查询场景（按账户/用户/来源聚合）
- ✅ 主键本身可作为索引，无需额外索引

**改进建议**（非阻塞）：
- 如果需要按 model 维度查询（模型级报表），可考虑添加 `(day_start, model)` 索引
- 当前索引足够覆盖 task.md 中提到的场景

### 3. Storage 层实现 ✅

**插入方法** - `insert_request_token_stat_daily_rollup()`:

```rust
ON CONFLICT(...) DO UPDATE SET
  input_tokens = request_token_stat_daily_rollups.input_tokens + excluded.input_tokens,
  // ... 累加其他字段 ...
  updated_at = excluded.updated_at
```

**验证结论**：
- ✅ **冲突处理正确**：使用 `ON CONFLICT ... DO UPDATE` 实现 UPSERT
- ✅ **累加逻辑正确**：所有计数和 tokens 字段累加，updated_at 覆盖
- ✅ **参数绑定安全**：使用 `params!` 宏防止 SQL 注入
- ✅ **字段映射完整**：19 个参数与表列一一对应

**查询方法** - `query_request_token_stat_daily_rollups()`:

```rust
WHERE day_start >= ?1 AND day_start <= ?2
```

**验证结论**：
- ✅ 按日期范围查询（闭区间）
- ✅ 返回 `Vec<RequestTokenStatDailyRollup>`
- ✅ 行映射完整（从 rusqlite Row 到结构体）

### 4. 测试覆盖 ✅

**5 个测试用例验证**：

1. **`test_daily_rollup_table_creation`**
   - ✅ 验证表创建成功
   - ✅ 验证 3 个索引存在

2. **`test_insert_and_query_daily_rollup`**
   - ✅ 基础 CRUD 功能
   - ✅ 插入后查询验证数据正确

3. **`test_daily_rollup_upsert_conflict`**
   - ✅ **核心逻辑验证**：主键冲突时累加
   - ✅ 插入两次相同主键，验证 tokens/counts 正确累加

4. **`test_daily_rollup_empty_string_normalization`**
   - ✅ **边界情况**：空字符串维度（系统级聚合）
   - ✅ 验证空 key_id/account_id/user_id 可以存储

5. **`test_daily_rollup_multiple_dimensions`**
   - ✅ **多维度组合**：不同维度组合独立存储
   - ✅ 验证主键正确隔离不同维度的记录

**测试结果**: **5 passed; 0 failed**

**验证结论**：
- ✅ 覆盖核心逻辑（UPSERT 累加）
- ✅ 覆盖边界情况（空维度、多维度）
- ✅ 测试数据合理（模拟真实场景）
- ✅ 断言完整（验证累加结果正确）

### 5. 向后兼容性 ✅

**幂等处理验证**：
- ✅ `CREATE TABLE IF NOT EXISTS` - 表已存在不报错
- ✅ `CREATE INDEX IF NOT EXISTS` - 索引已存在不报错
- ✅ Migration 074 可重复运行
- ✅ 旧库升级到新版本不会失败

**数据迁移验证**：
- ✅ 新表独立，不影响现有 `request_token_stats` 表
- ✅ 阶段 1 只建表，不强制填充数据（阶段 2 实施）
- ✅ 现有查询路径不受影响（阶段 3 才重构）

### 6. 命名规范 ✅

**表名**: `request_token_stat_daily_rollups`
- ✅ 复数形式（rollups）
- ✅ 与现有表 `request_token_stats` 命名一致
- ✅ 语义清晰（daily rollup 表达日级聚合）

**列名**: `day_start`, `key_id`, `input_tokens` 等
- ✅ snake_case 命名
- ✅ 与现有表列名风格一致

**索引名**: `idx_request_token_stat_daily_rollups_day_account`
- ✅ `idx_` 前缀 + 表名 + 维度列
- ✅ 符合项目约定

**方法名**: `insert_request_token_stat_daily_rollup()`, `query_request_token_stat_daily_rollups()`
- ✅ snake_case 命名
- ✅ 语义清晰（insert/query 动词 + 实体名）

---

## 代码质量评估

### 优点
1. ✅ 表结构设计合理，主键和索引覆盖查询场景
2. ✅ NULL 处理巧妙（空字符串归一化）
3. ✅ UPSERT 逻辑正确（冲突时累加）
4. ✅ 测试覆盖充分（核心逻辑+边界情况）
5. ✅ 向后兼容性良好（幂等处理）
6. ✅ 代码注释清晰（中文注释说明设计意图）

### 改进空间（非阻塞）
1. 📝 可为 Storage 方法添加更多文档注释（参数说明、返回值说明）
2. 📊 可补充性能测试（大量数据插入和查询的基准测试）
3. 🔍 可添加数据一致性测试（对比 rollup 聚合 vs live 查询结果）

---

## 性能影响分析

### 写入开销
- 新表写入：每天固化一次（阶段 2 实施）
- 索引维护：3 个索引，写入时自动更新
- **预期影响**：固化任务异步后台执行，不影响实时请求

### 查询收益
- 历史日查询：从扫描 N 天 `request_token_stats`（百万行）→ 读取 N 行 rollup
- **预期加速**：10-100 倍（取决于数据量和查询范围）
- 当前日查询：仍读 live 表（与现状一致）

### 存储开销
- Rollup 表行数：每天 K 维度组合（估计每天 10K-100K 行）
- 对比明细表：`request_token_stats` 每天可能数百万行
- **预期节省**：明细表可定期清理（保留 7-30 天），rollup 长期保留

---

## 审计决策

### ✅ **通过，允许进入阶段 2**

**理由**：
1. 表结构设计合理，主键和索引正确
2. Storage 层实现正确，UPSERT 逻辑验证通过
3. 测试覆盖充分，核心逻辑和边界情况全部验证
4. 向后兼容性良好，幂等处理到位
5. 代码质量高，命名规范符合项目约定
6. 阶段 1 目标明确且独立可验证

### 📋 **后续建议**

**阶段 2 实施要点**：
- 固化任务错误处理：失败不能阻塞业务查询
- 回填逻辑需要事务保护（避免部分成功）
- 清理策略需要可配置（避免误删明细）
- 时区处理明确（day_start 定义为 UTC 或本地时区）

**阶段 3 实施要点**：
- 查询函数需要数据一致性测试（rollup vs live 对比）
- 端点重构需要 A/B 测试（验证性能改善）
- 缓存策略需要监控（确认命中率）

---

## 验收清单

- ✅ Migration 074 正确创建表和索引
- ✅ 主键设计覆盖所有聚合维度
- ✅ 索引设计满足查询场景需求
- ✅ Storage 层 UPSERT 逻辑正确
- ✅ 测试覆盖核心逻辑和边界情况
- ✅ 向后兼容性验证通过
- ✅ 命名规范符合项目约定
- ✅ 代码质量良好，注释清晰

---

## 下一步行动

1. ✅ **允许合并阶段 1 commit**: c6ce98d7
2. 📝 **更新 task.md**: 标记 D 项阶段 1 已完成
3. 🔄 **启动阶段 2**: 实施维护任务和回填逻辑
4. 📊 **性能监控**: 观察固化任务执行时间和查询改善

---

**审计签名**: CodeX-Opus-4.6  
**状态**: ✅ 通过  
**时间**: 2026-06-22  
**下一阶段**: 可以启动阶段 2（维护任务）
