CREATE INDEX IF NOT EXISTS idx_events_created_at
  ON events(created_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_events_type_account_created_at
  ON events(type, account_id, created_at DESC, id DESC);
