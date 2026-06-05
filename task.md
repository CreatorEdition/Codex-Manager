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
- 账号页用量按需读取：`account/usage/list` 支持 `accountIds` 参数，账号管理页只读取当前页账号的最新 usage 快照。
- 启动快照瘦身：`startup/snapshot` 支持账号/API Key 预载限制并返回总数元数据，前端默认只预载首屏占位数据，避免几千账号时首屏全量搬运。
- 模型池接口轻量化：`quota/modelPools` 支持按页面关闭 sources/config 或限定来源类型，首页只取模型汇总，聚合 API 页只取聚合 API sources。
- 平台 Key 用量按需统计：`apikey/usageStats` 支持 `keyIds` 参数，平台 Key 页只聚合当前页 Key 的 Token/费用，member 路径也不再先全量聚合后过滤。
- 平台 Key 归属按需查询：`accountManager/apiKeyOwners/list` 支持 `keyIds`，`accountManager/users/list` 支持 `ids`，平台 Key 页首屏只读取当前页归属和相关用户，member Key ID 查询改为数据库条件过滤。
- 启动快照预取轻量化：`startup/snapshot` 支持关闭用量聚合、今日摘要、最近日志和模型目录，应用启动预取使用独立轻量缓存，避免首屏预热触发重聚合 RPC。
- 模型池来源按需查询：新增 `quota/modelPoolSources`，支持 `sourceKind/sourceIds/page/pageSize`，聚合 API 页不再通过 `quota/modelPools(includeSources:true)` 拉取全量来源明细。
- 聚合 API 列表后端分页：`aggregateApi/list` 支持 `page/pageSize/providerType/statusFilter/query`，管理页下推筛选和分页，并仅按当前页 API ID 装饰模型池来源。
- RPC 默认轻量化防护：`startup/snapshot` 裸调用默认限制账号/API Key 预载 20 条并关闭聚合、今日摘要、最近日志和模型目录；`quota/modelPools` 裸调用默认不返回来源明细和容量配置，完整读取必须显式传 include 参数。
- 管理员用量排行限载：`dashboard/adminUsageSummary` 新增 `rankingLimit`，默认仅返回 Top 8 用户、OpenAI 账号和聚合 API，并按 Top ID 读取元数据，避免首页返回全量排行。
- 模型页来源名称按需查询：模型管理页根据路由实际引用的账号和聚合 API ID 调用 lookup，不再为构建来源名称字典全量读取账号与聚合 API 列表。
- 启动快照客户端默认轻量化：`serviceClient.getStartupSnapshot()` 不再默认显式开启聚合、今日摘要、最近日志和模型目录，首页仪表盘改为显式 opt-in，并新增 runtime 回归测试。
- 账号体系用户列表分页：`accountManager/users/list` 支持 `page/pageSize` 返回分页结果，账号管理页只加载当前页用户，顶部成员总数改用后端状态字段。
- 成员仪表盘按归属 Key 聚合：`dashboard/memberSummary` 按成员 Key ID 批量查询 Key 元数据和用量聚合，避免全量 Key/模型用量扫描后本地过滤。
- 网关候选池 trace 限载：`CANDIDATE_POOL` 只输出候选总数、前 12 个短指纹样本和截断数量，不再为每次请求构造并写入全量账号池。
- 观测数据保留与 WAL 截断：高频 events 默认保留 14 天且保留账号状态事件，维护任务同时清理 events、usage snapshots、request logs/token stats，并在有变更时执行 WAL TRUNCATE checkpoint。
- 模型池 summary 避免来源全量扫描：`quota/modelPools` 裸调用只返回模型/价格 skeleton，不再为容量汇总扫描全部账号池和聚合 API；容量明细改由显式 `sourceKind/includeSources` 或分页来源接口承担。
- 用量轮询候选按批次读取：后台 `refresh_usage_for_polling_batch` 不再每轮全量加载账号、Token 和全部账号状态事件，改由 storage 层按游标分页读取本轮候选。
- Token refresh 后台任务限载：到期 Token 查询不再对全部账号状态事件做窗口排序，刷新前只按本轮 due token 批量读取账号元数据。

### ⚠️ 待处理

- `cargo test --workspace` 尚未全量执行，后续安全/CI 阶段再跑完整工作区测试。
- 旧工作副本 `C:\code\CodeX\Codex-Manager` 仅保留为审计参考，实际修改转入 `Codex-Manager-CE`。
- 账号页计划类型筛选、限流/封禁状态筛选和全局排序还缺后端分页等价能力，本次前端避免用当前页数据伪装全局筛选。
- `dashboard/adminUsageSummary` 已完成首页 TopN 限载；后续仍应拆 `dashboard/adminOverview` 与分页排行 RPC，并将 TopN/分页进一步下推到 SQL 聚合层。
- 运行版只读诊断显示 `events` / `usage_snapshots` / WAL 是体积主因；后台用量轮询和 token refresh 候选已改为按批次读取，后续还需继续审计用量刷新失败退避策略和全量 usage snapshot 查询路径。
- 首页模型池卡片在 summary 模式下容量数字会显示未知；后续如要展示容量，应通过独立轻量汇总或分页来源接口懒加载，不能回退到裸 RPC 全量扫描。
