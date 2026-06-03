CREATE INDEX IF NOT EXISTS idx_api_keys_status_created_id
  ON api_keys(status, created_at DESC, id ASC);

CREATE INDEX IF NOT EXISTS idx_api_keys_created_id
  ON api_keys(created_at DESC, id ASC);

CREATE INDEX IF NOT EXISTS idx_api_key_owners_kind_user_key
  ON api_key_owners(owner_kind, owner_user_id, key_id);
