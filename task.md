# Codex-Manager Fork Hardening 任务记录

## 2026-06-04

### 🔄 进行中：CreatorEdition fork 第一阶段治理

目标：

- 删除 README、文档、前端页面中的广告、赞助、打赏、作者推广和外部推荐入口。
- 审计并修复账号、API Key、请求日志、聚合 API 等列表在几千条数据下的分页、搜索和排序问题。
- 建立安全与工程门禁：Web 认证默认值、Docker 暴露风险、密码哈希、CI、测试脚本一致性、rustfmt。
- 所有修改按主题单独中文 commit，避免把广告清理、分页修复、安全修复混在同一提交。

当前分支：

- 工作副本：`C:\code\CodeX\Codex-Manager-CE`
- 远端：`origin=https://github.com/CreatorEdition/Codex-Manager.git`
- 上游：`upstream=https://github.com/qxcnm/Codex-Manager.git`
- 分支：`hardening/main`

### ✅ 已完成

- 广告与推广清理：README、多语言文档、作者页、赞助设置、远程作者内容接口和静态推广图片已清理，并通过残留扫描、前端 runtime 测试、前端构建、后端 check 与相关 Rust 测试。
- Rust 格式门禁：修复 `cargo fmt --all --check` 暴露的既有格式差异，作为独立 CI/rustfmt 提交处理。
- 账号列表后端装饰优化：分页查询只按当前页账号 ID 批量读取 token、usage、metadata、subscription、模型分配和配额覆盖，避免分页后仍全量装饰。
- API Key 列表后端分页：新增 `apikey/list` 分页参数与返回 `total/page/pageSize`，并将搜索、状态筛选、成员 owner 过滤、quota 装饰下推到后端分页链路。
- API Key 前端分页接入：平台 Key 页面通过后端分页参数加载列表，搜索、状态筛选、每页数量和翻页不再依赖本地全量列表。
- 日志页 API Key lookup：新增 `apikey/lookup` 按当前页日志 Key ID 批量查询展示信息，避免日志页额外全量加载平台 Key。
- 日志页账号 lookup：新增 `account/lookup` 供管理员日志页按当前页账号 ID 批量查询名称，避免日志页额外全量加载账号列表。
- 日志页聚合 API lookup：新增 `aggregateApi/lookup` 供管理员日志页按当前页聚合 API ID 批量查询展示信息，避免日志页额外全量加载聚合 API 列表。
- 账号页前端分页接入：账号管理页通过 `account/list` 后端分页加载当前页，搜索、可用筛选和低配额筛选下推到后端，页脚和翻页使用后端返回的 `total/page/pageSize`。

### ⚠️ 待处理

- `cargo test --workspace` 尚未全量执行，后续安全/CI 阶段再跑完整工作区测试。
- 旧工作副本 `C:\code\CodeX\Codex-Manager` 仅保留为审计参考，实际修改转入 `Codex-Manager-CE`。
- 启动快照全量预载仍需单独收敛；平台 Key 页面 Token/费用统计仍沿用全量 `usageStats` 聚合。
- 账号页 `account/usage/list` 仍全量读取 usage 快照；需后续按当前页账号 ID 查询或由账号列表返回已装饰 usage。
- 账号页计划类型筛选、限流/封禁状态筛选和全局排序还缺后端分页等价能力，本次前端避免用当前页数据伪装全局筛选。
