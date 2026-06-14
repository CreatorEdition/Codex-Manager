# 任务：修复测试阻塞问题

## 📌 任务目标
修复阻塞 `cargo test -p codexmanager-service http_bridge::delivery` 的既有问题，使HTTP Bridge重构的目标测试能够运行。

## 🎯 问题清单

### 问题1：Conflict Marker残留
**文件**: `crates/service/tests/rpc.rs`
**位置**: 行1086附近
**现象**: 存在 `<<<<<<< HEAD` 等git conflict标记
**要求**: 
- 读取该区域完整内容
- 判断正确的合并方式
- 清理conflict marker
- 单独commit："修复: 清理rpc.rs中的conflict marker"

### 问题2：测试模块导出缺失
**文件**: `crates/service/src/gateway/observability/tests/request_log_tests.rs`
**现象**: `should_skip_request_log` 函数未导出，导致其他测试无法引用
**要求**:
- 检查该函数的可见性
- 如需导出，添加 `pub` 修饰符
- 验证不会破坏现有测试
- 单独commit："修复: 导出should_skip_request_log供测试使用"

## ⚠️ 约束条件

1. **分步提交**：每个问题独立commit
2. **验证要求**：
   - 每次commit后运行 `cargo check -p codexmanager-service`
   - 最终运行 `cargo test -p codexmanager-service http_bridge::delivery`
3. **Commit格式**：
```
修复: [具体问题描述]

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```

## 📤 完成后报告

在 `.teamwork/sync/opus-to-gpt.md` 中写入：

```markdown
## 🔧 测试阻塞修复报告

### ✅ 已完成
- [ ] Commit 1: 清理conflict marker
- [ ] Commit 2: 导出should_skip_request_log

### 📊 验证结果
- cargo check: [通过/失败]
- cargo test http_bridge::delivery: [通过/失败]

### 📝 遇到的问题
[如果有]
```

---

**工作目录**: `/c/code/CodeX/Codex-Manager-CE`  
**当前分支**: `hardening/main`  
**预计时间**: 10-15分钟
