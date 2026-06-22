-- 中文注释：新增日级 rollup 表缓存已结束日期的统计数据。
-- 主键包含 day_start + 多维度（key_id, account_id, source_kind, source_id, user_id, model, status_bucket），
-- 支持按日期、账户、用户、来源、状态聚合，避免反复扫描 request_token_stats 明细。
-- 允许部分维度为 NULL（系统级聚合），主键中 NULL 字段用空字符串归一化。
-- 历史日数据写入后不可变（immutable），当前日数据仍从 live 表查询。
CREATE TABLE IF NOT EXISTS request_token_stat_daily_rollups (
  day_start INTEGER NOT NULL,
  key_id TEXT NOT NULL DEFAULT '',
  account_id TEXT NOT NULL DEFAULT '',
  source_kind TEXT NOT NULL DEFAULT '',
  source_id TEXT NOT NULL DEFAULT '',
  user_id TEXT NOT NULL DEFAULT '',
  model TEXT NOT NULL DEFAULT '',
  status_bucket TEXT NOT NULL DEFAULT '',
  input_tokens INTEGER NOT NULL DEFAULT 0,
  cached_input_tokens INTEGER NOT NULL DEFAULT 0,
  output_tokens INTEGER NOT NULL DEFAULT 0,
  total_tokens INTEGER NOT NULL DEFAULT 0,
  reasoning_output_tokens INTEGER NOT NULL DEFAULT 0,
  estimated_cost REAL NOT NULL DEFAULT 0.0,
  request_count INTEGER NOT NULL DEFAULT 0,
  success_count INTEGER NOT NULL DEFAULT 0,
  error_count INTEGER NOT NULL DEFAULT 0,
  source_rows INTEGER NOT NULL DEFAULT 0,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (day_start, key_id, account_id, source_kind, source_id, user_id, model, status_bucket)
);

-- 索引：按日期 + 账户查询（管理员概览、账户级报表）
CREATE INDEX IF NOT EXISTS idx_request_token_stat_daily_rollups_day_account
  ON request_token_stat_daily_rollups(day_start, account_id);

-- 索引：按日期 + 用户查询（成员仪表盘、用户级报表）
CREATE INDEX IF NOT EXISTS idx_request_token_stat_daily_rollups_day_user
  ON request_token_stat_daily_rollups(day_start, user_id);

-- 索引：按日期 + 来源查询（来源级用量排行）
CREATE INDEX IF NOT EXISTS idx_request_token_stat_daily_rollups_day_source
  ON request_token_stat_daily_rollups(day_start, source_kind, source_id);
