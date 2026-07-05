-- 中文注释：记录 token 明细是否已经写入日级 rollup，避免维护任务重启后重复固化。
ALTER TABLE request_token_stats ADD COLUMN daily_rolled_at INTEGER;

CREATE INDEX IF NOT EXISTS idx_request_token_stats_daily_rollup_pending
  ON request_token_stats(daily_rolled_at, created_at, id);
