-- 为用户排行 charge 子查询优化添加部分索引
-- 仅索引 entry_kind='request_charge' 的行，减小索引体积
-- 对应查询: summarize_request_token_stats_user_ranking_between
CREATE INDEX IF NOT EXISTS idx_app_wallet_ledger_request_charge
  ON app_wallet_ledger_entries(request_log_id)
  WHERE entry_kind = 'request_charge';
