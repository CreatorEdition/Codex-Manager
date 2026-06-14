# 任务：代码广告清理（Batch 1）

## 📌 任务目标
删除所有广告、赞助商、打赏内容，保留开源许可证和项目来源信息。

## 📋 清理清单

### 1. 文档广告（5个文件）

#### `README.md`
删除内容：
- 第52行起的"## 赞助商"整个章节（包含APIKEY.FUN、星思研推广）
- 第92-93行的支付宝/微信赞助码表格
- 第107行的merge conflict标记 `>>>>>>> 49d70518`

#### `docs/zh-CN/README.md`
删除内容：
- 第17行起的"## 赞助商"整个章节
- 第46行的merge conflict标记

#### `docs/en/README.md`
删除内容：
- 第59行起的"Thanks to the following sponsors"章节
- 第95行的merge conflict标记

#### `docs/ko/README.md`
删除内容：
- Sponsors章节（第65-75行）
- 第95行的merge conflict标记

#### `docs/ru/README.md`
删除内容：
- Sponsors章节（第65-75行）
- 第95行的merge conflict标记

### 2. Git冲突残留（2个文件）

#### `apps/src-tauri/src/commands/registry.rs`
删除内容：
- 第106行的 `>>>>>>> 49d70518 (Improve i18n, theme, gateway, and sponsors)`
- 第287行的 `>>>>>>> 49d70518 (Improve i18n, theme, gateway, and sponsors)`

注意：只删除标记行，保留实际代码内容。

### 3. 静态资源清理

检查并删除（如果存在）：
```bash
assets/images/sponsors/APIKey.Fun.png
assets/images/sponsors/xingsiyan.jpg
assets/images/sponsors/  # 如果为空目录，删除
assets/images/AliPay.jpg
assets/images/wechatPay.jpg
```

## ⚠️ 约束条件

1. **单独commit**: 每个文件修改后立即commit，格式：
   ```
   清理: 删除 [文件名] 中的广告内容
   
   Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
   ```

2. **保留内容**: 
   - MIT许可证声明
   - 原项目Fork来源说明
   - 技术文档和使用说明

3. **验证要求**:
   - 修改后运行 `cargo check` 确认语法正确
   - 使用 `git diff` 确认只删除广告相关内容
   - 确保README仍然可读（章节连贯、无空白段落）

## 📤 交付物

完成后在 `.teamwork/sync/opus-to-gpt.md` 中提供：

1. **修改汇总**：每个文件的删除行数和关键内容
2. **Commit列表**：所有提交的hash和message
3. **验证结果**：
   - `cargo check` 输出
   - `git log --oneline -10` 输出
   - 广告关键词残留扫描结果：`grep -rn "sponsor\|donation\|赞助" --include="*.rs" --include="*.md" --exclude-dir=node_modules --exclude-dir=target`

## 🔍 自检清单

- [ ] 所有文档中的赞助商章节已删除
- [ ] 所有merge conflict标记已清理
- [ ] 静态广告图片已删除
- [ ] 每个修改都有独立commit
- [ ] cargo check通过
- [ ] 无广告关键词残留（node_modules除外）
- [ ] README仍然保持可读性和完整结构

---

**工作目录**: `/c/code/CodeX/Codex-Manager-CE`  
**当前分支**: `hardening/main`  
**预计commit数**: 7-10个（取决于文件实际修改情况）
