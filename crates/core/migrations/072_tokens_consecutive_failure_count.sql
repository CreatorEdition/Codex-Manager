-- 中文注释：为 tokens 表新增连续刷新失败计数列，支撑临时故障的 per-account 指数退避。
-- 永久无效仍走长冷却并退出服务池，临时故障（网络/5xx/超时/Unknown401）按该计数指数退避，
-- 任意一次刷新成功即清零。SQLite 的 ADD COLUMN 在 NOT NULL 时必须带 DEFAULT。
ALTER TABLE tokens ADD COLUMN consecutive_failure_count INTEGER NOT NULL DEFAULT 0;
