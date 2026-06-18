# 任务：合并 hardening/main 到 main 并推送

## 📌 任务目标
将 hardening/main 的全部改动合并到 main，使 main 成为可发布分支，然后推送到远程。

## 🎯 背景
- 用户触发 GitHub Release workflow 时使用 `main` 作为 ref
- 但所有改动（117个commit）都在 hardening/main
- main 当前落后 117 提交，是空的上游镜像
- 已验证：main 是 hardening/main 的祖先，可 fast-forward 合并（无冲突）

## 📋 执行步骤

### 步骤1：切换到 main
```bash
git checkout main
```

### 步骤2：Fast-forward 合并
```bash
git merge --ff-only hardening/main
```
使用 --ff-only 确保是纯快进合并，不产生 merge commit，保持历史线性。

### 步骤3：验证合并结果
```bash
git log --oneline -3
git rev-list --left-right --count main...hardening/main
# 期望：0  0（两分支完全一致）
```

### 步骤4：推送 main
```bash
git push origin main
```

### 步骤5：切回 hardening/main（保持工作分支）
```bash
git checkout hardening/main
```

## ⚠️ 约束条件
1. 必须使用 --ff-only，禁止产生 merge commit
2. 不修改任何代码文件
3. 合并后切回 hardening/main 作为工作分支
4. 不使用 git add .

## 📤 交付要求
在 opus-to-gpt.md 中提供：
- 合并前后的 commit 对比
- main 与 hardening/main 的差异验证（应为 0 0）
- 推送结果
- 当前所在分支确认

---
**工作目录**: /c/code/CodeX/Codex-Manager-CE
**合并方式**: fast-forward（已验证无冲突）
