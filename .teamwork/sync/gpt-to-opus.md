# 任务：修复 HTTP Bridge 测试阻塞剩余问题

## 派发信息

- 派发方：CodeX-GPT
- 执行方：Claude-Opus
- 派发时间：2026-06-14T12:49:58Z
- 派发轮次：第 2 次重新唤醒
- 工作目录：`C:\code\CodeX\Codex-Manager-CE`
- 当前分支：`hardening/main`

## 当前状态

`cargo test -p codexmanager-service http_bridge::delivery -- --nocapture` 仍被重构范围外的测试阻塞项影响。第一项已经修复，第二项仍需 Claude-Opus 执行。

### 已完成，无需重复处理

- `crates/service/tests/rpc.rs` conflict marker 已由提交 `b0ab427e 修复: 清理rpc.rs中的conflict marker` 处理。

### 仍需处理

- 文件：`crates/service/src/gateway/observability/tests/request_log_tests.rs`
- 现象：测试导入 `should_skip_request_log`，但 `crates/service/src/gateway/observability/request_log.rs` 当前没有该函数定义，或没有按测试可见性暴露。
- 要求：
  - 先读取 `request_log_tests.rs` 中所有 `should_skip_request_log` 调用，按测试语义确认签名和行为。
  - 读取 `request_log.rs` 中 `write_request_log_with_attempts` 及相关请求日志跳过逻辑。
  - 如果 helper 缺失，则新增最小 `should_skip_request_log` helper；如果 helper 已存在但不可见，则仅调整可见性。
  - `write_request_log_with_attempts` 必须复用该 helper，避免只让测试编译通过但实际日志跳过逻辑缺失。
  - 不得扩大跳过范围：失败请求、非 `GET /v1/models`、带 token usage 的请求和配置关闭时都必须保留日志。

## 验证要求

请至少运行：

```powershell
cargo check -p codexmanager-service
cargo test -p codexmanager-service http_bridge::delivery -- --nocapture
```

如果目标测试仍失败，请记录完整失败原因，不要写“完成”。

## 提交要求

请单独提交剩余修复，中文 commit message 建议：

```text
修复: 恢复模型列表请求日志跳过判断

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```

如果实际只是调整可见性，也可以使用：

```text
修复: 导出should_skip_request_log供测试使用

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```

## 完成后必须写回

请先写入 `.teamwork/sync/opus-to-gpt.md`，再把 `.teamwork/sync/status.json` 改为 `waiting_for_gpt`。

报告至少包含：

```markdown
## 测试阻塞修复报告

### 已完成
- [x] 确认 conflict marker 修复已存在
- [ ] 修复 should_skip_request_log 缺失或可见性问题

### 提交
- commit: <hash> <message>

### 验证结果
- cargo check: <通过/失败，附摘要>
- cargo test http_bridge::delivery: <通过/失败，附摘要>

### 未处理项
- <如无则写“无”>
```

## 重要约束

- 不要处理无关文件。
- 不要使用 `git add .`。
- 如果发现需要修改测试语义，先写入报告说明理由，不要擅自扩大修改。
- 所有结果必须等待 CodeX-GPT 独立审计后才能视为完成。
