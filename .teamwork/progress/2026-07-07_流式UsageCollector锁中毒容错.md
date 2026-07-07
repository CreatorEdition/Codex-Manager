# 流式 Usage Collector 锁中毒容错

## 状态

✅ 已完成

## 背景

子代理只读审计确认生产路径没有 `Mutex::lock().unwrap/expect` 直接 panic 点，但存在锁中毒后通过 `if let Ok(...)` 或 `unwrap_or_default()` 静默丢失 usage、终止事件、terminal error 的问题。

## 处理内容

- 复用仓库已有 `crate::lock_utils::lock_recover`，为 `PassthroughSseCollector` 与 `UpstreamResponseUsage` 增加统一访问 helper。
- 将流式 reader 的 collector 写入路径改为锁中毒后记录告警并继续恢复写入。
- 将 `delivery.rs` 收尾读取路径改为恢复后 snapshot，避免 poisoned 时把真实 collector 状态替换成默认值。
- 补充 poisoned collector 单测，覆盖 passthrough collector 和 usage collector 恢复路径。

## 审计结论

子代理报告仅作为输入使用；主代理已重新扫描生产路径，剩余直接 `usage_collector.lock().expect(...)` 命中均位于测试模块内。

## 验证

已通过：
- `cargo test -p codexmanager-service usage_collector`
- `cargo test -p codexmanager-service http_bridge`
- `cargo fmt --all --check`
- `git diff --check`
- `rg -n "Linux\\.do|linux\\.do" README.md task.md docs/zh-CN/CHANGELOG.md`
