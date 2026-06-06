# 2026-06-07 用量窗口数据驱动展示

## 背景

用户反馈 Free 账号疑似改为按月刷新额度，且后续其他账号类型也可能出现不同刷新周期。当前前端和文档多处把单长周期窗口写死成 7 天，会导致 30 天或其他服务端窗口被错误展示。

## 处理

- 账号页额度条改用服务端 `windowMinutes` 动态生成窗口标签，30 天会显示为 `30天窗口`，不再固定显示 `7天`。
- 可用性文案从 `仅7天额度` / `7天窗口未提供` 改为 `仅长周期额度` / `长周期窗口未提供`。
- 设置页、README、额度管理中心设计文档同步从 `7天单窗口` 收口为 `单长周期窗口` 或 `短周期 + 长周期`。
- Rust 聚合测试把单长周期 free 示例改为 43200 分钟，确认月度窗口仍进入长周期桶。

## 验证

- `cargo fmt --all --check`
- `cargo test -p codexmanager-core usage_aggregate_summary_matches_bucket_semantics`
- `cargo test -p codexmanager-service aggregate_summary_routes_long_single_window_account_to_secondary_bucket`
- `git diff --check`

## 限制

- `pnpm` 不在 PATH，前端 lint/build 未能执行；本次仅做 TypeScript diff 静态审查。
