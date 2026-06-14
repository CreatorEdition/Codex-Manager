# 任务：代码质量检查与优化建议

## 📌 任务目标
审查当前codebase的代码质量问题，识别改进机会，并提供优化建议（不直接修改代码）。

## 🎯 审查范围

### 1. Rust代码质量
- [ ] 检查是否有unreachable pattern警告
- [ ] 检查是否有unused代码（dead_code警告）
- [ ] 审查unsafe代码使用
- [ ] 审查unwrap/expect使用

### 2. 测试覆盖
- [ ] 识别缺少测试的关键模块
- [ ] 检查测试命名和组织
- [ ] 评估测试质量

### 3. 文档质量
- [ ] 检查公开API文档完整性
- [ ] 审查README准确性
- [ ] 识别过时文档

## 📋 执行步骤

### 步骤1：编译警告分析
```bash
cargo check --workspace 2>&1 | grep "warning:" | head -20
```

### 步骤2：查找unsafe代码
```bash
grep -r "unsafe " crates/ --include="*.rs" | wc -l
```

### 步骤3：查找unwrap使用
```bash
grep -r "\.unwrap()" crates/ --include="*.rs" | wc -l
```

### 步骤4：测试覆盖评估
```bash
find crates/ -name "*.rs" -path "*/tests/*" | wc -l
```

## 📤 交付格式

在 `.teamwork/sync/opus-to-gpt.md` 中提供：

```markdown
# 代码质量审查报告

## 📊 概览
- 审查日期：YYYY-MM-DD
- 审查范围：全工作区
- 严重问题数：X
- 建议改进数：X

## 🔴 需要关注的问题
[列出需要修复的问题，按优先级排序]

## 🟡 改进建议
[列出可选的改进建议]

## ✅ 良好实践
[列出代码中的良好实践]

## 📝 优先级建议
[建议下一步应该处理什么]
```

## ⚠️ 约束条件

1. **只审查，不修改**：本次任务是质量审查，不直接修改代码
2. **提供证据**：每个问题都要有具体位置和代码片段
3. **优先级排序**：按影响程度和修复难度排序

---

**工作目录**: `/c/code/CodeX/Codex-Manager-CE`
**当前分支**: `hardening/main`
**预计时间**: 15-20分钟
