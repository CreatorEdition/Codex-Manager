#!/bin/bash
# 这是给ChatGPT的审计指令

echo "=== ChatGPT Git审计任务 ==="
echo ""
echo "请执行以下审计步骤："
echo ""
echo "1. 验证Commit历史"
git log --oneline -7
echo ""
echo "2. 查看每个Commit详情"
for commit in 0e222950 3400949f f87458b7 731cf5fe bc9896bc; do
    echo "--- Commit: $commit ---"
    git show $commit --stat
    echo ""
done
echo ""
echo "3. 验证编译状态"
cargo check 2>&1 | head -20
echo ""
echo "4. 查看整体变更"
git diff 6c7a6481..bc9896bc --stat
