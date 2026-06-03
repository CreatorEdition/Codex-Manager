# 2026-06-04 安全与工程门禁审计进度

## 来源

子代理 C：【CodeX-GPT 安全与工程门禁审计】

## 结论摘要

第一阶段建议拆成独立 PR/commit：

1. Web/Docker 默认鉴权 fail-closed：修 Web 无密码默认放行、Docker 默认 `0.0.0.0` 暴露。
2. 密码哈希升级：Web 访问密码和账号管理密码从盐值 SHA256 升级到 Argon2id 或 bcrypt，并兼容旧 hash。
3. Tauri CSP 与 `withGlobalTauri` 收紧：关闭全局 Tauri、收窄 CSP、处理 inline 初始化脚本。
4. PR CI 与 rustfmt 门禁：新增 pull_request/push CI，先修当前 rustfmt 差异。
5. 测试脚本文档对齐：处理 `scripts/tests/*.ps1` 文档引用但目录缺失的问题，并同步 pnpm 版本说明。
6. 敏感日志默认收口：Docker 默认关闭 trace/error stdout，对 URL query、token、cookie 等字段脱敏。
7. 敏感存储基线：后续单独处理 SQLite 中 token、API key、聚合 API secret 明文存储。

## 主窗口判断

- 前四项属于第一批可落地治理。
- 敏感存储涉及数据兼容和恢复路径，应单独排期，不和广告清理或分页修复混合。
- 所有建议合并前必须重新读源码和 diff。
