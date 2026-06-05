# 2026-06-06 用量刷新失败事件默认降噪

## 负责人

- 【CodeX-GPT】

## 背景

- 运行版只读诊断显示 `events` 是数据库体积主因之一，最大事件类型包含 `usage_refresh_failed`。
- 现有代码已经按账号和错误类做内存节流，但默认窗口只有 60 秒。
- 默认用量轮询间隔是 600 秒，意味着持续上游异常时，每轮都会为每个失败账号重新写一条失败事件；几千账号会让 `events` 和 WAL 继续线性膨胀。

## 变更

- `DEFAULT_USAGE_REFRESH_FAILURE_EVENT_WINDOW_SECS` 从 60 秒提高到 21,600 秒（6 小时）。
- 保留 `CODEXMANAGER_USAGE_REFRESH_FAILURE_EVENT_WINDOW_SECS` 环境变量覆盖能力。
- 新增测试固定默认失败事件窗口至少覆盖默认用量轮询周期。
- 不改变失败分类、账号状态标记、用量刷新重试或轮询调度语义。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service default_failure_event_window_covers_usage_poll_interval`
- ✅ `cargo test -p codexmanager-service usage_refresh_failure_events_are_throttled_by_error_class`
- ✅ `cargo test -p codexmanager-service usage_refresh_failure_throttle_splits_401_reason_classes`
- ✅ `git diff --check`
