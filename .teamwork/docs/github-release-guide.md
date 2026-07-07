# GitHub Release 打包操作指南

## 📌 概述

本仓库使用 GitHub Actions 进行多平台打包，需要**手动触发**。

## 🚀 触发打包流程

### 步骤 1：访问 Actions 页面

打开浏览器访问：
```
https://github.com/CreatorEdition/Codex-Manager/actions/workflows/release-all.yml
```

或者通过仓库导航：
1. 进入 GitHub 仓库首页
2. 点击顶部 `Actions` 标签
3. 左侧选择 `release-all` workflow

### 步骤 2：点击 "Run workflow"

在页面右上角找到蓝色按钮 **"Run workflow"**，点击展开参数表单。

### 步骤 3：填写参数

表单包含 3 个参数：

| 参数 | 说明 | 示例 | 是否必填 |
|------|------|------|----------|
| `tag` | 版本标签 | `v0.3.9` | ✅ 必填 |
| `ref` | 分支/SHA | `hardening/main` 或 `main` | 可选（默认 main） |
| `prerelease` | 预发布标记 | `auto` / `true` / `false` | 可选（默认 auto） |

**参数说明**：
- **tag**: 
  - 格式：`v` + 版本号（如 `v0.3.9`）
  - 该标签会用于 Release 标题和文件名
  
- **ref**: 
  - 要打包的分支名或 commit SHA
  - 常用值：
    - `main` - 主分支
    - `hardening/main` - 强化分支
    - commit SHA（如 `09f53759`）
  
- **prerelease**:
  - `auto` - 根据版本号自动判断（推荐）
  - `true` - 强制标记为预发布版
  - `false` - 正式发布版

发布前必须确认 `docs/zh-CN/CHANGELOG.md` 存在对应版本小节，例如 `v0.3.11` 必须有 `## [0.3.11]`。Release 正文会从该小节同步；缺少时 workflow 会失败。

### 步骤 4：确认并运行

1. 检查参数填写是否正确
2. 点击绿色按钮 **"Run workflow"**
3. 页面会自动刷新，显示新的 workflow run

### 步骤 5：监控构建进度

1. 在 Actions 页面会看到新启动的 workflow
2. 点击进入可查看详细日志
3. 构建时间约 **20-40 分钟**（多平台并行）

## 📦 构建产物

### 构建平台

workflow 会构建以下平台的安装包：

- ✅ **Windows (x64)**: `.msi` 和 `.exe`
- ✅ **macOS (Intel)**: `.dmg` 和 `.app.tar.gz`
- ✅ **macOS (Apple Silicon)**: `.dmg` 和 `.app.tar.gz`
- ✅ **Linux (x64)**: `.AppImage`, `.deb`, `.rpm`

### 下载位置

构建完成后，产物会发布到：

1. **GitHub Release 页面**:
   ```
   https://github.com/CreatorEdition/Codex-Manager/releases
   ```
   在对应版本标签下下载

2. **Artifacts**（7天保留期）:
   - 在 workflow run 页面底部 `Artifacts` 区域
   - 包含所有平台的打包文件

## ⚙️ 高级配置

### Workflow 文件位置

```
.github/workflows/release-all.yml
```

### 并发控制

- 同一个 `tag` 只能有一个 workflow 运行
- 如需重新构建，需先取消正在运行的 workflow

### 环境变量

```yaml
TAURI_CLI_VERSION: 2.10.1           # Tauri CLI 版本
CARGO_TARGET_DIR: target-shared     # Rust 编译缓存目录
```

## 🔄 典型发布场景

### 场景 1：发布新版本（hardening/main 分支）

```
tag: v0.3.9
ref: hardening/main
prerelease: auto
```

### 场景 2：测试打包（不创建 Release）

先在本地创建临时标签：
```bash
git tag v0.3.9-test
git push origin v0.3.9-test
```

然后触发 workflow：
```
tag: v0.3.9-test
ref: hardening/main
prerelease: true
```

### 场景 3：正式发布（main 分支）

```
tag: v1.0.0
ref: main
prerelease: false
```

## 🛠️ 故障排查

### 问题 1：Workflow 无法启动

**可能原因**：
- 没有 Actions 权限
- 仓库禁用了 Actions

**解决方法**：
1. 检查 Settings → Actions → General
2. 确保 "Allow all actions and reusable workflows" 已启用

### 问题 2：构建失败

**常见原因**：
- Rust 编译错误
- 前端依赖安装失败
- Tauri 配置错误

**解决方法**：
1. 查看失败的 job 日志
2. 本地运行 `cargo build` 和 `pnpm build` 验证
3. 检查 `apps/src-tauri/tauri.conf.json` 配置

### 问题 3：构建超时

**原因**：GitHub Actions 免费版有 6 小时限制

**解决方法**：
- 检查是否有死循环或无限等待
- 优化依赖缓存配置

## 📝 注意事项

1. **标签命名**：
   - 必须以 `v` 开头（如 `v0.3.9`）
   - 不要重复使用已存在的标签

2. **分支选择**：
   - 确保选择的分支已推送到远程
   - 验证分支代码可以正常编译

3. **预发布标记**：
   - 预发布版会在 Release 列表中标记为 "Pre-release"
   - 生产环境建议使用 `prerelease: false`

4. **并发限制**：
   - 相同 tag 的 workflow 会互斥
   - 修改 tag 后重新触发可绕过

## 📚 相关文档

- [Tauri 打包文档](https://tauri.app/v2/guides/building/)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Release 管理最佳实践](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)

---

**文档版本**: 1.0  
**最后更新**: 2026-06-14  
**维护者**: CreatorEdition Team
