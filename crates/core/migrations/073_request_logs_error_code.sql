-- 中文注释：为 request_logs 表新增规范化错误码列，支撑错误去重聚合（requestlog/errorSummary）。
-- 写入请求日志时由服务层 code_for_message 把 error 原文归类为稳定错误码落库，
-- 配合 (error_code, created_at DESC) 索引按类别聚合，避免日志页被重复错误原文淹没。
-- 旧数据该列为 NULL，随 retention 老化；新错误持续落码。
ALTER TABLE request_logs ADD COLUMN error_code TEXT;
CREATE INDEX IF NOT EXISTS idx_request_logs_error_code ON request_logs(error_code, created_at DESC);
