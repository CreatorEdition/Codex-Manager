# 2026-06-04 CreatorEdition Codex-Manager Fork Hardening

## 任务分配

### 主窗口：【CodeX-GPT】

- 负责总协调、变更审计、冲突处理、逐项中文 commit、最终 PR 汇总。
- 不直接盲信子代理结论；所有合并前必须读 diff 和关键源码。

### 子代理 A：广告与推广清理

- 范围：README、docs、apps/src、assets、与 sponsor/author/donation/recommendation 相关代码。
- 目标：删除或隐藏广告、赞助、打赏、作者推广和外部推荐入口，保留必要许可证和项目来源信息。
- 输出：变更文件、删除逻辑、需主窗口复核的风险点。

### 子代理 B：分页与搜索审计

- 范围：账号、API Key、请求日志、聚合 API、插件等 list/search 数据流。
- 目标：找出后端无分页、前端全量本地搜索、索引缺失和大数据量退化点。
- 输出：按优先级排序的问题清单和最小修复方案。

### 子代理 C：安全与工程门禁审计

- 范围：Web 认证、Docker 暴露、密码哈希、Tauri CSP、GitHub Actions、测试脚本、rustfmt。
- 目标：拆分第一阶段 hardening PR。
- 输出：PR 切分建议、验证命令、风险等级。

## 提交纪律

- 每个主题单独 commit。
- 广告清理、分页修复、安全修复、CI 修复不得混在同一 commit。
- 提交信息使用中文，例如：`文档: 记录 fork hardening 协作任务`。
