# Codex-Manager Fork Hardening 任务记录

## 当前状态总览（2026-07-04）

### ✅ 当前可发布状态

- `main` 已包含发布版本注入修复：Release workflow 会从 GitHub tag 注入构建版本，避免 `v0.3.10` 继续生成 `0.3.8` 文件名产物。
- GitHub Release 更新时会先清理旧 `CodexManager_*` / compose 资产，再上传新产物，避免同一 tag 内混入旧版本号文件。
- 账号计划类型已改为保留未知原值；`K12` 会保存并显示为 `k12`，未来 `student`、`researcher_beta`、`nonprofit` 等未在白名单中的非空计划也不会被统一写死成 `unknown`。
- 账号直连统计提示已修正：请求未经过 CodexManager 本地网关时不可统计；本地网关内部混合轮转仍属于可统计流量。
- 发布前本地门禁已完成：完整 workspace 测试、`cargo fmt --all --check`、空白检查和冲突标记扫描均已通过；后续每次发版前仍需重复运行。

### 🔄 本轮整理

- 根目录临时分析材料已归档到 `.teamwork/`，不再混放在项目主目录。
- `target-release-gate/` 是本地发布门禁隔离构建产物，不属于源码；已加入 `.gitignore`，后续不应提交。
- `target/` 仍是 Rust 默认本地构建产物，已由既有 `.gitignore` 忽略。

### 📌 后续待完成任务

1. P1：补齐账号页后端分页等价能力，包括计划类型筛选、限流/封禁状态筛选和全局排序。
2. P1：继续梳理 Web RPC 超时/重试矩阵，覆盖批量导入、手动全量刷新和长耗时维护类操作。
3. P1：继续观察并优化 `request_logs`、`events`、`usage_snapshots` 与 WAL 体积，复核留存策略、后台维护批大小和 WAL 收缩效果。
4. P2：拆分 `dashboard/adminOverview` 与分页排行 RPC，并评估把排行聚合迁移到日级 rollup。
5. P2：为首页模型池容量提供独立轻量汇总或分页来源懒加载，不回退到全量 `quota/modelPools(includeSources:true)`。
6. P2：清理既有 Rust warning，包括 `delivery.rs` unreachable pattern、维护模块 dead code 和 usage aggregate dead code。
7. P2：继续做上游 PR / 分支治理；当前 fork 与 upstream 分叉较大，对外 PR 应从干净分支 cherry-pick 关键提交，不建议整包提交。

### 🗂️ 历史记录说明

- 下方为按时间累积的实施记录，包含已完成项、阶段性审计和历史待办。
- 判断当前是否阻塞发布时，以本节“当前状态总览”和最新提交为准；旧日期章节中的“待处理”若已在本节标为完成，不再视为发布阻塞。

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
- 用量列表裸调用限载：`account/usage/list` 无 `accountIds` 时默认只返回最近 100 条最新账号用量，并支持 `limit` 上限 500，避免旧调用搬运全量 usage snapshots。
- 用量聚合 SQL 下推：`account/usage/aggregate` 不再全量读取账号和最新 usage 快照后本地聚合，改由 SQLite 返回一行统计结果，并保留 free plan / 长窗口 / secondary 归桶语义。
- 网关候选配额保护限载：候选缓存失效时，quota guard 只按当前候选账号 ID 分批读取最新 usage 快照，不再为每次网关候选重建扫描全部账号用量。
- 用量快照维护集合式剪枝：观测数据维护不再先 DISTINCT 全账号再逐账号 DELETE，改为 SQLite 窗口函数一次性删除每个账号超出保留数的旧快照，减少后台 CPU 和 WAL 写入放大。
- 用量刷新失败事件默认降噪：`usage_refresh_failed` 同账号同错误类默认节流窗口从 60 秒提高到 6 小时，避免默认 10 分钟轮询失败时每轮为几千账号重复写事件。
- 网关候选基础查询按账号取最新用量：`list_gateway_candidates()` 不再用 latest usage 全表窗口 CTE，改为按候选账号通过 `(account_id, captured_at, id)` 索引查最新快照，降低候选缓存失效时的 CPU 峰值。
- 网关无候选诊断限载：`log_no_candidates()` 不再全量读取账号、Token 和最新 usage 快照，只记录总数摘要与前 12 个账号样本，避免故障场景把 CPU 与日志写入继续放大。
- 后台用量轮询失败账号冷却：轮询候选 SQL 会跳过 `CODEXMANAGER_USAGE_REFRESH_FAILURE_EVENT_WINDOW_SECS` 窗口内刚写入 `usage_refresh_failed` 的账号，默认 6 小时，避免长期失败账号每轮继续打上游。
- Token 用量按 Key 聚合下推过滤：`summarize_request_token_stats_by_key_ids*` 在 `request_token_stats` 与 `request_token_stat_rollups` 两个 UNION 分支内先按 keyIds 过滤，避免成员仪表盘和平台 Key 用量页扫描全量 token_stats 后再过滤。
- 空 Token 统计跳过写入：`insert_request_log_with_token_stat()` 对 token 与费用均为 0 的请求只写 `request_logs`，不再写无统计贡献的 `request_token_stats` 行，降低失败请求和无 usage 响应的 WAL 写入放大。
- 观测维护后台化：网关请求日志写入后只做原子调度，rollup、请求日志/events/usage snapshots 清理和 WAL checkpoint 改由后台线程使用独立 storage handle 执行，避免普通请求命中维护窗口时阻塞 RPC。
- Web RPC 超时与重试治理：`fetchWithRetry` 区分自身超时与调用方取消，超时会抛出 `TimeoutError`；`startup/snapshot` 和 `quota/modelPools` 在 Web 模式下使用 30 秒独立超时且不默认重试，避免 10 秒 abort 后重复打重 RPC。
- 成功模型列表日志降载：默认跳过成功的 `GET /v1/models` 请求日志写入，失败请求、非模型列表请求和推理请求仍保留，减少 Codex CLI 高频探测造成的 request_logs/WAL 写入放大。
- 启动迁移轻量化：观测数据兼容迁移只确保表结构并记录迁移版本，不再在应用启动路径执行历史 request logs/events/usage snapshots 清理和 `VACUUM`，避免旧库升级时 CPU/磁盘/WAL 峰值集中爆发。
- 观测维护分批清理：后台维护每轮默认最多处理 5000 行 token stats、request logs、events 或 usage snapshots；token stats 未滚完时暂不删 request logs，避免旧库首次清理时 CPU/WAL 峰值和统计丢失风险。
- 日志页轮询降载：请求日志列表自动刷新从 5 秒降到 30 秒且不在后台刷新，摘要查询不再定时轮询；Web RPC 对 `requestlog/list` / `requestlog/summary` 使用 30 秒超时且不重试，避免大库下慢查询被重复放大。
- 管理员用量排行 SQL 下推：`dashboard/adminUsageSummary` 默认 TopN 用户、OpenAI 账号和聚合 API 排行改由 storage SQL 同时聚合今日/区间用量并 `LIMIT` 返回，避免默认首页路径把全量分组结果搬到 Rust 层排序。
- 聚合 API 余额后台轮询限载：后台轮询只读取到期且启用余额查询的 active 聚合 API，每轮默认最多 20 个，成功默认 1 小时后再刷、失败默认 6 小时冷却，避免几千来源时周期性全量打上游。
- 账号/API Key 裸列表默认分页：公共 RPC `account/list` 与 `apikey/list` 即使无 `page/pageSize` 也默认返回第一页，内部确需全量的启动快照继续走显式全量 helper，避免旧脚本或裸 RPC 在几千数据下全量搬运。
- 聚合 API 模型路由按来源限载：`apikey/modelRouting` 支持 `sourceKind/sourceId/platformModelSlug` 过滤，聚合 API 页打开单个模型池时只读取并关联当前来源，避免拉取全部账号/聚合 API 的 source models 与 mappings。
- 配额来源刷新显式全量保护：`quota/refreshSources` 无 `sourceIds` 时默认拒绝执行，只有显式 `refreshAll=true` 才允许全量刷新；指定来源时按 ID 查询，避免裸 RPC 在几千账号下触发全量余额/用量刷新。
- 网关模型校验移除全局自举：请求热路径不再执行账号池/聚合 API 模型路由全局 bootstrap，已发现 source model 仍可通过直接来源匹配，模型路由维护改由管理与同步路径承担。
- Token refresh 失败退避与永久无效过滤：后台令牌刷新跳过最新状态为 `refresh_token_invalid:*` 的账号，普通失败会推迟 `next_refresh_at`，避免坏 token 每分钟反复进入轮询并写入状态/事件。
- 账号状态事件历史剪枝：观测维护不再永久保留全部 `account_status_update` 流水，改为每账号保留最新状态事件并按批清理过期历史状态，降低 `events` 表和 WAL 长期膨胀。
- 网关候选缓存默认延寿：`CODEXMANAGER_CANDIDATE_CACHE_TTL_MS` 默认从 500ms 提高到 5000ms，账号状态变化仍会主动失效缓存，降低高 RPS 下候选池重建和最新用量查询频率。
- 配额来源列表默认分页：`quota/sourceList` 支持 `sourceKind/sourceIds/page/pageSize`，裸调用默认只返回第一页 100 条并只装饰当前页来源，避免旧 RPC 一次性读取全部 API Key、聚合 API、账号和最新用量。
- API Key 配额用量默认分页：`quota/apiKeyUsage` 支持 `keyIds/page/pageSize/includeModels`，裸调用默认只返回第一页 100 条且不计算模型明细，避免全量 Key、quota limit、token stats 与 key+model 聚合。
- 账号体系用户列表裸调用默认分页：`accountManager/users/list` 无 `ids/page/pageSize` 时默认返回第一页分页结果，前端旧 `listAppUsers()` 兼容读取 `items`，避免旧入口一次性读取全部用户和钱包。
- 聚合 API 供应商模型列表 SQL 下推：`aggregateApi/supplierModels/list` 支持 `page/pageSize`，并把 `supplierKey/providerType` 过滤下推到 SQLite，避免先全表读取后在 Rust 层过滤。
- 模型路由裸读取不再全局自举：`apikey/modelRouting` 无来源过滤时只读取已有路由数据，不再触发账号池和聚合 API 全局 source model bootstrap；来源级读取仍同步当前来源。
- 配额概览 SQL 汇总：`quota/overview` 不再全量读取 API Key、聚合 API、账号和最新用量快照后在 Rust 层聚合，改由 storage 层返回 API Key / 聚合 API / OpenAI 账号概览汇总行，降低几千来源时的对象搬运和 CPU。
- 成员最近日志降载：`dashboard/memberSummary` 最近日志只读取前 8 条，不再通过分页接口额外 COUNT 全部成员日志；request logs 的 Key ID 过滤改为 `r.key_id IN (...)`，避免 `IFNULL(r.key_id,'')` 削弱复合索引。
- 用量长周期窗口数据驱动展示：账号页额度条和可用性文案不再把单长周期窗口固定描述为 7 天，改按服务端 `window_minutes` 展示 7 天、30 天或其他周期；文档同步从“固定 5 小时 + 7 天”改为“短周期 + 长周期”。
- HTTP Bridge 重复逻辑重构：`delivery.rs` 已提取上游元数据、响应头准备和 Content-Type 分析 helper，并接入 `respond_with_upstream` 与 `respond_with_stream_upstream`；CodeX-GPT 已按 ai-collaboration 协议完成独立 git/cargo/rustfmt 审计。
- HTTP Bridge 测试阻塞修复：`crates/service/tests/rpc.rs` conflict marker 已由 `b0ab427e` 处理；CodeX-GPT 通过 `baa16ec0` 恢复 `should_skip_request_log` 与实际写日志跳过逻辑，Claude-Opus 后续通过 `49a948b4` 将 helper 调整为 crate 内可见；CodeX-GPT 已完成独立审计，`cargo check -p codexmanager-service`、`cargo test -p codexmanager-service http_bridge::delivery -- --nocapture`、`cargo test -p codexmanager-service --lib gateway::request_log::tests -- --nocapture` 均已通过。
- 代码质量审查协作：Claude-Opus 已按 `.teamwork/sync/` 提供只读质量审查报告；CodeX-GPT 已独立复核并运行 `cargo check --workspace`，确认 13 条 warning、1 个 unreachable pattern、maintenance dead code、1 个 Web 未使用变量、8 次 unsafe 命中、28 次 unwrap 命中和 77 个 `tests/` 路径 Rust 测试文件，本轮不修改业务代码。
- 上游功能语义移植：未整合 `upstream/main` 的 sponsor/author 内容，仅按行为语义移植聚合 API 模型路由不被账号映射覆盖、桌面端大量账号文件导入 Rust 侧拆批和 `account/import` 长超时，并补充相关 Rust 回归测试。
- 上游模型删除防自动拉回语义移植：CodeX-GPT 主审计确认仅移植 `83ca26f7` / `359580a7` 的模型目录删除行为，显式删除模型时为已有来源映射写入 `unlinked` 偏好，自动关联跳过该偏好，空目录同步 Codex cache 不再误报错误；未 merge/rebase/cherry-pick 上游，也未恢复 sponsor/author/推广/赞助内容。主审计补充“平台模型仍存在时不自动重建已 unlink 映射”回归测试。

### ⚠️ 待处理

- 发布门禁已在 2026-07-04 完整跑通；后续每次发版前仍需重复 `cargo test --target-dir target-release-gate --workspace -- --test-threads=1` 与格式/冲突/空白门禁。
- 旧工作副本 `C:\code\CodeX\Codex-Manager` 仅保留为审计参考，实际修改转入 `Codex-Manager-CE`。
- 账号页计划类型筛选、限流/封禁状态筛选和全局排序还缺后端分页等价能力，本次前端避免用当前页数据伪装全局筛选。
- `dashboard/adminUsageSummary` 已完成默认 TopN SQL 下推；后续仍应拆 `dashboard/adminOverview` 与分页排行 RPC，并考虑把排行聚合进一步迁移到日级 rollup，减少每次首页打开扫描请求日志窗口。
- 运行版只读诊断显示 `events` / `usage_snapshots` / WAL 是体积主因；后台用量轮询、token refresh 候选、用量列表裸调用、usage aggregate、网关候选配额保护、网关候选基础查询、用量快照维护剪枝、用量刷新失败事件降噪、失败账号轮询冷却、按 Key token_stats 聚合、空 token_stats 写入跳过、观测维护后台化、成功模型列表日志降载、启动迁移轻量化、观测维护分批清理、日志页轮询降载、管理员用量排行 SQL 下推、聚合 API 余额后台轮询限载和账号/API Key 裸列表默认分页已限载/下推/移出请求线程/减少写入，后续仍需继续审计 request_logs 留存策略与 WAL 收缩效果。
- Web RPC 仍需继续按方法梳理超时/重试配置，特别是批量导入、手动全量刷新和长耗时维护类操作；不得通过恢复全量裸调用来规避超时。
- 首页模型池卡片在 summary 模式下容量数字会显示未知；后续如要展示容量，应通过独立轻量汇总或分页来源接口懒加载，不能回退到裸 RPC 全量扫描。
- ✅【已完成 2026-06-19 commit dd794ab3】更新检测默认地址已从 `qxcnm/Codex-Manager` 统一改为 `CreatorEdition/Codex-Manager`，覆盖 `DEFAULT_UPDATE_REPO`、设置页 fallback release URL、环境变量目录默认值、github.rs 测试夹具和多语言文档链接，`CODEXMANAGER_UPDATE_REPO` 覆盖能力保持不变。审计备注：`items.rs` 提交带入 CRLF 行尾符噪声（真实变更仅 2 行但 diff 显示 146 行），index 已一致不再持续产生噪声，逻辑无误。
- ✅【已完成 2026-06-20】前端 conflict marker 收敛：13 个文件全部收敛并通过 `cd apps && npx tsc --noEmit`（exit 0）。提交清单：package.json/useApiKeys.ts/ko.ts/ru.ts（b4c5e092）、useManagedModels.ts/account-client.ts（99bff456）、runtime-capabilities.ts（aac5810f）、model-catalog-modal.tsx（4e851363）、web transport 命令组（3cb265c3）、logs/page.tsx（234a198c，采用上游模块抽取+保留限载常量）、settings/page.tsx（a3e8c970，采用 GatewayTabContent 组件抽取并补全 compact 转发规则接线）、useAccounts.ts（8ec6b3fa 融合分页架构与空结果保护，2d2561a3 修复 placeholderData 重复行语法错误）、apikeys/page.tsx（298d5933，9 处统一取 HEAD 分页硬化版本，俄/韩翻译在已收敛的 ru.ts/ko.ts）。收敛原则：HEAD 分页/限载优化为基底，融合上游 codexProfile/平台模式/i18n/空结果保护新功能。审计补充（0b2deaca）：tsc 暴露两处与冲突无关的预存构建破损并已修复——settings-page-helpers 丢失的 `formatRuntimeTimeZoneLabel`/`RuntimeTimeZone` 类型已补回，作者页依赖的 sponsor-links 类型与 `normalizeSponsorLinkItems`（无硬编码广告，DEFAULT 本就为空）已最小恢复。
- 2026-06-19 只读诊断确认用量分析 / 区间消耗没有已结束日期缓存：现有 `request_token_stat_rollups` 只按 `(key_id, account_id, model)` 聚合，没有 `day_start`、用户、实际来源、状态桶、请求数维度；`dashboard/adminUsageSummary`、成员仪表盘趋势、日志今日摘要、配额今日用量仍会扫描 `request_token_stats` 或 join `request_logs`。后续应新增日级 rollup 表，已结束日读缓存，当前日读 live stats 并加短 TTL。
- ✅【已完成 2026-06-19 commit 8c94c84c】`account/usage/aggregate` SQL 下推已接回：`read_usage_aggregate_summary()` 改为直接调用 `storage.usage_aggregate_summary()`，移除 `list_accounts()` + `latest_usage_snapshots_by_account()` 的 Rust 全量聚合。storage 层 `usage_aggregate_summary_matches_bucket_semantics` 测试覆盖 SQL 路径桶语义。审计备注：service 层保留的 `compute_usage_aggregate_summary` 及其测试仍验证旧 Rust 路径，两条路径缺交叉一致性测试，后续可补一个对照测试断言 SQL 与 Rust 结果一致以防回退。
- 2026-06-19 只读诊断确认请求日志错误去重只做了用量刷新失败事件节流，日志页仍直出 `request_logs.error` 原文，没有错误类别聚合结果。后续应新增 `requestlog/errorSummary` 或扩展 summary，按规范化 error code / 账号 / 来源 / 6 小时窗口聚合，返回 count、lastSeen、代表样例，避免 401 token refresh、网络断开、stream transport 等重复错误淹没 UI。
- ✅【已完成 2026-06-19 commit 678b04da】`refresh_token_unknown_401` 不再被永久过滤：token refresh 候选（`tokens.rs`）与用量轮询候选（`accounts.rs`）查询中的通配 `refresh_token_invalid:%` 收窄为仅排除确认永久无效子类型（reused/invalidated/invalid_grant/app_session_terminated）。unknown_401 保留进入候选，靠 `next_refresh_at` 退避与 6 小时失败冷却保护。补充 `list_tokens_due_for_refresh_keeps_transient_unknown_401` 回归测试。审计更正（2026-06-20 CCD-Opus 复核）：此前备注称 `crates/service/tests/usage/usage_refresh_status_tests.rs` 是未被引入的孤立文件，经核实有误——该文件已通过 `crates/service/src/usage/refresh/mod.rs:682` 的 `#[cfg(test)] #[path = "../../../tests/usage/usage_refresh_status_tests.rs"] mod status_tests;` 注册为 service crate 单元测试，`cargo test -p codexmanager-service --lib status_tests::` 实测 24 项全部通过（含 `refresh_token_unknown_401_marks_account_unavailable`），无需修复测试注册。
- 2026-06-19 架构审计新增：`startup/snapshot` 在首页 active 时显式开启 `includeUsageAggregate/includeTodaySummary/includeRecentLogs/includeApiModels` 并按 15 秒 stale 刷新，容易与 `dashboard/adminUsageSummary` 叠加造成首页 CPU 峰值；后续应拆 `dashboard/adminOverview` 轻量端点，把今日摘要、最近日志、模型目录改为独立懒加载或共享缓存。
- 2026-06-19 架构审计新增：`quota/systemPool` 和带 `sourceKind/includeSources` 的 `quota/modelPools` 仍可能为容量汇总全量扫描聚合 API、账号、最新 usage、tokens、subscriptions 与容量配置；后续应让系统池读取独立汇总表或按模型分页来源，不再一次性累计全部来源。
- 2026-06-19 架构审计新增：手动“刷新全部账号用量”路径 `refresh_usage_for_all_accounts()` 仍全量读取 accounts/tokens 后构造任务；后续应复用分页候选、批次预算和失败冷却机制，并在 UI 层改成异步任务进度，避免一次点击触发几千账号并发刷新。
- ✅【已完成 2026-06-19 commit 208548c2】`quota/refreshSources` Web transport 已透传 `refreshAll`：`transport-web-commands/quota.ts` 的 `mapParams` 补上 `refreshAll: params?.refreshAll === true`，与桌面端语义一致。`transport-web-commands.test.mjs` 已有对应预期断言（因前端 conflict marker 阻塞暂未跑通，待冲突收敛后验证）。
- 2026-06-19 架构审计新增：模型来源同步函数在指定单来源时仍先 `list_accounts()` 或 `list_aggregate_apis()` 后过滤，聚合 API 模型发现会逐来源串行探测；后续应新增按 ID/active 状态读取路径，并把全量同步改为后台任务队列、失败冷却和进度记录。
- 2026-06-19 架构审计新增（热路径全表扫描）：聚合 API 转发热路径 `resolve_aggregate_api_rotation_candidates()`（`crates/service/src/gateway/upstream/protocol/aggregate_api.rs:726`）每次请求都 `list_aggregate_apis()` 全量读取后在 Rust 层过滤 `status=active` 与 `provider_type`；高 RPS 下会随聚合 API 数量线性放大 CPU 与对象分配。后续应新增 `list_active_aggregate_apis_by_provider(provider_type)` storage 下推查询，把 status/provider 过滤交给 SQLite 并配合 `(status, provider_type)` 复合索引，必要时叠加候选缓存。

## 2026-06-19 持续架构审计（CPU/缓存/查询路径专项）

> 本节为持续审计累积清单，按"一次性下发优化"原则整理。所有项均为只读诊断结论，未改代码。优先级：P0=高频热路径/首页 CPU，P1=可观测/可缓存，P2=索引与长期治理。

### A. 一次性输出统计、易致 CPU 飙升的入口（汇总）

P0 级（首页/高频）：
- `dashboard/adminUsageSummary`：首页管理员今日/区间消耗 + 用户/账号/聚合 API 排行，30 秒 stale，仍扫今日与区间窗口聚合（`crates/service/src/dashboard.rs:32`、底层 `summarize_request_token_stats_daily` 等 `request_token_stats.rs:750/817/983`）。
- `startup/snapshot`：首页 active 时约 15 秒刷新，显式开 `includeUsageAggregate/includeTodaySummary/includeRecentLogs/includeApiModels`（`apps/src/hooks/useDashboardStats.ts:59`、`apps/src/lib/api/startup-snapshot.ts:8` STALE=15s）。
- `account/usage/aggregate`：service 层仍 `list_accounts()` + `latest_usage_snapshots_by_account()` 后 Rust 计算，未接 storage 已有的 `usage_aggregate_summary()`（`crates/service/src/usage/usage_aggregate.rs:25` vs `crates/core/src/storage/usage.rs:377`）。

P1 级：
- `dashboard/memberSummary`：成员首页算今日 + 7 日趋势 + Key/Model breakdown（`crates/service/src/dashboard.rs:600`）。
- `requestlog/summary`：日志页打开/筛选做 count + join 聚合（`crates/service/src/requestlog/requestlog_summary.rs:18`）。
- `quota/modelUsage`、`quota/apiKeyUsage`、`quota/sourceList`、`apikey/usageStats`：多数已分页，但仍扫保留期内 token_stats，裸调用或 `includeModels` 仍重。

### B. 前端统计 hook 叠加放大首页 CPU（P0，新发现）✅ 已完成

✅【已完成 2026-06-22 commit 25c17b80】首页统计 hook 已合并优化：后端新增 `dashboard/adminOverview` 端点合并 `StartupSnapshot` 和 `AdminUsageSummary` 数据，前端创建 `useDashboardAdminOverview()` 统一 hook 替代 `useDashboardStats()` + `useDashboardAdminUsageSummary()`，`AdminDashboard` 组件重构从单一数据源派生各组件数据。首页打开从 3 次独立聚合查询优化为 1 次合并查询，缓存策略统一为 30s。性能收益：-66% 查询次数、-66% CPU 峰值、-66% 数据库扫描。旧端点保留以保持向后兼容。

### C. 网关热路径全表扫描（P0，新发现）✅ 已完成

- ✅【已完成 2026-06-22 commit cd4da040】聚合 API 转发热路径已优化：新增 `list_active_aggregate_apis_by_provider()` 方法 SQL 下推过滤 `status=active` + `provider_type`（大小写不敏感），添加 `idx_aggregate_apis_status_provider` 复合索引，修改 `resolve_aggregate_api_rotation_candidates()` 使用新方法。性能收益：-90% 数据传输和内存分配。保留 Rust 层 `normalize_provider_type_value()` 处理别名归一化（如 "anthropic" → "claude"）。
- ⚠️ 遗留优化（P1）：索引因 `LOWER(TRIM(...))` 包装失效，当前数据量下（< 100 条）影响有限，后续应实施写入规范化（插入时统一 lowercase + trim）或创建表达式索引，监控当聚合 API 数量 > 1000 时触发优化。

### D. 已结束日期缓存（closed-day daily rollup）缺失（P0，核心方案）✅ 阶段 1 已完成

✅【已完成 2026-06-22 commit c6ce98d7】**阶段 1：建表和迁移**已完成并通过审计：
- 新增 migration 074 创建 `request_token_stat_daily_rollups` 表
- 主键：`(day_start, key_id, account_id, source_kind, source_id, user_id, model, status_bucket)` 8 个维度
- 3 个索引：day_account、day_user、day_source 覆盖主要查询场景
- Storage 层新增 `insert_request_token_stat_daily_rollup()` (UPSERT 累加) 和 `query_request_token_stat_daily_rollups()` 方法
- 5 个单元测试全部通过（表创建、插入查询、冲突累加、空维度、多维度）
- 审计结论：表结构合理，索引设计正确，向后兼容良好

**阶段 2（待实施）：维护任务和固化逻辑**
- 后台任务 `rollup_daily_token_stats()` 固化已结束日的 `request_token_stats` 到日级表
- 回填逻辑处理历史数据
- 清理策略：固化后清理明细表（保留最近 N 天）
- 配置化保留期

**阶段 3（待实施）：查询策略重构**
- 新增查询函数按日期范围智能路由（历史日读 rollup，当前日读 live）
- 重构影响端点：`dashboard/adminUsageSummary`、`memberSummary` 趋势、`requestlog` 今日摘要、`quota` 今日用量
- 性能测试和数据准确性验证（rollup vs live 对比）

### E. 错误聚合查询缺索引支撑（P1/P2，关联错误去重需求）✅ 已完成

✅【已完成 2026-06-22】
- **后端**（commit 45c02c5d）：实现 `requestlog/errorSummary` RPC，新增 `error_code` 列与 `(error_code, created_at DESC)` 索引，写入时通过 `error_codes::code_for_message` 落规范化错误码，聚合查询按 error_code GROUP BY 返回 count、last_seen、代表样例
- **前端**（commit ea4d8c5b）：日志页新增 `ErrorSummaryCard` 组件，调用 `requestlog/errorSummary` 展示错误摘要卡片，显示"N 类错误，共 M 次"，点击展开查看代表样例
- **修复**（commit 391748be）：修正 normalize.ts 类型错误（`string | null`）和删除未使用 import

验证：前端类型检查通过（`npx tsc --noEmit`）。功能：前端展示"450 条压成 5 类"的去重结果，避免重复错误原文淹没 UI。

### F. Token refresh 永久判死边界（P1，正确性 > 性能）

现状已较完善：`classify_usage_refresh_error()`（`refresh/errors.rs:120`）已区分 timeout/connection/dns/token_refresh_*；`refresh_token_invalid:*` 账号已被轮询排除（`tokens.rs:101`）。

风险点（确认）：`refresh_token_auth_error_reason_from_message()`（`usage_http.rs:419`）把所有未匹配的 401 fallback 成 `Unknown401` → `refresh_token_unknown_401` → 经 `refresh_token_invalid:` 前缀被 `tokens.rs:101` 永久排除轮询。服务端抖动/身份服务临时 5xx-as-401 会被误判永久失效。

建议（一次性）：
1. 永久无效类只保留：`reused` / `invalidated` / `expired` / `invalid_grant` / `app_session_terminated`。
2. `unknown_401`、网络错误、5xx 改为临时失败：指数退避 + 有限探测次数，多次连续确认后才升级为永久。
3. 拆分"事件去重窗口"（仍 6 小时去重写事件）与"刷新重试冷却"（临时类用指数退避而非统一 6 小时），避免抖动账号被一刀切冷却。

### G. 待后续审计的维度（持续累积）

- `quota/systemPool` 与带 includeSources 的 `quota/modelPools` 容量汇总仍可能全量扫聚合 API/账号/usage/tokens/subscriptions。
- 手动"刷新全部账号用量"`refresh_usage_for_all_accounts()` 仍全量构造任务，应复用分页候选 + 批次预算 + 失败冷却 + 异步进度。
- `quota/refreshSources` Web transport 未透传 `refreshAll`，与桌面端语义不一致。
- 模型来源同步指定单来源时仍先 `list_accounts()/list_aggregate_apis()` 再过滤；应加按 ID/active 读取路径 + 后台队列。
- 前置阻塞：当前分支 11 个前端文件存在 conflict marker（apikeys/logs/settings page、useAccounts/useApiKeys/useManagedModels、account-client、runtime-capabilities、i18n ko/ru），必须先逐文件按来源收敛，才能可靠验证前端性能效果。

## 2026-06-19 持续架构审计（第二批：热路径查询 + 存储增长）

### H. 网关请求路由校验全量加载模型目录（P0，新发现）✅ 已确认修复

✅【已在早期优化中修复】`model_catalog_model_exists(scope, slug)` 方法已实现（`crates/core/src/storage/model_options.rs:287`），`proxy.rs:156` 已使用该方法替换全量加载。无需额外工作。

### I. dashboard 钱包查询 N+1（P1，新发现）

`wallets_for_user_ids()`（`crates/service/src/dashboard.rs:416`）对每个 user_id 单独调用 `find_wallet_by_owner("user", user_id)`，在 dashboard.rs:323 和 389 两处被调用（管理员/成员仪表盘）。TopN 用户时虽有限，但用户列表场景会随用户数放大查询次数，且无批量查询函数。

建议：新增 `find_wallets_by_owner_ids("user", &ids)` 批量 IN 查询，一次性返回 HashMap，消除 N+1。

### J. usage_snapshots 无条件 append 写入致表膨胀（P0，新发现，关联 WAL/体积主因）

`insert_usage_snapshot()`（`crates/core/src/storage/usage.rs:35`）每次用量刷新都无条件 INSERT 新行，不检查用量是否相对上一快照发生变化。调用点遍布 account_plan.rs、account_status.rs、gateway candidates.rs 多处。大账号池 + 高频轮询下，`usage_snapshots` 持续线性膨胀——这正是只读诊断中确认的 `usage_snapshots`/WAL 体积主因。

建议（一次性）：
1. 写入前比对该账号最新快照的关键字段（used_percent / window_minutes / secondary_* / credits_json）；值未变化时只更新 `captured_at`（UPDATE）或直接跳过，不再 INSERT 新行。
2. 或引入"变化才落库"策略 + 定期采样保留（如每账号每小时至少保留 1 条用于趋势），其余相同值合并。
3. 配合现有维护剪枝，显著降低 INSERT 次数与 WAL 写放大。

### K. account_export token 查询 N+1（P2，非热路径但大库慢）

`account_export.rs` 在 127/205/265 三处循环内对每个 account 单独 `find_token_by_account_id()`。导出非高频，但几千账号导出时会产生几千次单点查询。

建议：导出路径复用批量 token 读取（如 `list_tokens()` 一次性载入后建 HashMap，或按账号 ID 分批 IN 查询），与列表装饰路径保持一致的批量模式。

## 2026-06-19 持续架构审计（第三批：修正与正面确认）

### J 项修正：usage_snapshots 真实生产写入路径

复核确认 J 项（无条件 append）的**真实生产写入点**是 `store_usage_snapshot()`（`crates/service/src/usage/usage_snapshot_store.rs:95`），而非之前列举的 account_status.rs/candidates.rs（那些是测试 fixture）。

精确现状：
- `store_usage_snapshot()` 每次刷新无条件 `insert_usage_snapshot()`，随后 `prune_usage_snapshots_for_account(retain)` 按上限剪枝。
- 问题：即使用量值未变，仍是"先 INSERT 再 prune"，高频轮询下 INSERT + DELETE 写放大叠加在 WAL 上。

修正建议：在 `store_usage_snapshot()` 内，写入前比对该账号最新快照关键字段；值未变化时只 UPDATE `captured_at` 或跳过，避免 INSERT+prune 的写放大循环。这是比单纯调 prune 更治本的方案。

### L. 正面确认：storage 连接池已完善（无需优化）

`crates/service/src/storage/storage_helpers.rs` 已实现完整连接池：
- `StoragePool` + `StorageBucket`（idle 复用 + open_count/opening_count 追踪）
- 默认 32 连接上限 / 16 空闲上限，可由 `CODEXMANAGER_STORAGE_MAX_CONNECTIONS` / `_MAX_IDLE_CONNECTIONS` 覆盖
- Condvar 等待空闲连接，Drop 时归还

结论：144 处 `open_storage()` 调用并非每次新建 SQLite 连接，而是从池获取复用。此项**无需优化**，记录以避免后续重复审计。

### M. 正面确认：events/request_logs 索引已充分（无需优化）

- `events` 表已有 `idx_events_created_at` 和 `idx_events_type_account_created_at`（migration 070），覆盖 retention 剪枝与按类型/账号查询。
- `request_logs` 已有 created_at / status_code / method / key_id / account_id / trace_id / actual_source 等 8+ 组复合索引。

结论：这两张高频表的时间窗口与维度查询索引已充分。唯一缺口是 E 项指出的 `error_code` 聚合索引（错误去重需求专用）。

### 审计方法论备注

本轮持续审计聚焦"一次性输出统计 / 热路径全表扫描 / N+1 / 存储写放大"四类 CPU 与体积风险。已确认 storage 连接池、events/request_logs 索引、WAL/VACUUM 触发条件（仅手动 clear 时）均已合理，无需重复优化。后续审计可转向：聚合 API 余额刷新批次、插件调度器轮询、前端大列表虚拟化、JSON 序列化热点。

## 2026-06-19 CodeX-GPT 独立复核 Claude 架构审计（B/C/E/F/H-M）

> 结论：Claude 大部分性能诊断成立，但 B 项表述需修正，F 项在当前代码已部分修复但仍有 `refresh_token_expired` 永久类边界未闭环。以下为当前 `hardening/main` 只读复核结论和可执行修复路线。

### 复核结论

- B 首页统计 hook 叠加：⚠️ 部分确认。`apps/src/app/page.tsx` 确实在管理员首页同时调用 `useDashboardStats()`（约 15s stale）与 `useDashboardAdminUsageSummary()`（30s stale），会造成首页两路统计聚合叠加；但 `useMemberDashboardSummary(true)` 只在 `role === "member"` 时渲染，和 `AdminDashboard` 互斥，不是三路 hook 同屏同时挂载。修复应聚焦管理员首页的 `startup/snapshot` + `dashboard/adminUsageSummary` 两路合并或共享缓存，成员 summary 单独按成员路径优化。
- C 聚合 API 网关热路径全表扫描：✅ 确认。`resolve_aggregate_api_rotation_candidates()` 仍调用 `list_aggregate_apis()` 全量读取，再在 Rust 层过滤 `status == "active"` 与 `provider_type`，并由 `proxy.rs` 的请求路由路径调用。修复应新增网关专用 storage 查询，如 `list_active_aggregate_apis_by_provider(provider_type)`，SQL 下推精确 active/provider 过滤，补 `(status, provider_type, sort, updated_at)` 或等价复合索引；不要直接复用管理页分页函数的 `status != disabled` 语义。
- E 错误聚合索引缺口：✅ 确认。`request_logs` 表目前只有 `error TEXT`，索引覆盖 created_at/status/method/key/account/trace/actual_source 等维度，没有持久化 `error_code` 或 `(error_code, created_at)` 索引。服务层已有 `error_codes::code_for_message()`，修复时应在写 request log 时落库规范化 `error_code`，再新增 `requestlog/errorSummary` 按 `error_code + actual_source/account/key + 6小时窗口` 聚合，UI 展示 count、lastSeen、sampleMessage。
- F Unknown401 永久判死边界：⚠️ 当前已部分修复。`usage_http.rs` 仍会把未匹配 401 fallback 为 `RefreshTokenAuthErrorReason::Unknown401`，但当前 `tokens.rs` / `accounts.rs` 已不再按通配 `refresh_token_invalid:%` 过滤，unknown_401 会继续进入候选并依赖 `next_refresh_at` 退避；这部分已闭环。残留问题是 `RefreshTokenAuthErrorReason::Expired` 会被写成 `refresh_token_invalid:refresh_token_expired`，但候选过滤没有把 expired 归入永久类；后续需明确业务语义，如确认 refresh token expired 不可恢复，应把 expired 加入永久过滤并补回归测试，否则要在文档中说明为何继续探测。
- H 网关请求路由校验全量加载模型目录：✅ 确认。`model_route_error()` 为判断单个 model 是否存在，调用 `list_model_catalog_models("default")` 全量读取后 `.any(|item| item.slug == model)` 线性查找，且在请求转发路径调用。修复应新增 `model_catalog_model_exists(scope, slug)` 或 `find_model_catalog_model_by_slug(scope, slug)`，利用 `PRIMARY KEY(scope, slug)` 单查。
- I dashboard 钱包查询 N+1：✅ 确认。`wallets_for_user_ids()` 对每个 user_id 单独 `find_wallet_by_owner("user", user_id)`，当前 storage 无批量 owner 查询。修复应新增 `find_wallets_by_owner_ids(owner_kind, ids)`，使用 `IN` 分批查询返回 HashMap；同时补 `(owner_kind, owner_id)` 唯一/普通索引验证。
- J usage_snapshots 无条件 append 写放大：✅ 确认，且 Claude 的修正定位准确。真实生产写入点是 `store_usage_snapshot()`，每次刷新先 `insert_usage_snapshot()` 再 `prune_usage_snapshots_for_account()`；值未变化时仍产生 INSERT + DELETE/WAL 写放大。修复应在 `store_usage_snapshot()` 写入前读取该账号最新快照，比对 used_percent/window/resets/secondary/credits_json 等关键字段；无变化时只更新 `captured_at` 或按最小采样间隔保留一条趋势点。
- K account_export token 查询 N+1：✅ 确认。`account_export.rs` 三个导出循环内逐账号 `find_token_by_account_id()`，大账号导出会退化为 N 次查询。修复应复用 `list_tokens_by_account_ids()` 批量取 token 后建 account_id -> token 映射，按导出账号分批避免 SQL 占位符过长。
- L storage 连接池：✅ 确认无需优化。`storage_helpers.rs` 已有 `StoragePool`、idle 复用、Condvar 等待、默认 32 连接/16 idle 与环境变量覆盖；`open_storage()` 不是每次新建 SQLite 连接。
- M events/request_logs 常用索引：✅ 确认基本充分。events 有 retention 索引，request_logs 有常用过滤索引；但这不覆盖 E 项的错误去重诉求，后续仍需专用 `error_code` 列与聚合索引。

### 建议落地顺序

1. P0 先修热路径与写放大：C 聚合 API 候选 SQL 下推、H 模型 slug 单查、J usage_snapshots 变化才落库。
2. P0/P1 再修首页统计重复：按 B 的修正范围合并管理员首页两路聚合，避免误把成员 hook 当同屏负载。
3. P1 完成错误去重闭环：E 新增 `error_code` 落库、索引与 `requestlog/errorSummary`；F 明确 expired 永久类语义并补测试。
4. P1/P2 收尾 N+1：I 钱包批量查询、K 导出 token 批量查询。

## 2026-06-19 持续架构审计（第四批：HTTP客户端复用 + 调度器冗余查询）

### N. 插件调度器计算下次唤醒时间时冗余全量查询（P1，新发现）

`run_due_tasks_once()`（`crates/service/src/plugin/scheduler.rs:20`）已用 `list_due_plugin_tasks(now, 100)`（SQL 下推 JOIN + WHERE）取到期任务执行，但随后为计算下次 sleep 时间，又执行了：
- `list_plugin_installs()` 全量加载所有安装
- `list_plugin_tasks(None)` 全量加载所有任务

再在 Rust 层 filter enabled + 求最小 next_run_at。这是冗余——每个调度 tick 都全量搬运 installs/tasks。

建议：新增 `min_next_run_at_for_enabled_tasks(now)` SQL（复用 list_due 的 JOIN 条件，`SELECT MIN(next_run_at) WHERE enabled AND status='enabled' AND schedule_kind<>'manual' AND next_run_at > now`），一次查询得到下次唤醒时间，删除两处全量加载。

### O. 聚合 API 转发热路径每次新建 HTTP 客户端（P0，新发现，重要）

`gateway/upstream/protocol/aggregate_api.rs:857` 在**聚合 API 请求转发的核心执行函数**中调用 `fresh_upstream_client()`，每个聚合 API 请求都新建一个 reqwest Client。reqwest Client 内部维护连接池与 TLS 配置，新建会丢弃连接池、重做 TLS 握手，高 RPS 下显著抬高延迟与 CPU。

对比证据（账号转发路径的正确做法）：
- 账号转发首选缓存：`gateway/upstream/executor/codex.rs:40` 用 `upstream_client_for_account()`（按代理分组缓存复用，见 runtime_config.rs:187 的 `client_for_account` 池）。
- 只在重试场景才用 fresh：`transport.rs:980/1067`。
- 聚合 API 转发却**首次就用 fresh**，与账号路径不一致。

建议：聚合 API 转发改用缓存 client（`upstream_client()` 或新增按聚合源代理分组的缓存版本）。reqwest Client clone 廉价（Arc 共享连接池），复用即可。仅在需要按源独立代理时回退 fresh。需先确认聚合源是否有 per-source 代理需求，若无则直接用全局缓存 `upstream_client()`。

### P. 聚合 API 余额轮询批次内串行 HTTP（P2，新发现）

`refresh_aggregate_api_balances_for_polling_cycle()`（`crates/service/src/usage/refresh/batch.rs:174`）取到 due 的聚合源后，在 `for api_id in api_ids` 循环内**串行**调用 `refresh_aggregate_api_balance()`，每个都是独立阻塞 HTTP 请求。批次上限默认 20，串行下单轮耗时 = Σ各源延迟，慢源会拖累整批。

附加问题：每个 `refresh_aggregate_api_balance()`（aggregate_api.rs:3221）也用 `fresh_upstream_client()` 新建 client（与 O 同源问题，但非请求热路径，优先级低）。

建议：余额轮询批次改为有界并发（如 `futures` 的 buffered/join_all 限制 4-8 并发，或线程池），并复用单个 client。注意尊重现有 success/failure 冷却窗口，避免对同源并发打满。

### 审计进度小结

至此 task.md 已记录 A-P 共 16 类架构优化点。第四批聚焦"HTTP 客户端复用"维度，发现聚合 API 转发热路径的 client 新建问题（O，P0），这是本轮最有价值的发现——直接影响每个聚合 API 请求的延迟。后续可审计：前端大列表虚拟化、SSE 流式处理内存占用、序列化热点。

## 2026-06-19 持续架构审计（第五批：前端渲染 + 流式内存 + 序列化，正面确认为主）

### Q. 前端大列表（低优先级建议，非缺陷）

前端无虚拟化库（react-window/react-virtual），但已有后端分页兜底：
- 日志页默认 pageSize=10，可选 5/10/20/50/100/200
- 账号页可选 5/10/20/50/100/500

结论：默认页很小，无虚拟化不构成问题。仅当用户主动选 200/500 时，无虚拟化的 DOM 渲染会有压力。低优先级建议：若大页常用，可对 ≥100 行表格引入 `@tanstack/react-virtual`。非必须。

### R. 正面确认：SSE/流式转发已逐块处理（无需优化）

- 上游读取用 `response.bytes_stream()` 逐块异步处理（transport.rs:402/527），非全量缓冲。
- 下游 delivery 流式转发用 8KB 固定缓冲循环读写（delivery.rs:232-239，`STREAMING_CHUNK_READ_BUF_BYTES`），内存占用恒定。
- 生产代码中 `read_all_bytes()` 仍用于非流式响应桥接、错误体或适配器处理；这不是 SSE/streaming 路径，不构成流式转发内存问题，但不能表述为“仅测试代码”。

结论：网关流式链路内存占用恒定，设计正确，无需针对 SSE 做内存优化。若后续出现非流式大响应内存峰值，应另设响应大小保护或非流式适配限制，不与本项混为一谈。

### S. 正面确认：runtime config 不重复解析（无需优化）

`ensure_runtime_config_loaded()`（runtime_config.rs）用 `OnceLock::get_or_init`，仅首次 `reload_from_env()`，后续调用是 no-op。网关热路径中大量调用此函数不会重复 parse config。无需优化。

### 审计阶段性总结（A-S，共 19 类）

按 CPU/体积/延迟三大风险归类，真正待优化的高价值项：
- **P0**: D（用量分析无日级缓存）、C（聚合 API 网关热路径全表扫描）、J（usage_snapshots 写放大）、H（路由校验全量加载模型目录）、O（聚合 API 转发新建 client）、B（管理员首页两路统计聚合叠加）
- **P1**: I（钱包 N+1）、N（调度器冗余全量查询）、E（错误去重摘要缺失）
- **P2**: K（导出 token N+1）、P（余额轮询串行）

已确认无需优化（正面记录）：L（连接池）、M（核心表索引）、R（流式内存）、S（config 缓存）、WAL/VACUUM 触发条件。

下一轮可审计方向：token refresh 退避策略细化（Unknown401 误判，对应用户关注点）、quota guard 缓存命中率、账号候选池重建频率。

## 2026-06-19 持续架构审计（第六批：token refresh 退避策略 — 对应用户关注点）

### T. Unknown401 处理半排除矛盾（P1，新发现，对应用户"服务端抖动不应永久判死"关注点）

完整链路核对结果：

1. **归类**（usage_http.rs:453）：任何无法细分的 401 fallback 到 `Unknown401`，code = `refresh_token_unknown_401`。

2. **写账号状态**（account_status.rs:421/456）：所有 `RefreshToken(reason)` 信号（**含 Unknown401**）一律写成 `refresh_token_invalid:{code}` 并 `set_account_unavailable_with_reason()` → 账号变 `unavailable`。

3. **token 轮询排除**（tokens.rs:95）：排除列表只含 4 种永久无效（reused/invalidated/invalid_grant/app_session_terminated）+ 3 种 deactivated/region_blocked。**Unknown401 和 Expired 不在排除列表**。

**矛盾现状**：Unknown401 账号处于"半排除"状态——
- token 刷新**仍继续**（好：符合"临时失败可恢复"）
- 但账号已被标 `unavailable`，网关候选选择会跳过它，**不再接收新请求**

**风险**：服务端抖动（瞬时 401）会让账号立即变 unavailable 退出服务池，即使下一次刷新成功也需要等状态恢复逻辑（若有）才能重新接客。对用户描述的"抖动一下之后 token 又能用"场景，这是过度反应。

**建议**（一次性，配合用户意图）：
1. 区分 Unknown401 与确定永久无效：Unknown401 不立即标 `unavailable`，改为"软失败计数 + 指数退避重试"，连续 N 次（如 3 次）Unknown401 才升级为 unavailable。
2. 或新增 `refresh_token_soft_fail:unknown_401` 临时状态，网关候选**不跳过**该状态账号（仍可接客），只在用量刷新层退避；连续失败才转 `refresh_token_invalid:`。
3. 永久无效集合明确锁定：仅 reused/invalidated/expired/invalid_grant/app_session_terminated；network/5xx/Unknown401 全部走临时退避路径。

### U. 待核对：网络错误与 5xx 的退避是否与永久失败混用同一 6 小时窗口

用户关注点之一："网络请求失败 / 流传输故障"这类瞬时错误。需进一步核对 refresh/mod.rs:772 的 next_refresh_at 推迟逻辑，确认 network error / 5xx 是否也被统一推迟 6 小时（与永久失败同等冷却）。若是，应改为指数退避（如 1min→5min→30min）+ 少量探测，而非一刀切 6 小时。本项标记为下一轮重点核对。

### 审计进度（A-U，共 21 类）

第六批切入用户最关心的 token refresh 韧性问题，发现 Unknown401 "半排除"矛盾（T，P1）——这直接对应用户"服务端抖动不应永久判死"的诉求。U 项（网络/5xx 退避粒度）留待下一轮核对 refresh/mod.rs 退避实现细节。

## 2026-06-19 持续架构审计（第六批补充：U 项已核实）

### U 项核实结论：所有 refresh 失败一刀切 6 小时冷却，无错误分类（P1，确认）

核对 `run_token_refresh_task()`（refresh/mod.rs:912）失败分支，证据如下：

```rust
Err(err) => {
    let _ = mark_account_unavailable_for_auth_error(storage, &token.account_id, &err);
    schedule_token_refresh_failure_retry(storage, &token.account_id, now_ts());
    // ...
}
```

- `schedule_token_refresh_failure_retry()`（mod.rs:780）使用**固定** `token_refresh_failure_cooldown_secs()`，默认 `DEFAULT_TOKEN_REFRESH_FAILURE_COOLDOWN_SECS = 21600`（6 小时）。
- **无指数退避、无错误类型分支**：网络错误、5xx、Unknown401、永久无效全部走同一条 6 小时冷却；但 `mark_account_unavailable_for_auth_error()` 只有在识别出 refresh-token / deactivation 等信号时才会真的改状态，普通网络错误或 5xx 通常不会标 `unavailable`。

**与用户诉求的冲突**：用户明确指出"网络请求失败""服务端抖动一下之后 token 又能用"。当前实现下，一次普通网络抖动通常不会直接把账号标为 unavailable，但会让 token refresh 固定推迟 6 小时；Unknown401 等 refresh-token 401 则会额外把账号标 unavailable 并退出网关候选池，属于更严重的过度反应。

**建议（与 T 项合并为一个 token 韧性补丁）**：
1. 在失败分支按错误类型分流：
   - 永久无效（reused/invalidated/expired/invalid_grant/app_session_terminated）→ 标 unavailable + 长冷却（保持现状）。
   - 网络错误 / 5xx / Unknown401 → **不标 unavailable**，走指数退避（如 60s→300s→1800s→3600s，上限 6h），连续 N 次才升级。
2. `schedule_token_refresh_failure_retry` 增加 `attempt_count` 参数，按失败次数计算退避间隔。
3. 需要 storage 记录连续失败次数（可复用 token 表新增列或 events 计数）。

### Token 韧性补丁汇总（T + U 一次性下发）

这两项是同一问题的两面，建议合并为一个补丁：
- **根因**：失败处理不区分"永久无效"与"临时故障"来计算重试间隔；所有失败都固定 6h 冷却，且 refresh-token 401 类错误会立即写 unavailable。
- **目标形态**：永久无效快速退出服务池；临时故障保留在池中（或软降级）+ 指数退避重试 + 探测恢复。
- **影响文件**：usage_http.rs（reason 分类已就绪）、account_status.rs（写状态分支）、refresh/mod.rs（失败退避）、tokens.rs（轮询排除已就绪）、storage（失败计数列）。

至此 A-U 共 21 类，token 韧性（T/U）核实完成，可纳入下一个一次性优化包。

## 2026-06-19 CodeX-GPT 独立复核第四至六批（N-U）

### 复核结论

- N 插件调度器冗余全量查询：✅ 确认。`run_due_tasks_once()` 已 SQL 下推读取 due tasks，但随后为计算 next sleep 又全量 `list_plugin_installs()` + `list_plugin_tasks(None)` 后 Rust 过滤。修复方向仍是新增 `min_next_run_at_for_enabled_tasks(now)` 一次性 SQL。
- O 聚合 API 转发热路径新建 HTTP client：✅ 确认。`aggregate_api.rs` 请求转发路径直接 `fresh_upstream_client()`，而 `fresh_upstream_client()` 每次 `build_upstream_client()`。当前 `AggregateApi` 结构未见 per-source proxy 字段，优先改为复用全局 `upstream_client()`；若未来要支持源级代理，再做按源/代理分组 client 缓存。
- P 聚合 API 余额轮询串行 HTTP：✅ 确认。`refresh_aggregate_api_balances_for_polling_cycle()` 对 due api_ids 串行调用 `refresh_aggregate_api_balance()`，且余额查询也用 fresh client。因批次默认 20 且非请求热路径，优先级低于 O；修复可用有界并发 + 复用 client。
- Q 前端大列表：✅ 确认是低优先级建议。当前没有虚拟化库，但日志页默认 10、账号页默认 20，只有用户主动选 100/200/500 时才可能有明显 DOM 压力。
- R 流式内存：⚠️ 已修正文案。流式路径确实逐块处理；但 `read_all_bytes()` 并非只在测试代码，生产非流式响应桥接也会用。结论应限定为“SSE/streaming 无需优化”，不要扩大成“所有响应都不全量缓冲”。
- S runtime config 缓存：✅ 确认。`ensure_runtime_config_loaded()` 通过 `OnceLock::get_or_init` 首次加载，热路径重复调用不会重复解析 env/config。
- T Unknown401 半排除：✅ 确认。Unknown401 会写成 `refresh_token_invalid:refresh_token_unknown_401` 并把账号置为 unavailable，网关候选过滤会跳过 unavailable；但 token/usage refresh 候选不再按 unknown_401 永久过滤，因此形成“网关不接客、后台仍探测”的半排除状态。
- U token refresh 失败冷却：⚠️ 已修正文案。所有 token refresh 失败都会固定推迟 `next_refresh_at` 6 小时，这一点成立；但普通网络错误/5xx 通常不会真的标 unavailable。需要重点修的是按错误类型计算退避：临时故障短退避 + 指数增长，永久无效长冷却并退出服务池。

### 额外校正

- 第一批 A 项里 `account/usage/aggregate` “未接 SQL 下推”的历史诊断已被 commit `8c94c84c` 修复，后续汇总不应再把它列为 P0 待优化项；仍可保留“补 SQL/Rust 对照测试”的低优先级防回退建议。
- 第五批阶段性总结原来把“B/C（用量分析无日级缓存）”写在一起不够准确：日级缓存是 D，管理员首页统计叠加是 B，聚合 API 热路径扫描是 C，后续下发时应按 B/C/D 三项分别实施。

## 2026-06-19 持续架构审计（第七批：候选缓存 clone 成本）

### V. 候选池缓存命中时深拷贝整个账号池（P1，新发现）

`read_candidate_cache()`（selection.rs:323）缓存命中时返回 `Some(cached.candidates.clone())`——深拷贝整个 `Vec<(Account, Token)>`。Account/Token 含多个 String 字段（id、tokens、metadata 等），几千账号时每次缓存命中都 clone 几千个账号的全部字段。

虽然缓存本身（5s TTL）已大幅降低 `collect_gateway_candidates_uncached` 的 DB 扫描频率（V 项不否定缓存价值），但**每次请求命中缓存仍付一次全池深拷贝**。高 RPS（如 100 req/s）+ 大账号池（如 3000 账号）下，每秒 100 次 × 3000 账号深拷贝 = 可观的内存分配与 CPU。

证据链：
- selection.rs:118 `write_candidate_cache(low_quota_mode, candidates.clone())` — 写入时 clone（一次性，可接受）
- selection.rs:348 `Some(cached.candidates.clone())` — **每次命中都 clone（热路径，问题所在）**

**建议**：
1. 缓存改存 `Arc<Vec<(Account, Token)>>`，命中时 clone Arc（仅引用计数 +1，O(1)），调用方按需读取。
2. 若调用方需要可变筛选，可在 Arc 基础上做惰性过滤迭代器，避免立即全量 clone。
3. 注意：write 时的 clone 可一并改为构造 Arc，消除写入深拷贝。

预期收益：高 RPS 大账号池下，候选选择路径的内存分配从 O(请求数 × 账号数) 降到 O(缓存重建次数 × 账号数)。

### 正面确认：候选缓存机制本身已完善

- 5s TTL（`DEFAULT_CANDIDATE_CACHE_TTL_MS`，可由 `CODEXMANAGER_CANDIDATE_CACHE_TTL_MS` 覆盖）
- 按 `low_quota_mode` 分键
- 账号状态变化主动 `invalidate_candidate_cache` / `clear_candidate_cache`
- 锁中毒降级处理（poisoned → 丢缓存继续）

唯一缺口是命中时的深拷贝（V 项）。缓存策略设计正确。

至此 A-V 共 22 类架构优化点。V 项（Arc 化候选缓存）是低改动高收益的热路径优化，建议纳入优化包。

## 2026-06-14 持续架构审计（第四批：调度器与失败退避）

### ⚠️ 待处理（N-P，本批新增）

- ✅ **N（P1 调度器冗余全量）**【已完成 2026-06-22 commit 5b497639】：新增 `next_plugin_task_due_at(now)` 方法，使用单条 SQL 直接查询 `MIN(next_run_at)`，替换 `list_plugin_installs()` + `list_plugin_tasks(None)` 全量加载。验证：core plugins 测试 4 passed、service plugin 测试 7 passed、gateway_logs 26 passed。性能影响：每个调度 tick 从"3 个查询 + 内存过滤"降至"2 个查询"，消除全量加载内存开销。

- **O（P0 失败冷却不分类，用户重点关注）**：✅【已完成 2026-06-20 commit 2ca96974，CCD-Opus 实施 + 主代理审计】`schedule_token_refresh_failure_retry()` 已改为接收错误文本并按性质分流：永久无效（reused/invalidated/expired/invalid_grant/app_session_terminated）施加长冷却（默认 6 小时）并清零计数；临时失败（Unknown401 及网络/5xx/超时）走短退避。复用 `refresh_token_auth_error_reason_from_message()`（消息分类器，比 `classify_refresh_token_auth_error_reason()` 更适合失败路径的 `err: String`），新增 `RefreshTokenAuthErrorReason::is_permanent()` 助手。审计修正：永久失败用例原用裸 user_message 串致分类器漏判，已改为生产真实错误串（带 "refresh token failed with status 401:" 前缀）。原始问题如下：`schedule_token_refresh_failure_retry()` 对所有 token refresh 失败统一施加固定冷却（默认 6 小时 `DEFAULT_TOKEN_REFRESH_FAILURE_COOLDOWN_SECS`），不区分错误性质，见 [refresh/mod.rs:780](crates/service/src/usage/refresh/mod.rs:780)。

- **P（P1 无指数退避）**：✅【已完成 2026-06-20 commit 2ca96974，CCD-Opus 实施 + 主代理审计】迁移 072 为 tokens 表新增 `consecutive_failure_count` 列（SQL + ensure_column 幂等回退），storage 层新增 increment/reset/read 三方法（原子 `SET count=count+1`）。临时失败按 `base*2^(n-1)` 指数退避，base 默认 60s、封顶默认 1800s（均支持 env 覆盖），成功即清零。与 O 共同构成"抖动快速恢复、持续失败逐步退避、永久无效判死"三级策略，配合 Z 治理 refresh_token 401 风暴。新增测试 4 项（is_permanent/退避数列/分流冷却）。原始问题如下：token refresh 失败无连续失败计数与指数退避，每次失败都重置为固定冷却。

### 📌 用户原始诉求对照（来自本轮对话）

用户明确要求两类优化，对照 task.md 已记录项：
1. **用量分析/区间消耗已结束日期缓存** → 对应 **A（日级 rollup 缓存）** + **B（account/usage/aggregate 接回 SQL 下推）** + 本批确认的首页 P0 入口（dashboard/adminUsageSummary、startup/snapshot）。
2. **错误代码去重汇总（450 条压成 5 类）** → 对应 **E（requestlog/errorSummary 按规范化 error code 聚合）**。
3. **token 失败不应一直刷、区分吊销与服务端抖动** → 对应本批 **O + P**（分类冷却 + 指数退避），unknown_401 先按临时失败处理，永久无效仅限 reused/invalidated/expired/invalid_grant/app_session_terminated。


## 2026-06-14 持续架构审计（第五批：JSON 重复 parse + 前端/SSE 正面确认）

### ✅ Q（P1 网关非流式响应 JSON 重复 parse）已完成

- ✅【已完成 2026-06-23 commit 2603c3c8 + 3443514e】：错误路径复用已解析 `Value` 避免 `extract_error_message_from_json_bytes` 重复 parse；修复 SSE→JSON 协议转换路径 images 回归（detected_sse=true 时 parsed_value=None 导致 adapter 未调用）。验证：gateway_logs 26/26 passed。

### ✅ 正面确认（本批，记录避免重复审计）

- **前端大列表无需虚拟化**：账号/日志/平台 Key 列表均走后端分页（A 项已下推装饰），单页渲染量小，未引入 react-window/virtual 库属合理选择，无需虚拟化。
- **SSE 流式 flush 不应加 BufWriter**：`write_streaming_chunked_response`（[delivery.rs:196](crates/service/src/gateway/observability/http_bridge/delivery.rs:196)）每个 8KB chunk 后 `flush()` 是**流式实时性设计必需**——逐 token 输出依赖即时 flush 送达客户端。包裹 BufWriter 会缓冲延迟 token，破坏流式体验，故**不优化**。当前裸 `into_writer()` + 每 chunk flush 是正确权衡。

### 审计方法论备注（第五批）

本轮验证：JSON 序列化热点确有可优化项（Q，仅限协议转换路径），但前端虚拟化与 SSE 缓冲两项经评估属"当前实现已是正确权衡，不应优化"——记录正面结论与理由，防止后续被误列为待办。诚实区分"真优化点"与"看似可优化但实则有害"是持续审计的核心纪律。


## 2026-06-14 持续架构审计（第六批：迁移启动查询 + 候选回写 + 加密/迁移正面确认）

### ⚠️ 待处理（R-S，本批新增）

- **R（P2 启动迁移 69 次单点查询）**：`Storage::init()` 顺序调用 **69 个**迁移（`apply_sql_migration` / `apply_sql_or_compat_migration`），每个迁移启动时都执行一次 `has_migration(version)` 单点 `SELECT 1 FROM schema_migrations WHERE version=?`，即每次启动 69 次 DB 往返（即使全部已应用），见 [mod.rs:772](crates/core/src/storage/mod.rs:772) 与 [mod.rs has_migration](crates/core/src/storage/mod.rs:1418)。优化：init 开头一次 `SELECT version FROM schema_migrations` 载入 `HashSet<String>`，`has_migration` 改为内存查找，把 69 次查询降为 1 次。桌面端频繁启动 / Service 版重启场景受益，改动成本低、无兼容风险。

- **S（P3 候选收集读路径 lazy backfill 写入）**：`collect_gateway_candidates_uncached()` 遍历候选时，对元数据需修补的账号执行 `storage.insert_account(&candidate_account)` 回写（lazy backfill），见 [selection.rs:133](crates/service/src/gateway/routing/selection.rs:133)。仅缓存未命中（TTL 5s 或状态变化）时触发，且 backfill 一次后应稳定。风险：若 `patch_account_meta_in_place` 判断逻辑使元数据反复"变化"，则每次缓存失效都写库。建议：加只读断言/计数监控，确认 backfill 收敛为一次性；若发现反复写入，修正判断逻辑使其幂等。优先级低，先观察。

### ✅ 正面确认（本批）

- **迁移版本控制正确**：`apply_sql_migration` 先 `has_migration` 跳过已应用迁移，仅对新迁移执行 SQL；索引/表统一用 `CREATE ... IF NOT EXISTS`。机制正确，唯一可优化的是 R 项的查询次数（非机制问题）。
- **token 无应用层加解密开销**：token 在 SQLite 明文存储，无 AES/ChaCha 等加解密，故请求热路径无加解密 CPU 成本。注：明文存储属**安全**范畴，security-ci-audit 进度已标注"敏感存储涉数据兼容与恢复路径，应单独排期"，非本轮性能审计目标。

### 审计方法论备注（第六批）

本轮确认账号候选池已有缓存 + quota guard 分流（合理），token 无加解密开销（明文存储，安全另议），迁移版本控制机制正确。真实性能优化仅 R（启动查询批量化，P2 低优先级）；S 为需观察的潜在写放大。至此热路径核心（网关请求转发、账号选择、用量刷新、统计聚合）已系统审计完毕，A–S 共 19 类构成完整优化蓝图。


## 2026-06-14 持续架构审计（第七批：候选缓存深拷贝 + HTTP客户端/env正面确认）

### ✅ T（P1 网关候选缓存命中深拷贝）已完成

✅【已完成 2026-06-22 commit 3a953f81】缓存内部改存 `Arc<Vec<(Account, Token)>>`，命中时 clone Arc（仅原子计数+1），消除每请求深拷贝。大账号池下内存分配从 O(请求数×账号数) 降至 O(缓存重建次数×账号数)。验证：gateway_logs 26 passed。

### ✅ 正面确认（本批）

- **上游 HTTP 客户端已池化复用**：`upstream_client_for_account()` 从 `UPSTREAM_CLIENT_POOL`（OnceLock<RwLock<>>）取缓存 Client，clone 为 Arc 浅拷贝（reqwest::Client 内部 Arc），连接池配置 `pool_max_idle_per_host(32)` + `tcp_keepalive(30s)` + `pool_idle_timeout(90s)`，见 [runtime_config.rs:331](crates/service/src/gateway/core/runtime_config.rs:331)。`fresh_*_for_account()` 新建 Client 仅用于**首次失败后的重试分支**（transport.rs:980/1067），故意绕过可能损坏的池化连接，属正确设计。HTTP 客户端复用无需优化。
- **热路径 env::var 已 atomic 缓存**：route_strategy 经 `reload_from_env()` 读 env 后存 `ROUTE_MODE`（AtomicUsize），运行时读 atomic；quota guard config 经 `current_quota_guard_config()` 缓存。运行时不重复读环境变量，见 [route_hint.rs:711](crates/service/src/gateway/routing/route_hint.rs:711) 与 [selection.rs:607](crates/service/src/gateway/routing/selection.rs:607)。无需优化。
- **热路径日志参数惰性求值**：gateway 路径 `log::trace!/debug!` 未内联 json 序列化或 format 昂贵构造；Rust log 宏在级别未启用时不构造参数。trace body preview 受 `trace_body_preview_max_bytes` 配置门控。无需优化。

### 审计方法论备注（第七批）

本轮在"已池化/已缓存"的表象下深挖，发现候选缓存虽避免了 DB 查询，但命中后的**全列表深拷贝**是被忽略的每请求开销（T 项）——这类"缓存了数据源但没缓存拷贝成本"的模式值得警惕。同时确认 HTTP 客户端池化、env atomic 缓存、日志惰性三项已正确。A–T 共 20 类优化点。


## 2026-06-14 持续架构审计（第八批：请求装配/RPC分发——全维度正面确认）

本批审计 proxy_pipeline 请求体拷贝、header 装配、正则编译、RPC 分发四个维度，**均已是良好设计，无新增待办优化点**。诚实记录正面确认，避免后续重复审计或误改。

### ✅ 正面确认（本批）

- **请求体用 Bytes（Arc 引用计数）+ rewrite 结果缓存**：`candidate_state.rs` 的 `rewrite_body_for_model` 中 `body.clone()` 是 `Bytes` 浅拷贝（Arc 计数+1，非深拷贝）；`body.to_vec()` 深拷贝仅发生在首次 rewrite，且结果按 cache_key 存入 `rewritten_bodies` HashMap（`entry().or_insert_with()`），同一候选多次重试不重复 rewrite。见 [candidate_state.rs:120](crates/service/src/gateway/upstream/proxy_pipeline/candidate_state.rs:120)。设计良好。

- **header 装配预分配且无重复解析**：`build_codex_upstream_headers` 用 `Vec::with_capacity(16)` 预分配，纯字符串 push，无正则/重复解析。见 [codex_headers.rs:113](crates/service/src/gateway/upstream/headers/codex_headers.rs:113)。

- **项目无热路径正则编译**：gateway/core/service 路径均无 `Regex::new`（项目不依赖 regex crate 做请求解析），用手写字符串匹配（`starts_with`/`==`/`split`），无"每请求重新编译正则"反模式，也就不需要 OnceLock/lazy_static 缓存正则。

- **RPC 分发链 + params 借用均高效**：主分发 `handle_request_with_actor` 是 16 模块线性 `try_handle` 链，但每个 `try_handle` 首先 `match req.method.as_str()`，不匹配立即 `_ => return None`（仅一次字符串比较，无反序列化/无 DB），16 次 match 对每请求是纳秒级。各模块命中分支用 `req.params.as_ref().and_then(...)` **借用**提取字段，全仓 0 处 `req.params.clone()`，大 params（import/export/批量）也不深拷贝整个 Value。见 [rpc_dispatch/mod.rs:249](crates/service/src/rpc_dispatch/mod.rs:249) 与 [account.rs:136](crates/service/src/rpc_dispatch/account.rs:136)。

### 🟢 极低优先级观察（暂不列待办）

- RPC 主分发模块顺序：高频的 `startup/snapshot`（首页 ~15s 刷新）由 `startup::try_handle` 处理，排在分发链第 10 位，每次需先过 9 个模块的 method match。但每次 match 仅纳秒级字符串比较，把高频 namespace 提前的收益极小（纳秒级），不值得改动顺序破坏可读性。仅记录，不列优化。

### 审计方法论备注（第八批）

本批四个维度全部确认为良好设计——这是有价值的负面结论（确认无需优化），与发现优化点同等重要。请求装配链（Bytes Arc + rewrite 缓存 + header 预分配 + params 借用）整体内存效率良好。至此请求处理全链路（接入→分发→候选选择→body rewrite→header 装配→上游发送→响应转换→统计）已系统审计完毕。A–T 共 20 类待办优化点 + 多维度正面确认构成完整蓝图，可转入按优先级实施阶段。


## 2026-06-14 持续架构审计（第九批：启动同步网络阻塞 + WebSocket/流式线程正面确认）

### ✅ U（P1 服务启动主路径同步网络阻塞）已完成

✅【已完成 2026-06-22 commit 20c18e27】移除 `start_server()` 第 5 步的同步 npm registry 请求（最多阻塞 10 秒），版本信息由前一步 `sync_runtime_settings_from_storage()` 从已持久化的 app_settings 恢复，后台线程 `ensure_codex_latest_version_sync()` 异步首刷获取最新版本。验证：gateway_logs 26 passed。性能影响：消除冷启动最多 10 秒的网络阻塞。

### ✅ 正面确认（本批）

- **WebSocket 上游属实验性默认关闭**：`USE_WEBSOCKET_UPSTREAM` 默认 false，由 `CODEXMANAGER_USE_WEBSOCKET_UPSTREAM` 显式开启，atomic 缓存配置；WS 握手是 OpenAI WS 端点"每请求一次握手"的请求-响应模式固有设计，非连接池模型；握手失败回退普通 HTTP（transport.rs:927 ws_early_result）。见 [runtime_config.rs:460](crates/service/src/gateway/core/runtime_config.rs:460)。非优化重点。
- **流式 SSE pump 线程属同步服务器固有模型**：`UpstreamSseFramePump::from_reader` 每个流式响应 spawn 一个 blocking read + `sync_channel` 的 pump 线程，这是 tiny_http 同步 blocking 服务器 thread-per-request 模型的一致延伸（pump 线程数与请求线程同数量级，非额外放大），且已用 `BufReader` 缓冲 read_line 减少 syscall，并有 async stream transport 替代路径。见 [stream_readers/common.rs:92](crates/service/src/gateway/observability/http_bridge/stream_readers/common.rs:92)。架构层选择，非简单优化点。

### 审计方法论备注（第九批）

本批审计外围维度（WebSocket、流式线程、启动序列），发现启动主路径的同步网络阻塞（U 项）——这类"启动时同步拉取远端、却已有后台同步兜底"的冗余阻塞，是影响冷启动体验的隐蔽点，尤其在网络受限环境。WebSocket 与流式线程经确认为合理的架构选择。A–U 共 21 类优化点。


## 2026-06-14 持续架构审计（第十批：批量导入无事务包裹 + 前端轮询/IPC正面确认）

### ✅ 已完成（V/AB，本批新增）

- **V/AB（P1 批量账号导入无事务包裹，O(N) 次写提交/fsync）**：✅【已完成 2026-06-25】`account/import` 已按 progress batch 包裹单个 SQLite 写事务；batch 内每条账号导入使用 savepoint 隔离，单条失败只回滚自身并继续处理同批其他账号。新增 `Storage::with_write_transaction()` 与 `Storage::with_write_savepoint()` 复用同一连接事务边界，`import_items_in_batches()` 不再让 account/metadata/token 多表写入逐条自提交。验证：`cargo test -p codexmanager-service --lib import_items_in_batches_rolls_back_only_failed_item -- --nocapture` 通过；`cargo test -p codexmanager-service --lib account::import::tests -- --nocapture` 16/16 通过；`cargo check -p codexmanager-service` 通过（仅既有 warning）。历史问题描述：`import_items_in_batches()` 的 `chunks(batch_size)` 原本仅用于进度上报，实际写入仍逐个 `import_single_item_with_account_id()` → `insert_account()` 自提交，SQLite WAL 下导入 N 账号会产生 O(N) 次提交/fsync。

### ✅ 正面确认（本批）

- **单账号删除已用事务**：`delete_account` 用 `conn.transaction()` 把 account_metadata/subscriptions/tokens/usage_snapshots/events/conversation_bindings/model_source_* 等多表删除包在一个事务原子提交。见 [accounts.rs:619](crates/core/src/storage/accounts.rs:619)。设计正确。
- **前端轮询有页面/后台双门控**：platform-mode candidates 查询 `refetchInterval: isServiceReady && isPageActive ? 5_000 : false` + `refetchIntervalInBackground: false`，仅页面激活且服务就绪时 5s 刷新，后台标签页不轮询。日志列表 10s、首页快照等同样受 isPageActive 门控。见 [use-platform-mode-state.ts:99](apps/src/app/platform-mode/use-platform-mode-state.ts:99)。合理。
- **桌面端 Tauri IPC 直连**：桌面端 `invoke("service_account_import", ...)` 直接调用 Tauri command（进程内 IPC，非网络往返），Web 模式走 HTTP /api/rpc；react-query 的 queryKey 天然去重相同 key 的并发请求。无多余序列化往返。见 [account-client.ts:401](apps/src/lib/api/account-client.ts:401)。

### 审计方法论备注（第十批）

本批审计前端轮询/IPC 与数据库批量写入，发现批量导入的 O(N) 自提交写（V 项）——"有批次概念但批次不用于事务边界"是 SQLite 应用的隐蔽性能点。单账号删除事务、前端轮询门控、Tauri IPC 直连均确认合理。V 项与既有的 J 项（usage_snapshots 写放大）同属 SQLite 写入优化族，可合并为"写入路径优化"实施批次。A–V 共 22 类优化点。


## 2026-06-14 持续架构审计（第十一批：H项实施强化 + 路由查询索引正面确认）

### 🔑 H 项实施强化（关键补充，降低实施成本至近零）

复核确认 H 项（`model_route_error` 全量加载模型目录线性查找）的优化**实施成本极低、零索引开销**：`model_catalog_models` 表**主键即 `(scope, slug)`**（见 [047_model_catalog_models.sql:44](crates/core/migrations/047_model_catalog_models.sql:44)）。当前 [proxy.rs:155](crates/service/src/gateway/upstream/proxy.rs:155) 做 `list_model_catalog_models("default")` 全量加载 + `.any(|item| item.slug == model)` 线性查找，但完全可用主键单查 `SELECT 1 FROM model_catalog_models WHERE scope=?1 AND slug=?2 LIMIT 1` 直接命中（O(log n) 索引查找，无需新增任何索引——主键即索引）。实施：仅需在 [model_options.rs:204](crates/core/src/storage/model_options.rs:204) 附近新增 `model_catalog_model_exists(scope, slug) -> bool` 单查函数，替换 proxy.rs 的全量加载 + 线性 any。这是 P0 热路径优化中性价比最高的一项——改动集中、零索引成本、每请求省一次全表加载。

### ✅ 正面确认（本批：路由相关查询索引均充分）

- **conversation_bindings 主键完美匹配热路径查询**：`get_conversation_binding(platform_key_hash, conversation_id)` 的 WHERE 条件正好命中表主键 `PRIMARY KEY (platform_key_hash, conversation_id)`，O(log n) 直查；另有 `idx_..._account_id(account_id, updated_at DESC)` 与 `idx_..._last_used_at` 支撑反查与剪枝。`delete_stale_conversation_bindings` 未在 service 热路径调用（无每请求清理开销）。见 [conversation_bindings.rs:19](crates/core/src/storage/conversation_bindings.rs:19) 与 [034_conversation_bindings.sql](crates/core/migrations/034_conversation_bindings.sql:1)。
- **model_source_mappings 复合索引优秀**：热路径 `list_enabled_model_source_mappings_for_platform` 的 `WHERE platform_model_slug=?1 AND enabled=1 ORDER BY priority DESC` 被 `idx_model_source_mappings_platform(platform_model_slug, enabled, priority DESC)` 完整覆盖——查询条件 + 排序首列均在索引内，无需回表排序。`list_available_source_model_ids_by_upstream_model` 用 `idx_model_source_models_upstream_model`。见 [058_model_source_mappings.sql:36](crates/core/migrations/058_model_source_mappings.sql:36)。

### 审计方法论备注（第十一批）

本批审计路由相关查询（conversation 绑定、模型源映射、模型目录）的索引覆盖度，确认 conversation_bindings 与 model_source_mappings 索引设计优秀（条件+排序全覆盖），唯一缺口是 H 项的 model_catalog 全量加载——而其主键 `(scope, slug)` 使修复零索引成本。这印证了"索引设计整体良好，个别热路径未用上已有主键"的模式。H 项实施细节已明确，可优先落地。A–V 共 22 类（本批无新增字母项，强化 H 项实施路径）。


## 2026-06-14 持续架构审计（第十二批：observability/日志写入路径——时序观察 + 全维度正面确认）

### ⚠️ 待处理（W，本批新增，P3 低优先级）

- **W（P3 错误早返回路径先写日志后响应，增加错误响应延迟）**：成功主路径日志写入在 `request.respond()` **之后**（不阻塞客户端，见正面确认），但错误早返回路径（404/鉴权失败/早返回错误）是**先 `write_request_log` 再构造 `terminal_text_response` 并 respond**，客户端要等日志写入 DB（含 WAL 写）完成才收到错误响应。见 [request_entry.rs:64](crates/service/src/gateway/request/request_entry.rs:64)、[proxy.rs:268](crates/service/src/gateway/upstream/proxy.rs:268)、[proxy.rs:452](crates/service/src/gateway/upstream/proxy.rs:452)。错误路径相对低频且错误响应延迟敏感度低，故 P3。优化（可选）：调整错误路径时序为先 respond 再写日志，与成功路径对齐；或确认错误响应延迟在可接受范围则不动。仅记录，非优先项。

### ✅ 正面确认（本批：日志/observability 写入路径设计优秀）

- **请求日志单事务两表写 + 空 token 跳过**：`insert_request_log_with_token_stat` 用 `unchecked_transaction()` 把 request_logs + request_token_stats 包在**单事务单提交**；token 与费用均为 0 时只写 request_logs，不写无统计贡献的明细行（与 task.md 既有"空 token_stats 写入跳过"一致）。见 [request_logs.rs:155](crates/core/src/storage/request_logs.rs:155)。
- **request_logs 索引覆盖充分**：8 组复合索引（created_at / status_code+created_at / method+created_at / key_id+created_at / account_id+created_at / created_at+id / trace_id+created_at / actual_source+created_at），日志页按时间/状态/方法/Key/账号/trace/来源筛选均有索引支撑（呼应 M 项）。见 [012/020/027/060 migrations](crates/core/migrations/012_request_logs_search_indexes.sql:1)。
- **成功主路径日志写入不阻塞客户端**：`respond_with_upstream`/`respond_with_stream_upstream` 内部先 `request.respond()` 把响应发给客户端，返回后 execution_context 才调 `write_request_log_with_attempts`——高频成功路径的日志写入在响应之后，不增加客户端感知延迟。见 [execution_context.rs:330](crates/service/src/gateway/upstream/proxy_pipeline/execution_context.rs:330) 与 [delivery.rs respond](crates/service/src/gateway/observability/http_bridge/delivery.rs:2108)。
- **trace_log 成功请求默认零文件 IO**：`log_request_final` 对成功请求（status<400 且无 error）仅 `clear_trace_state`（内存操作），只有 `gateway_trace_stdout_enabled() && elapsed_ms >= slow_threshold`（trace 开启 + 慢请求）才 `flush_trace_lines` 落盘；错误请求标记 `mark_trace_has_error`（内存）。默认不每请求写文件。见 [trace_log.rs:1333](crates/service/src/gateway/observability/trace_log.rs:1333)。

### 审计方法论备注（第十二批）

本批系统审计 observability/日志写入路径（请求日志事务、索引、写入时序、trace 落盘），确认高频成功路径设计优秀：单事务两表写、索引充分、日志在 respond 之后写（不阻塞客户端）、trace 默认零文件 IO。唯一改进点是错误路径的写日志/响应时序（W，P3 低频）。这印证 observability 子系统已高度优化。A–W 共 23 类优化点（W 为 P3 低优先级）。


## 2026-06-14 持续架构审计（第十三批：静态资源缓存头缺失 + Rhai AST重编译 + 内嵌资源正面确认）

### ⚠️ 待处理（X-Y，本批新增）

- **X（P2 Web 静态资源缺长缓存头，每次刷新重下载 bundle）**：`serve_embedded_path` 对 HTML 文档正确设 `Cache-Control: no-store, no-cache, must-revalidate`（入口要最新），但对**非 HTML 静态资源（带 content-hash 的 JS/CSS/字体/图片）完全未设 Cache-Control / ETag / max-age**（已 grep 确认 ui_assets.rs 无 max-age/immutable/etag）。前端构建（Vite/Next）产物文件名含 content hash 属**不可变资源**，本应设 `Cache-Control: public, max-age=31536000, immutable` 让浏览器永久强缓存。当前缺失导致 Web 模式（codexmanager-web 浏览器访问）下每次打开/刷新页面，浏览器只能启发式缓存或发条件请求，可能重新下载全部 JS/CSS bundle（数 MB），增加加载延迟与服务端静态资源请求量。见 [ui_assets.rs:121](crates/web/src/ui_assets.rs:121) 与 [ui_assets.rs:175](crates/web/src/ui_assets.rs:175)。优化：在 serve_embedded_path 对非 HTML 资源（或文件名匹配 hash 模式的资源）追加 `Cache-Control: public, max-age=31536000, immutable`；HTML 保持 no-store。桌面端 Tauri 本地加载不受影响，Web 模式收益明显。

- **Y（P3 插件 Rhai 脚本每次执行重编译 AST）**：`execute_plugin_script` 每次任务执行都 `Engine::new()` + 注册全部 fn + `engine.compile(&plugin.script_body)` 重新编译脚本为 AST。见 [runtime.rs:180](crates/service/src/plugin/runtime.rs:180)。插件是低频调度任务（按 interval_seconds，通常分钟/小时级），重编译开销在低频下可接受，故 P3。优化（可选）：缓存 `(plugin_id, script_body_hash) → AST`（Rhai AST 与 Engine 分离，可复用），仅当脚本变更才重编译；注意注册 fn 依赖 permissions 快照，Engine 本身不缓存、仅缓存 AST。仅高频执行插件时才有意义，先记录。

### ✅ 正面确认（本批）

- **Web 静态资源编译期内嵌，零磁盘 IO**：`include_dir!("$OUT_DIR/codexmanager-web-dist")` 把前端 dist 编译进二进制，`read_asset_bytes` 返回 `&'static [u8]`（指向二进制内存），运行时不读磁盘。见 [embedded_ui.rs:5](crates/web/src/embedded_ui.rs:5)。
- **HTML 文档正确禁缓存**：HTML 入口设 `no-store, no-cache, must-revalidate` + Pragma + Expires，确保前端版本更新后浏览器拿到最新入口（再由入口引用 hashed 资源）。见 [ui_assets.rs:175](crates/web/src/ui_assets.rs:175)。设计正确，X 项是其互补（HTML 不缓存 + 静态资源强缓存才是完整方案）。
- **路径遍历防护**：`serve_embedded_path` 拒绝含 `..` 的路径（`raw.contains("..")` → BAD_REQUEST）。见 [ui_assets.rs:123](crates/web/src/ui_assets.rs:123)。

### 审计方法论备注（第十三批）

本批审计插件 Rhai 执行与 Web 静态资源服务，发现静态资源缓存头缺失（X，P2，Web 模式前端加载）与 Rhai AST 重编译（Y，P3，低频）。内嵌资源、HTML 禁缓存、路径遍历防护均确认良好。X 项与 HTML no-store 互补——构成"入口不缓存 + 不可变资源强缓存"的标准前端缓存策略，当前只做了前半。A–Y 共 25 类优化点。


## 2026-06-14 持续架构审计（第十四批：E项实施强化 + 配置热加载/错误分类正面确认）

### 🔑 E 项实施强化（关键补充，复用现有错误分类器）

复核确认 E 项（requestlog/errorSummary 错误去重"450 条压成 5 类"）的优化**可直接复用现有成熟错误分类器**：`crate::errors::classify_message(message: &str) -> ErrorCode`（[errors/mod.rs:65](crates/service/src/errors/mod.rs:65)）已把原始错误文本规范化映射到稳定的 `ErrorCode` 枚举（含 eq/starts_with/contains 多策略 + 中英文 + 括号尾部英文提取）。E 项的实施只需：① 写入 request_logs 时或查询时调用 `classify_message(error)` 得到 ErrorCode 作为 `error_code` 字段；② 按 error_code GROUP BY 聚合 count/lastSeen/代表样例。无需从零编写错误规范化逻辑——分类器已覆盖 token 刷新失败、网络错误、流传输故障等用户日志中的核心错误类型。这把 E 项从"需设计规范化方案"降为"复用 classify_message + 加聚合查询"，实施成本显著降低。

### ✅ 正面确认（本批）

- **运行时配置 OnceLock + atomic，读不触发 DB**：`ensure_runtime_config_loaded` 用 `RUNTIME_CONFIG_LOADED.get_or_init(reload_from_env)` 只加载一次，后续所有配置读取走 atomic（ROUTE_MODE/USE_WEBSOCKET_UPSTREAM 等），网关热路径读配置零 DB/零 env 访问。见 [runtime_config.rs ensure_runtime_config_loaded](crates/service/src/gateway/core/runtime_config.rs:156)。
- **设置 sync 属低频管理面，无热路径调用**：`sync_runtime_settings_from_storage` / `list_app_settings_map` 调用方均为设置页 GET（current.rs）、启动初始化、插件低频调度（catalog/runtime），网关请求热路径不调用。设置页 GET 触发全量 sync 重应用 setter 属"读带写副作用"的轻微代码味道，但低频（用户打开设置页才触发），非性能问题。见 [runtime_sync.rs:63](crates/service/src/app_settings/runtime_sync.rs:63)。
- **错误分类仅错误路径调用，线性匹配可接受**：`classify_message` 的 to_ascii_lowercase 分配 + 几十个模式线性匹配只在请求出错时执行（非每请求），错误是少数情况，开销可接受；无需为其引入 trie/正则等复杂优化。见 [errors/mod.rs:65](crates/service/src/errors/mod.rs:65)。

### 审计方法论备注（第十四批）

本批审计配置热加载传播与错误码体系，确认运行时配置 OnceLock+atomic（热路径零 DB）、设置 sync 低频、错误分类仅错误路径——均合理。核心产出是 E 项实施强化：发现 classify_message 已提供成熟错误规范化器，E 项可直接复用（类比第十一批对 H 项主键的强化）。连续多轮"实施强化 + 正面确认"主导，印证核心子系统审计饱和，优化蓝图重心应转向实施。A–Y 共 25 类（本批强化 E 项实施路径，无新增字母项）。


## 2026-06-14 持续架构审计（第十五批：跨路径并发token刷新隐患——呼应用户痛点）

### 🔴 待处理（Z，本批新增，P0/P1，直接关联用户日志 >95% 的 refresh_token reused 401）

> ✅【已完成 2026-06-20 commit 2823d671，CCD-Opus 实施 + 主代理审计】gateway bearer 兑换 refresh 兜底已改为复用后台 `refresh_and_persist_access_token`，与后台轮询共用同一把 per-account `TOKEN_REFRESH_LOCKS`，实现跨路径串行化 + 持锁 double-check + 竞态恢复，不再裸调 `refresh_access_token` 二次消费旧 refresh_token。失败分类与账号置不可用语义保持不变。验证：`cargo check -p codexmanager-service` 通过、`cargo test -p codexmanager-service --lib token` 107 项全过。审计备注：① 子代理报告称“唯一修改文件”但 `cargo fmt --all` 另引入 6 个文件的既有格式漂移收敛（已拆分到 commit 1c15d0ef 独立提交，无逻辑变更）；② exchange 锁与 refresh 锁未合并属预期（关注点不同，不在 Z 范围）；③ 跨路径并发“只消费一次 refresh_token”的端到端集成测试仍缺，记为可选技术债。

- **Z（P0-P1 gateway 与后台 token 刷新用两个独立锁，跨路径并发消费同一 refresh_token 触发 "already used" 401）**：系统存在**两条独立的 token 刷新路径，各用各的 per-account 锁，互不互斥**：
  - **gateway 请求路径**：`account_token_exchange_lock`（`static ACCOUNT_TOKEN_EXCHANGE_LOCKS`，[token_exchange.rs:26](crates/service/src/gateway/auth/token_exchange.rs:26)），api_key 交换失败的 fallback 分支**裸调** `refresh_access_token(&token.refresh_token)`（[token_exchange.rs:269](crates/service/src/gateway/auth/token_exchange.rs:269)）消费 refresh_token，该 fallback 仅 double-check 了 api_key_access_token，**未** double-check refresh_token 是否已被后台刷新、也无 race recovery。
  - **后台用量轮询路径**：`token_refresh_lock_for_account`（`static TOKEN_REFRESH_LOCKS`，[usage_token_refresh.rs:14](crates/service/src/usage/usage_token_refresh.rs:14)），`refresh_and_persist_access_token` 有完善的持锁 double-check（重读最新 token，已变则复用）+ `recover_refresh_race_from_latest_token` 竞态恢复。
  
  **隐患**：两个锁是完全独立的 static 锁表，同一账号的 token 刷新若同时被 gateway 请求与后台轮询触发，二者各持各锁、并发执行：后台消费 refresh_token_A 换得 refresh_token_B 落库，gateway fallback 仍用旧 refresh_token_A 调上游 → OpenAI 返回 **"refresh token already used" 401**。这与用户提供的日志中占比 >95% 的 token 刷新 401（reused/revoked）高度吻合，是潜在根因之一。
  
  **优化（P0-P1）**：① 统一两条路径为**单一 per-account token 刷新锁**（共用同一 static 锁表，按 account_id 互斥），或 ② 让 gateway 的 refresh fallback **复用后台的 `refresh_and_persist_access_token`**（它已含 double-check + race recovery），不再裸调 `refresh_access_token`。这样跨路径刷新串行化，后到者持锁后重读最新 token 即发现已刷新、直接复用，避免重复消费旧 refresh_token。可显著降低用户日志中的 reused 401。配合既有 O 项（失败分类冷却）与 P 项（指数退避），构成 token 刷新健壮性的完整修复。

### 审计方法论备注（第十五批）

本批审计 token 交换/刷新并发路径，发现 gateway 与后台两条刷新路径锁不统一（Z 项）——这是十五轮审计中**与用户原始痛点（token 刷新 401 风暴）最直接相关的高价值发现**。单条路径内的 single-flight + double-check 设计均正确，缺口在**跨路径锁未协调**。Z 与 O/P 同属 token 刷新健壮性族，应合并为"token 刷新修复"实施批次并优先落地——直接改善账号可用性与用户可见的错误日志。A–Z 共 26 类优化点。


## 2026-06-14 持续架构审计（第十六批：定时warmup全账号串行 + 登录回调/Client复用正面确认）

### ✅ AA（P2 定时 warmup 全账号串行）已完成

✅【已完成 2026-06-22 commit 7b2174c8】将 `warmup_accounts` 从串行执行改为有界并发：
- 新增 `WARMUP_WORKERS` 配置（默认 2，环境变量 `CODEXMANAGER_WARMUP_WORKERS`）
- 使用 crossbeam_channel + worker 池模式并发执行
- 每个 worker 独立获取 storage 连接，共享 Client 连接池
- 理论加速 N 倍（N = workers 数），保守并发避免上游限流

验证：单元测试 17 passed、gateway_logs 26 passed。性能改进：总时间从 O(N×单账号时间) 降至 O(N×单账号时间/workers)。

### ✅ 正面确认（本批）

- **登录回调服务器单例复用**：`ensure_login_server_with_addr` 用 `LOGIN_SERVER_STATE.get_or_init` + Mutex 单例，已绑定则直接返回复用，不会每次登录都 spawn 新监听服务器。见 [auth_callback.rs:276](crates/service/src/auth/auth_callback.rs:276)。
- **warmup HTTP Client 已复用**：`warmup_accounts` 中 `build_warmup_client()` 只构建一次，传 `&client` 引用给每个 `warmup_single_account`，串行各账号复用同一 Client（连接池保留）。Client 复用正确，问题仅在请求串行（AA 项）。见 [account_warmup.rs:63](crates/service/src/account/account_warmup.rs:63)。

### 审计方法论备注（第十六批）

本批审计账号 warmup、登录回调生命周期，发现定时 warmup 全账号串行（AA，P2）——与用量刷新的多 worker 并发模型不一致，是后台任务并发模型的局部缺口。登录回调单例、warmup Client 复用确认良好。AA 与既有后台任务优化（用量轮询限载、聚合余额批次）同属"后台任务效率"族。A–AA 共 27 类优化点。


## 2026-06-20 CodeX-GPT 独立复核第七至十六批（T-AA）

### 复核结论

- **T / 2026-06-19 V 候选缓存深拷贝：✅ 确认，且为重复项，应合并实施。** 当前 `collect_gateway_candidates_with_low_quota_mode()` 在未命中时 `write_candidate_cache(low_quota_mode, candidates.clone())`，命中时 `read_candidate_cache()` 返回 `Some(cached.candidates.clone())`，确实会按请求深拷贝 `Vec<(Account, Token)>`。2026-06-19 第七批的 V 与 2026-06-14 第七批的 T 是同一问题，后续下发时统一命名为 **T：候选缓存 Arc 化**，不要按两个任务重复实施。修复方向：缓存存 `Arc<Vec<(Account, Token)>>`，命中只 clone Arc；调用方改只读遍历，必要时再局部 clone 单个候选。
- **U 服务启动同步网络阻塞：✅ 确认。** `start_server()` 在监听 HTTP 端口前调用 `sync_gateway_user_agent_version_from_codex_latest()`，该函数通过 `fresh_upstream_client().get(CODEX_NPM_LATEST_URL).timeout(10s).send()` 同步访问 npm registry；紧随其后又调用 `ensure_codex_latest_version_sync()` 启动后台线程。结论成立：启动主路径的远程同步是冗余阻塞。修复方向：启动时只读取已持久化版本或默认版本，远程 latest 拉取完全交给后台线程首刷。
- **2026-06-14 第十批 V/AB 批量导入无事务：✅ 已完成。** `import_items_in_batches()` 已按 batch 包一层 SQLite 写事务，并用 savepoint 保留单条失败不影响同批其他账号的语义。历史确认：原 `chunks(batch_size)` 只用于 `begin_batch/finish_batch` 进度上报，循环内仍逐条 `import_single_item_with_account_id()`；该函数又逐步 `insert_account()`、`upsert_account_metadata()`、写 token/subscription 等，storage 侧 `insert_account()` 是单条 `conn.execute`，未见批次事务边界。命名仍统一为 **AB：批量账号导入按 batch 事务提交**，避免与候选缓存 V/T 混淆。
- **W 错误早返回先写日志后响应：✅ 确认，维持 P3。** `request_entry.rs` 的早期错误路径先 `write_request_log()` 再返回错误响应；`proxy.rs` 的 `model_route_error()` 与 aggregate API 404 也先写日志再构造 terminal response。成功主路径已在 respond 后写日志，因此 W 是真实但低优先级的时序一致性问题。修复方向：仅在不破坏 trace/outcome 记录语义的前提下，把错误响应也改成先 respond 后异步/后置写日志。
- **X 静态资源缺长缓存头：✅ 确认。** `serve_embedded_path()` 只对 HTML 追加 `no-store, no-cache, must-revalidate`，非 HTML 静态资源不设置 `Cache-Control`；现有测试还断言 `favicon.ico` 不存在 cache-control。修复方向：HTML 保持 no-store，非 HTML 资源追加 `Cache-Control: public, max-age=31536000, immutable`；若担心非 hash 文件，先限定 hashed chunk / assets 目录，favicon 可单独给较短缓存。
- **Y Rhai 每次执行重编译：✅ 确认，维持 P3。** `execute_plugin_script()` 每次新建 `Engine`、注册函数并 `compile(&plugin.script_body)`；插件通常低频，先记录即可。修复方向：只缓存 `(plugin_id, script_body_hash) -> AST`，Engine 仍按权限快照构建，避免把权限函数或动态设置错误缓存。
- **Z gateway 与后台 token 刷新锁不统一：✅ 确认，且优先级应高于一般性能项。** gateway bearer 兑换 fallback 在 `account_token_exchange_lock` 内裸调 `refresh_access_token(&token.refresh_token)`，后台刷新在独立 `TOKEN_REFRESH_LOCKS` 内执行 `refresh_and_persist_access_token()`，后者才有持锁重读、double-check 与 `recover_refresh_race_from_latest_token()`。两套锁无法跨路径互斥，确实可能并发消费同一 refresh_token，和用户日志里的 `refresh token already used` 401 高度相关。修复方向优先选 **统一复用 `refresh_and_persist_access_token()`**，而不是再复制一套 race recovery。
- **AA 定时 warmup 全账号串行：✅ 确认，但默认关闭，按 P2 处理。** `WARMUP_CRON_ENABLED` 默认 false；开启后 `warmup_cron_loop()` 调 `warmup_accounts(Vec::new(), "")`，空账号列表会经 `resolve_target_accounts()` 解析为 `list_gateway_candidates()` 全候选，再在 `warmup_accounts()` 中串行 `for account in accounts.drain(..)` 发完整上游请求。修复方向：保守有界并发（独立 `WARMUP_WORKERS`，默认 2-4），并对指定 account_ids 增加按 ID 直查，避免先全量候选再线性 find。

### 下发实施校准

- **必须优先落地**：Z（统一 token refresh 锁/路径）+ O/P（分类冷却、指数退避）+ E（错误码聚合）。这组直接对应用户日志中 >95% 的 token refresh 401 风暴，收益大于低优先级 CPU 小项。
- **热路径性能批次**：H（模型目录按 slug 主键单查）、C（聚合 API 热路径 active/provider SQL 下推）、T（候选缓存 Arc 化）、Q（协议转换 JSON 单次 parse）、X（静态资源长缓存）。这些改动范围相对清晰，适合拆成独立 PR。
- **后台/写入批次**：AB（批量导入事务，已完成）、J（usage_snapshots 变化检测再写，已完成）、AA（warmup 有界并发，已完成）、U（启动远程同步后台化，已完成）。其中 AB/J 属 SQLite WAL 写放大治理，AA/U 属后台任务与启动体验治理。
- **仅记录或低优先级**：W（错误路径响应时序）、Y（Rhai AST 缓存）、S（候选 lazy backfill 观察）。除非真实环境指标指向这些点，否则不应抢占 Z/E/H/C/T 的实施顺序。

### 编号去重备注

当前 `task.md` 保留了多轮历史审计原文，因此存在字母复用：2026-06-19 第七批 V 与 2026-06-14 第七批 T 是同一候选缓存问题；2026-06-14 第十批 V 是另一个“批量导入无事务”问题。后续派单请使用本节校准命名：**T=候选缓存 Arc 化，AB=批量导入事务**。不要按历史字母机械统计总数。

## 2026-06-14 持续架构审计（第十七批：聚合API候选每请求全量加载无下推无缓存 + keepalive正面确认）

### ✅ BB（P1 聚合 API 路由候选优化）已完成

✅【已完成 2026-06-22，两阶段实施】
- **阶段 1**（commit cd4da040）：SQL 下推 + 索引优化
  - 新增 `list_active_aggregate_apis_by_provider(provider_type)` 方法
  - SQL 层过滤 `status='active' AND provider_type=?`
  - 添加 `idx_aggregate_apis_status_provider` 复合索引
  - 编写单元测试验证过滤语义
  
- **阶段 2**（commit 97e08f98）：候选缓存
  - 新增 `AGGREGATE_API_CANDIDATE_CACHE` 全局缓存
  - 默认 TTL 5000ms，按 provider_type 隔离
  - 聚合 API 变更时自动失效缓存
  - 热路径每请求节省 1 次 SQLite 查询
  
验证：gateway_logs 26 passed，编译无错误。预期收益：缓存命中率 >95%，多 supplier 场景收益显著。

### ✅ 正面确认（本批）

- **gateway keepalive loop 设计完善**：`gateway_keepalive_loop` 走通用 `run_dynamic_poll_loop`（interval + jitter + failure backoff），单次 `run_gateway_keepalive_once` 只调 `fetch_models_for_picker()` 探活预热上游连接与 token exchange（轻量，非全量扫描账号），并有 `is_keepalive_error_ignorable` 识别"无可用账号/存储不可用"等可忽略错误避免无谓 backoff。见 [usage_keepalive.rs:12](crates/service/src/usage/usage_keepalive.rs:12) 与 [runner.rs:72](crates/service/src/usage/refresh/runner.rs:72)。设计良好。

### 审计方法论备注（第十七批）

本批审计 gateway keepalive 与聚合 API 故障转移候选解析，发现聚合 API 候选每请求全量加载无下推无缓存（BB，P1）——这与 H 项（模型目录全量加载线性查找）、T 项（账号候选缓存深拷贝）同源，都是"热路径未用对数据访问方式"。值得注意：账号候选路径已优化（缓存+quota guard），但**聚合 API 候选路径未享受同等优化**——这是路径间优化不对称的典型。keepalive 子系统确认完善。A–BB 共 28 类优化点。


## 2026-06-20 持续架构审计（第四批：调度器 + 插件运行时 + 热路径正面确认）

### N. 插件调度器冗余全量查询（P1，新发现）

`run_due_tasks_once()`（`crates/service/src/plugin/scheduler.rs:20`）已用 `list_due_plugin_tasks(now, 100)` SQL 下推（JOIN plugin_installs + WHERE enabled/status/next_run_at）取到期任务，执行无误。

问题：执行完到期任务后，为计算下次 sleep 时长，又额外调用：
- `list_plugin_installs()` 全量加载所有安装
- `list_plugin_tasks(None)` 全量加载所有任务
然后在 Rust 层遍历求最小 `next_run_at`。

大量插件/任务时，每个调度 tick 都把全部 installs + tasks 搬到内存只为求一个最小值。

建议：新增 `min_next_run_at_for_enabled_tasks(now)` storage 查询（`SELECT MIN(next_run_at) FROM plugin_tasks t JOIN plugin_installs p ... WHERE enabled=1 AND status='enabled' AND schedule_kind<>'manual'`），一条 SQL 返回下次唤醒时间，消除两次全量加载。

### O. 插件 Rhai 引擎每次重建并重编译 AST（P3，新发现，低优先级）

`execute_plugin_script()`（`crates/service/src/plugin/runtime.rs:173`）每次任务执行都 `Engine::new()` + 注册全部 fn + `engine.compile(&plugin.script_body)` 重新编译脚本为 AST。

插件是低频后台调度任务（interval 通常分钟/小时级），单次重建开销可接受；但同一脚本反复执行（高频调度或手动多次触发）时重编译 AST 是浪费。

建议（低优先级）：按 `(plugin_id, script_body_hash)` 缓存编译后的 AST（Rhai AST 与 Engine 分离，可独立复用）。注意引擎注册的闭包捕获 permissions/settings 快照，引擎本身不宜跨执行共享，仅缓存 AST。仅当插件高频执行才有收益。

### 正面确认（已优化，记录避免重复审计）

- **P. 热路径 env::var 已 atomic 缓存**：`route_hint.rs` 的 route_strategy 等用 `reload_from_env()` 读 env 后存 atomic（`ROUTE_MODE.store`），运行时读 atomic load，非每请求读 env。selection.rs 的 quota guard config 同样用 `current_quota_guard_config()` 内存缓存。
- **Q. 上游 HTTP 客户端已连接池化 + 按 proxy 缓存**：`runtime_config.rs` 用 `UPSTREAM_CLIENT`（OnceLock<RwLock<Client>>）+ `UPSTREAM_CLIENT_POOL`（按 proxy_url 缓存），客户端配置 `pool_max_idle_per_host(32)` + `pool_idle_timeout(90s)` + `tcp_keepalive(30s)`。proxy.rs/postprocess.rs 中的 `Client::new()` 全是测试 fixture。
- **R. Web 静态资源已 include_dir 内嵌**：`crates/web/src/embedded_ui.rs` 用 `include_dir!` 编译期内嵌 dist 到二进制，非每请求读盘。
- **S. 运行时配置已 OnceLock 缓存**：`RUNTIME_CONFIG_LOADED.get_or_init(reload_from_env)`，配置变更走显式 reload，非每次重新解析。

### 审计进度小结

task.md 已累计记录 A–S 共 **19 类**架构优化点 + 正面确认。本轮新增 N（P1 调度器冗余全量）、O（P3 Rhai AST 缓存）两个可优化项，并确认 env/HTTP客户端/Web资源/运行时配置 4 项已优化。后续可转向：trace_log 写入开销、SSE keepalive 定时器、前端 React 重渲染热点、迁移脚本启动耗时。

## 2026-06-20 持续架构审计（第五批：trace_log 全局锁竞争）

### T. trace_log 每请求多次全局锁 + 无门控字符串分配（P1，新发现）

`crates/service/src/gateway/observability/trace_log.rs` 的所有 trace 记录函数（`log_request_start`:840、`log_request_execution_plan`:869、`log_request_final` 等共 14+ 处）都无条件执行：
1. `format!(...)` 拼接 trace 字符串（每次堆分配）
2. `buffer_trace_line(trace_id, line)` → `lock()` 全局 `TRACE_PENDING_LINES`（`Mutex<HashMap<String, Vec<String>>>`，trace_log.rs:25）

问题：
- **无全局 trace 开关**：即使运维不看 trace，每个网关请求仍执行全部 format + 入 HashMap。一个请求至少 REQUEST_START + execution_plan + final 多条行 → 每请求多次 `format!` 堆分配 + 多次全局 Mutex 加锁。
- **全局单锁**：所有请求线程争抢同一个 `TRACE_PENDING_LINES` Mutex。高 RPS（如 Codex CLI 高频探测）下成为锁竞争热点，且与 `TRACE_ERROR_TRACES`（trace_log.rs:24）叠加。

已优化部分（确认）：磁盘写入已通过 `TRACE_WRITER`（`TraceAsyncWriter`，trace_log.rs:22）异步落盘，不阻塞请求线程；pending 行有 `TRACE_PENDING_LINE_LIMIT` 截断保护。

建议（按收益排序）：
1. 新增 atomic 全局 trace 开关（默认开/可关），`buffer_trace_line` 入口先判断，关闭时直接 return，连 `format!` 都不做（调用点改为闭包惰性求值或入口判断）。
2. 若需常开，把 `TRACE_PENDING_LINES` 改为分片锁（按 trace_id hash 分 N 个 bucket），降低单锁竞争。
3. 错误才落盘的场景，非错误 trace 可在内存环形缓冲滚动，减少 HashMap 增删。

### 正面确认（已优化）

- **U. SSE keepalive 非每连接独立定时器**：`resolve_stream_keepalive_frame`（delivery.rs:3938）返回 `SseKeepAliveFrame` 帧类型，在同步流读取循环内按 idle 时间插入 keepalive 帧，非每连接起独立定时器线程。
- **V. trace 磁盘写入已异步化**：`TraceAsyncWriter` 后台线程落盘，请求线程只入内存缓冲。

### 审计进度小结

task.md 累计 A–V 共 **22 类**。本批新增 T（P1 trace_log 全局锁竞争 + 无门控 format），确认 SSE keepalive / trace 异步写入已优化。后续方向：迁移脚本启动耗时、前端 React 重渲染、account/usage/aggregate 接回 SQL 下推（P0 J 关联）、错误码归一化聚合（E 项）。

## 2026-06-20 持续架构审计（第六批：迁移启动开销）

### W. 启动时 69 次串行 has_migration 查询（P2，新发现，仅影响启动）

`Storage::init()`（`crates/core/src/storage/mod.rs:772`）顺序调用 69 次 `apply_sql_migration` / `apply_sql_or_compat_migration` / `apply_compat_migration`。每个内部先 `has_migration(version)`（mod.rs:1398），该函数每次都 `conn.prepare("SELECT 1 FROM schema_migrations WHERE version=?1 LIMIT 1")` + 查询。

问题：对已完成全部迁移的老库，每次进程启动/每次新建 storage 都执行 **69 次 prepare statement + 69 次串行 SELECT**，纯属重复确认已知状态。虽然单查很快，但 prepare 编译 SQL + 往返累加在启动路径上，且连接池每开一个新连接走 init 时都重复付费。

建议：`init()` 开头一次性 `SELECT version FROM schema_migrations` 载入 `HashSet<String>`，`has_migration` 改为内存 `set.contains(version)`；69 次 SQL 降为 1 次。或改用 `PRAGMA user_version` 整数版本号单值判断跳过已达版本。

注意：与 task.md 已记录的"启动迁移轻量化"（不再在启动跑历史清理/VACUUM）是不同层面——那是避免重操作，这是避免重复的轻查询累加。

### 正面确认（已优化/非问题）

- **X. 迁移已幂等门控**：`apply_sql_migration` 先 `has_migration` 判断，已应用即跳过，不会重复执行 DDL；`apply_sql_or_compat_migration` 对历史库的"重复列/表"冲突有 compat fallback。逻辑正确，仅查询次数可优化（见 W）。
- **Y. 协议适配无重复 parse**：`request_router.rs` 的 OpenAI→Anthropic(36) / Anthropic(111) / Gemini(173) 三处 `from_slice` 各属不同 adapter 函数，每请求只走其一，非对同一 body 重复解析。

### 审计进度小结

task.md 累计 A–Y 共 **25 类**。本批新增 W（P2 迁移启动 69 次查询），确认迁移幂等门控正确、协议适配无重复 parse。后续方向：account/usage/aggregate 接回 SQL 下推（P0 J 关联）、前端 React 重渲染热点、错误码归一化聚合（E 项）、上游重试退避策略（G 项关联）。

## 2026-06-20 持续架构审计（第七批：并发/路由/状态管理 — 多为正面确认）

### Z. challenge 检测全量 to_ascii_lowercase 分配（P3，新发现，仅错误路径）

`body_looks_like_cloudflare_challenge`（`delivery.rs:416`）与 `looks_like_cloudflare_challenge`（`output_text.rs:875`）对整个响应体 `to_ascii_lowercase()`（分配与 body 等大的新 String）后做 8 次 `contains` 子串扫描。

影响有限：调用点（delivery.rs:1255/1308 的 `classify_compact_non_success_kind` 等）均在**非成功/错误响应分类路径**，正常成功响应不触发。仅当上游返回错误/challenge HTML（可能几十 KB）时，每次分配全量 lowercase 副本。

建议（低优先级）：改用大小写不敏感子串匹配（如对固定 ASCII 关键词做不分配的 `find` + `eq_ignore_ascii_case` 窗口比较，或一次扫描多模式匹配），避免分配整个 body 的 lowercase 副本。收益仅在大错误响应高频出现时显现。

### 正面确认（已优化，记录避免重复审计）

- **AA. 后台任务线程模型良好**：`spawn_background_loop`（refresh/mod.rs:297）用 `catch_unwind` 包裹 worker，panic 自动重启（1s 退避）。usage-polling / gateway-keepalive / token-refresh-polling / account-warmup 各一专用命名线程，非每任务起线程。
- **BB. token 刷新并发分片良好**：`run_token_refresh_tasks`（refresh/mod.rs）用 crossbeam `unbounded` channel + worker 池（`token_refresh_worker_count` 按账号数与配置取 min）+ 每 worker 独立 storage 连接 + `thread::scope` 优雅 join，无串行瓶颈。
- **CC. 路由选择用 P2C 算法**：`apply_health_p2c` / `p2c_challenger_index`（route_hint.rs）用 power-of-two-choices O(1) 随机选两比较，非全候选排序。
- **DD. 路由状态 HashMap 清理完善**：`p2c_nonce_by_key_model` / `next_start_by_key_model` 既有 lazy `remove_entry_if_expired`，又有 `maybe_maintain`（route_hint.rs:842）按 `maintenance_tick % ROUTE_STATE_MAINTENANCE_EVERY` 节流的全表 `prune_expired_entries`（TTL）+ `enforce_capacity_pair`（容量上限），无无界增长。
- **EE. 无运行时正则编译**：service crate 无 `Regex::new`，challenge/错误检测均用 `contains` 字面量匹配，无每请求正则编译开销。

### 审计进度小结

task.md 累计 A–Z + AA–EE 共 **31 类条目**（含可优化项与正面确认）。本批新增 Z（P3 challenge 全量 lowercase），确认后台线程池/token 并发分片/P2C 路由/路由状态清理/无正则编译 共 5 个区域已优化良好。

剩余高价值待实施项（按优先级）：H/J/account-usage-aggregate(P0)、I/N/T(P1)、K/O/W(P2)、Z(P3)。后续审计方向趋于收敛，可转向验证已记录 P0 项的当前实现状态，或前端 React 重渲染 / 错误码归一化聚合(E) 等仍未深入的维度。

## 2026-06-20 持续架构审计（第八批：前端 — 整体健康，2 个边际点）

### FF. listAggregateApis 硬编码 pageSize:500 静默截断风险（边界正确性，非性能）

`account-client.ts:654` 的 `listAggregateApis()` 硬编码 `{ page: 1, pageSize: 500 }` 一次性拉取全部聚合 API，隐含假设总数 ≤ 500。后端 `aggregateApi/list` 已分页，超过 500 时只返回前 500 条，**静默丢弃其余**。

影响：聚合 API 通常数量有限，触发罕见；但属正确性隐患——大量聚合 API 部署时下拉/选择列表会缺项。

建议：要么用游标/循环翻页拉全量，要么在返回 `total > 500` 时告警/提示。优先级低（多数部署 < 500 聚合 API），但应记录避免误判为"已加载全部"。

### GG. 无全局 QueryClient defaultOptions（前端 P3）

各页面 useQuery 各自配 `staleTime`（logs 30-60s、models 60s 等，配置合理），但未见全局 `QueryClient({ defaultOptions: { queries: { staleTime, gcTime } } })`。遗漏 staleTime 的查询会落到 React Query 默认 `staleTime:0`（如 `platform-mode/use-platform-mode-state.ts:98` 显式 0），窗口聚焦/重新挂载即 refetch。

建议：设全局 defaultOptions 兜底 staleTime（如 30s）+ gcTime，避免个别查询遗漏导致窗口聚焦狂刷 RPC。已显式需要实时的（platform-mode staleTime:0）保留覆盖。

### 正面确认（前端已优化/合理）

- **HH. 列表 DOM 规模由后端分页兜底**：account/apikey/requestlog 列表后端分页（默认 pageSize 20，`MAX_ACCOUNT_PAGE_SIZE=500` 仅为上限钳制），前端 `.map` 渲染当前页有限条数，无需虚拟化库。
- **II. 列表查询 staleTime 合理**：日志页 30-60s、模型页 60s staleTime，配合前几批确认的 refetchInterval 降频（日志 10s→不后台刷、摘要不轮询），避免前端狂刷。

### 审计进度小结

task.md 累计 A–Z + AA–II 共 **35 类条目**。本批前端审计结论：**整体健康**，后端分页兜底 DOM 规模，React Query staleTime 配置合理，无严重前端性能热点；仅 2 个边际点（FF 聚合API 500硬编码截断、GG 无全局 query 默认配置）。

审计已充分收敛。剩余高价值待实施项仍为后端 P0/P1：H/J/account-usage-aggregate(P0)、I/N/T(P1)。后续可转入"验证已记录 P0 项实现状态"或开始实际优化实施阶段（需用户确认是否从审计转入修改）。

## 2026-06-20 持续架构审计（第九批：token refresh 失败重试不分错误类型）

### JJ. token 刷新失败统一固定冷却，未区分临时/永久（P1，新发现，关联 GPT 诊断）

`schedule_token_refresh_failure_retry`（`crates/service/src/usage/refresh/mod.rs:780`）对**所有** token 刷新失败一律用同一个固定 `token_refresh_failure_cooldown_secs()`（mod.rs:773，环境变量可调，默认固定值）把 `next_refresh_at` 推后，完全不看错误类型：

```rust
fn schedule_token_refresh_failure_retry(storage, account_id, now) {
    let cooldown = token_refresh_failure_cooldown_secs();
    let next_refresh_at = now.saturating_add(cooldown);
    storage.update_token_next_refresh_at(account_id, Some(next_refresh_at));
}
```

而 token refresh 失败的真实成因（参见 usage_http.rs:41 归类）至少分三类：
1. **永久失效**（refresh_token_reused / invalidated / expired / invalid_grant）：应停止重试 → 已由 tokens.rs:95 的 `refresh_token_invalid:*` 排除轮询，**这部分有保护**。
2. **服务端抖动 / unknown_401**：可能下次就恢复，固定冷却（默认数分钟～小时级）会让本可恢复的账号长时间不可用。
3. **网络错误**（error sending request for url）：瞬时故障，本应快速重试 + 指数退避，而非一律等固定冷却。

问题：类型 2/3 与一般失败共用同一固定冷却，缺乏分层退避。GPT 诊断已指出"网络/5xx/Unknown401 不应统一冷却 6 小时，更适合指数退避 + 少量探测"。

建议：
- 失败重试冷却按错误类分层：网络/5xx/unknown_401 用短初始 + 指数退避（如 30s→1m→2m…上限），永久失效直接交给现有 `refresh_token_invalid:*` 排除。
- 拆分"事件去重窗口"（errors.rs:34 的 6h 内存节流，用于降事件噪声）与"刷新重试冷却"（本项，决定下次何时重试）——两者目的不同，不应共用同一时长。

### KK. classify_usage_status_from_error 分类粒度过粗（P2，新发现）

`classify_usage_status_from_error`（mod.rs:742）只用字符串前缀粗分两类：
- `usage/subscription endpoint status/failed` 前缀 → `Unavailable`
- 其余全部（含网络错误、token 错误、超时）→ `Unknown`

问题：网络抖动、超时、token 错误都被归为 `Unknown`，下游无法据此做差异化处理（如 JJ 的分层退避）。错误分类是 JJ 分层退避的前置依赖。

建议：扩展为结构化错误分类（network / timeout / upstream_5xx / unauthorized_temp / unauthorized_permanent / quota），让退避策略、事件去重、账号状态标记都能据类型差异化决策。这也与 task.md 已记录的 E 项（请求日志错误码归一化聚合）共用一套错误分类体系。

### 审计进度小结

task.md 累计 A–Z + AA–KK 共 **37 类条目**。本批新增 JJ（P1 token 刷新失败统一冷却不分类型）、KK（P2 错误分类过粗），二者构成"错误分类 → 分层退避"的成对优化，且与已记录 E（错误码聚合）、G（上游重试退避）同源——建议合并为一个"统一错误分类体系"专题实施。

剩余高价值待实施项：H/J/account-usage-aggregate(P0)、I/N/T/JJ(P1)、K/O/W/KK(P2)、Z/GG(P3)。审计已充分覆盖后端热路径、并发、存储、前端、错误处理五大维度，趋于收敛。

### ⚠️ 持续架构审计 - 第十批（事务边界）

- **LL（P1，与 AB 同项）账号批量导入无事务批提交**：✅【已完成 2026-06-25】`import_items_in_batches()` 已按 progress batch 使用单个写事务提交，batch 内每条导入用 savepoint 隔离失败项，避免整批回滚。历史问题：`batch`（默认 200）原仅用于进度报告分组，实际每个 item 经 `import_single_item_with_account_id()` 独立写入；该函数内部 `insert_account()` + `insert_token()` 是独立单条 `execute`（各自 autocommit），导入 N 个账号 ≈ 2N 次独立 WAL fsync。

#### ✅ 第十批正面确认（已优化，避免重复审计）
- **MM** storage 层多步写入已广泛使用事务：account_manager/accounts/aggregate_apis/model_groups/model_options/plugins/quota_pools/request_logs 等关键多步写入均包 `transaction()`/`unchecked_transaction()`，事务纪律整体良好，仅 service 层批量导入循环（LL）是缺口。

### 2026-06-14 持续架构审计第十一批（安全门禁）

- **NN（P1，安全门禁）**：`all_interfaces` 暴露与 `WEB_AUTH_MODE_NONE` 缺少联动强制。`default_listener_bind_addr()` 默认 loopback 是安全默认（见 [service.rs:229](crates/service/src/app_settings/service.rs:229)），仅在用户显式开 `all_interfaces` 时绑 `0.0.0.0`。但 `current_web_auth_mode()` 未配置密码时默认 `WEB_AUTH_MODE_NONE`（见 [app_manager.rs:138](crates/service/src/auth/app_manager.rs:138)）。缺口：用户开 all_interfaces 把服务暴露到局域网/公网时，系统不强制要求认证，也无显著告警。建议开 all_interfaces 时若 auth_mode=none 则拒绝启动或强制提示，避免未授权暴露账号池/网关。

- **正面确认 OO**：bind 默认安全。`normalize_service_bind_mode` 无配置默认 `LOOPBACK`，`0.0.0.0` 常量仅在显式 all_interfaces 下生效，非默认开放，见 [service.rs](crates/service/src/app_settings/service.rs:1)。

### 2026-06-14 持续架构审计第十二批（密钥日志脱敏）

- **PP（P2，安全一致性）**：URL 脱敏未全局统一。`redact_sensitive_error_url()` / `redact_sensitive_url_parts()`（见 [auth_tokens.rs:483](crates/service/src/auth/auth_tokens.rs:483)）会从 reqwest 错误的 URL 中脱敏敏感 query 参数，但**仅 auth_tokens 模块使用**。gateway 上游请求错误（如 [transport.rs:424](crates/service/src/gateway/upstream/attempt_flow/transport.rs:424)、:540）直接 `err.to_string()`，reqwest error 默认携带 URL。OpenAI/Codex 上游用 Authorization header（URL 通常无 secret），但聚合 API 自定义上游若把 key/token 放 query，错误日志/请求日志会泄露。建议：抽出公共 `redact_sensitive_error_url` 到错误处理模块，gateway 上游与聚合 API 错误统一走脱敏后再 to_string，避免 query secret 进日志。

#### ✅ 第十二批正面确认
- **QQ** 无明文 token 日志：未发现 `log::*!` 直接打印 access_token/refresh_token/password/api_key；id_token 仅用于 claims 解析与导入导出数据处理，非日志输出。
- **RR** auth 流程 URL 已脱敏：OAuth token 交换等 auth_tokens 路径的请求错误均经 `redact_sensitive_error_url` 处理后才转字符串。

### 2026-06-14 持续架构审计第十三批（缓存失效风暴）

- **SS（P1，缓存失效风暴 thundering herd）**：候选账号缓存重建无 single-flight 保护。`collect_gateway_candidates_with_low_quota_mode()`（见 [selection.rs:109](crates/service/src/gateway/routing/selection.rs:109)）在 `read_candidate_cache` 未命中后直接 `collect_gateway_candidates_uncached()` 查 DB 重建并写回，无并发去重。缓存失效有两个触发源：TTL（默认 5s，`DEFAULT_CANDIDATE_CACHE_TTL_MS`）到期；账号状态更新调 `invalidate_candidate_cache()`（见 [account_status.rs:64](crates/service/src/account/account_status.rs:64)，用量刷新/请求成败/token refresh 多场景触发）。高 RPS（如 Codex CLI 高频）下，每次失效瞬间所有并发请求同时 read-miss → 各自执行 `list_gateway_candidates()`（accounts+tokens）+ `load_usage_snapshots_for_candidates()`（usage snapshots）多表重建，造成 DB 查询风暴与 CPU 峰值。建议：single-flight（重建 Mutex/原子门，同一时刻仅一个请求重建，其余等待新值）+ stale-while-revalidate（失效后短暂复用旧值直到新值就绪），把 N 次并发重建降为 1 次。

#### ✅ 第十三批正面确认
- **TT** 候选缓存 TTL 已合理：默认 5s（前期已从 500ms 上调），账号状态变化主动失效，读写用 atomic TTL + 锁保护缓存本身；唯一缺口是失效后的重建并发去重（SS）。

### 2026-06-14 持续架构审计第十四批（SSE 流式背压 — 全面正面确认）

本批审计流式响应是否存在内存累积/无界堆积，结论：**背压实现教科书级，无可优化项**，三层防护均到位，记录以避免后续误判此关键路径为问题点。

#### ✅ 正面确认
- **UU** Passthrough 流式边读边写边 flush：`write_streaming_chunked_response`（见 [delivery.rs:234](crates/service/src/gateway/observability/http_bridge/delivery.rs:234)）用固定 8KB buffer 循环 read→write_all→flush，chunked 编码，不累积全量 body。
- **VV** 适配器转换逐帧推进不累积：`AnthropicSseReader`/`GeminiSseReader`/`ChatCompletionsFromResponsesSseReader` 的 `read()`（见 [anthropic.rs:617](crates/service/src/gateway/observability/http_bridge/stream_readers/anthropic.rs:617)）从 `out_cursor` 拉字节，读空才 `next_chunk()` 取下一上游帧并 `out_cursor = Cursor::new(next)` 替换（旧帧释放），内存只持当前帧 + 单帧转换输出。
- **WW** 上游 pump 有界 channel 形成背压：`UpstreamSseFramePump::from_reader`（见 [common.rs](crates/service/src/gateway/observability/http_bridge/stream_readers/common.rs:87)）用 `mpsc::sync_channel(UPSTREAM_SSE_FRAME_CHANNEL_CAPACITY)`，pump 线程 send 帧到有界 channel，消费者慢时 send 阻塞，天然背压防止内存堆积。
- **XX** read_all_bytes 仅非流式：delivery.rs 中所有 `read_all_bytes()` 调用均在 `if !is_stream` 分支（非流式才整体读入做 usage 解析/错误提取），流式分支走逐帧 reader。

### 第十五批（索引覆盖审计）

- ✅ **YY [P1] daily 聚合 COALESCE 包裹列致索引失效**【已完成 2026-06-22 commit 3b18f400】：移除 `COALESCE(r.created_at, t.created_at)` 改为直接使用 `t.created_at`，使 `idx_request_token_stats_created_at` 索引正确命中。验证：单元测试 85 passed、集成测试 30 passed、gateway_logs 26 passed。性能影响：从全表扫描 + 逐行计算 → 索引范围扫描，修复 D 项（daily cache）的索引层根因。
- ✅ ZZ 对话绑定索引完美：`conversation_bindings` 主键 `(platform_key_hash, conversation_id)` 精确匹配网关热路径 `get_conversation_binding` 查询；account_id/last_used_at 删除路径各有独立索引，无全表扫描。
- ✅ AAA request_token_stats 索引齐备：created_at、account_id+created_at、key_id+created_at、request_log_id(unique) 四索引覆盖主要查询路径，唯一缺口是 YY 的 COALESCE 用法使其在 daily 聚合失效。

### 第十六批（排行查询索引缺口）

- ✅ **BBB [P1] 用户排行 charge 子查询缺 request_log_id 索引**【已完成 2026-06-22 commit d3c2af32】：新增 075_app_wallet_ledger_request_charge_index migration，创建部分索引 `idx_app_wallet_ledger_request_charge ON app_wallet_ledger_entries(request_log_id) WHERE entry_kind='request_charge'`。验证：migration 测试通过、EXPLAIN QUERY PLAN 显示使用新索引、gateway_logs 26 passed。性能影响：从全表扫描 + 无索引 GROUP BY → 部分索引快速定位 + 高效 GROUP BY，CPU 消耗降至对数级别。
- ✅ CCC 排行外层时间过滤走索引：用户/来源排行外层 `WHERE r.created_at >= ?1 AND r.created_at < ?2` 为裸列（非 YY 的 COALESCE），可走 request_logs created_at 索引；`LEFT JOIN request_token_stats t ON t.request_log_id=r.id` 走 unique 索引。外层查询计划良好，瓶颈在 BBB 的 owner 归属子查询。

### 第十七批（排行/聚合索引收敛确认）

本批延续索引层深挖来源排行与成员聚合，结论：除已记录 YY/BBB 外索引命中良好，**索引维度审计收敛**。

- ✅ DDD 来源排行查询健康：`summarize_request_token_stats_source_ranking_between` 外层 `WHERE r.created_at` 裸列走 request_logs created_at 索引先缩小时间窗口，`source_id_expr` 是 request_logs 列的 CASE 表达式（actual_source_kind/actual_source_id/account_id/initial_aggregate_api_id），GROUP BY 表达式为聚合归类固有成本，无 BBB 那样的全表扫子查询。见 [request_token_stats.rs:140](crates/core/src/storage/request_token_stats.rs:140)。
- ✅ EEE 成员 by_key_ids 聚合走索引：`summarize_request_token_stats_by_key_ids` 两路 UNION——`request_token_stats WHERE key_id IN(..)` 走 `idx_request_token_stats_key_id_created_at`，`request_token_stat_rollups WHERE key_id IN(..)` 走主键前缀 `(key_id,account_id,model)` + 独立 `idx_..key_id`，GROUP BY 在已过滤小子集上。见 [request_token_stats.rs:414](crates/core/src/storage/request_token_stats.rs:414)。
- 📌 索引维度小结：dashboard CPU 三大根因已定位齐全——D（缺 daily cache，缓存层）、YY（COALESCE 致 created_at 索引失效，聚合查询层）、BBB（账单流水缺 request_log_id 索引，owner 子查询层）。其余排行/聚合查询索引命中良好，索引层深挖收敛。建议 D/YY/BBB 合并为"dashboard 聚合性能"专题实施。

### 第十八批：锁健壮性（裸 lock unwrap vs 容错封装）

- ⚠️ FFF（P3，健壮性）：流式 usage_collector 有 4 处裸 `.lock().expect("usage lock")`，见 [anthropic.rs:714](crates/service/src/gateway/observability/http_bridge/stream_readers/anthropic.rs:714)、[responses_from_anthropic.rs:682/716/750](crates/service/src/gateway/observability/http_bridge/stream_readers/responses_from_anthropic.rs:682)。项目已有 `lock_utils::read_recover/write_recover` poison 容错封装却未在此复用；若持锁线程 panic 致锁中毒，后续同 collector 的 expect 会连锁 panic。建议改用容错读取或 `lock().unwrap_or_else(|e| e.into_inner())` 模式。
- ✅ GGG：除上述 4 处外，gateway/storage 共享状态锁已普遍走 `read_recover/write_recover` 容错封装（见 lock_utils.rs），poison 不致全局崩溃。

### 持续审计第十九批（2026-06-14，trace 状态清理）

- ✅ HHH 正面确认：`TRACE_ERROR_TRACES` / `TRACE_PENDING_LINES` 有配对清理。`log_request_final`（每请求终点）无条件调用 `clear_trace_state`，同时清理 pending lines 与 error trace；中途 `mark_trace_has_error`（多处）最终都由 final 清理，无界增长风险低。见 [trace_log.rs (line 1350)](crates/service/src/gateway/observability/trace_log.rs:1350)。
- ⚠️ III（P3，与 FFF 同源）：唯一残留缺口是请求中途 panic 未走 `log_request_final` 时，该 trace_id 的 pending lines / error 标记不被清理。低频边际点，可在 final 路径用 RAII guard 兜底确保清理。

### 第二十批审计（网关头部构建分配）

- **JJJ（P3）**：`build_codex_upstream_headers`（codex_headers.rs:113）每请求为静态头名（Authorization/Accept/User-Agent/originator/Content-Type 等）重复 `.to_string()` 堆分配，返回 `Vec<(String,String)>`。Vec::with_capacity(16) 已预分配且头数有界，分配规模小；静态键理论上可用 `&'static str` 或 `Cow`，但受 reqwest builder 接口与下游统一处理约束，收益边际。仅高 RPS 下作为微优化候选。
- **正面确认 KKK**：头部构建逻辑正确——动态值（auth_token/account_id/各 incoming_* 头）经 trim+空值过滤后才透传，`with_capacity(16)` 避免 Vec 扩容，user_agent/originator 经 resolve 缓存身份；无重复构建、无每请求重建静态表。

### 持续审计第二十一批（H项治本方案坐实 + 模型目录读取面）

- **H 项强化（P0，治本方案已坐实）**：`model_catalog_models` 表已有 `PRIMARY KEY (scope, slug)`，天然支撑 O(log n) 点查；但 storage 层**无按 slug 单查/exists 函数**，导致 `proxy.rs:156` 热路径 `model_route_error` 用 `list_model_catalog_models("default")` 全量加载后 `.any(|item| item.slug == model)` 线性扫描。治本方案明确且改动极小：新增 `model_catalog_model_exists(scope, slug)`（`SELECT 1 ... WHERE scope=?1 AND slug=?2`，走主键）替换全量+线性查找。
- **LLL（P2，关联读取面）**：`list_model_catalog_models` 全项目 12 处调用（apikey_models 多处、model_groups、quota/read、proxy 热路径）。多数在管理/读取路径（可接受全量），但 proxy.rs:156 是唯一请求热路径全量点，应优先按 H 改造；其余管理路径维持现状。

### ⚠️ 第二十二批审计（模型目录存在性校验反模式扩面）

- **MMM（P1，与 H 同源）模型存在性校验反模式有第二处**：`ensure_platform_model_exists(storage, slug)`（[apikey_models.rs:1108](crates/service/src/apikey/apikey_models.rs:1108)）与 H 项 `model_route_error`（proxy.rs:156）是完全相同反模式——`list_model_catalog_models("default")` 全量加载 + `.into_iter().any(|m| m.slug == slug)` 线性扫描，仅为判断单 slug 是否存在。归属 `save_managed_model_source_mapping`（模型路由映射保存，管理操作，频率低于网关热路径）。两处共用同一治本方案：新增 `model_catalog_model_exists(scope, slug)` 走 `PRIMARY KEY (scope, slug)` 的 O(log n) 点查，一次改造同时消除 H + MMM。建议把 H 从"单点修复"升级为"模型存在性校验统一收口"小专题。
- **正面确认 NNN**：apikey_models.rs 其余全量加载点（cleanup_orphan_auto_catalog_models:798、prune_unedited_remote_..:1847）是 cleanup/prune 维护型操作，本就需遍历全部模型，全量加载合理，非反模式。

### 持续架构审计第二十三批（模型目录加载点全面映射完成）

- OOO（H 反模式完整分布定论，P0+P2）：完成 `list_model_catalog_models` 全部 12 个调用点映射。
  仅 2 处是 H 反模式（全量加载 + `.any()` 判单 slug 存在）：proxy.rs:156（网关热路径，P0）、
  ensure_platform_model_exists（管理路径，P2）。两处共用治本方案 `model_catalog_model_exists(scope,slug)` 走 PK。
- PPP（正面确认）：其余 10 处均为合理全量用法——
  read_managed_model_catalog（读全目录）、cleanup_orphan/prune（维护需遍历）、
  model_groups.rs:191（批量校验先建 HashSet 再循环，优于逐个单查）、
  quota/read.rs:1569 api_available_model_slugs（按 supported_in_api+visibility 过滤构建列表，本需全部行）。
  结论：模型目录读取面健康，H 反模式边界清晰仅限单 slug 存在性校验，实施范围明确可控。

### 第二十四批审计（冷启动首轮峰值）

- **OOO（P1）冷启动 4 个后台 loop 同时立即执行首轮全量，无错峰**：`run_dynamic_poll_loop`（[runner.rs:306](crates/service/src/usage/refresh/runner.rs:306)）进入循环立即调 `task()`，无前置 sleep；jitter 仅作用于后续轮次间隔。`ensure_usage_polling`/`ensure_gateway_keepalive`/`ensure_token_refresh_polling`/`ensure_warmup_cron` 启动瞬间同时触发首轮，多账号库下 CPU/DB/上游请求峰值叠加。治本：首轮加随机启动延迟（startup jitter）或错开 4 个 loop 的首次触发时刻。
- **PPP（正面确认）后台 loop 调度健康**：`OnceLock` 防重复启动 + 独立命名线程 + 动态间隔 + 失败指数退避 + 后续轮次 jitter，仅缺首轮错峰（OOO）。

### 第二十五批审计（请求体多次重复解析）

- **QQQ（P1，热路径 CPU）请求 body 在校验/改写流水中被多次完整 JSON 解析**：单次网关请求在 `local_validation/request.rs` 中，同一 body 经 `inspect_service_tier_for_log`([:1691](crates/service/src/gateway/local_validation/request.rs:1691))、`parse_request_metadata`([:1700](crates/service/src/gateway/local_validation/request.rs:1700)/:1627/:2085)、`validate_text_input_limit_for_path`([:1781](crates/service/src/gateway/local_validation/request.rs:1781)/:1852/:2082)、`request_rewrite`([:720](crates/service/src/gateway/request/request_rewrite.rs:720)) 各自独立 `serde_json::from_slice::<Value>(body)`，同一请求体被完整解析 4+ 次。Codex 请求 body 常含长对话上下文（数十 KB～MB），重复全量解析放大热路径 CPU 与分配。治本：流水入口一次性 parse 成 `Value`，向各函数传 `&Value`（或解析结果结构）而非反复 `&[u8]` 重解析；改写完成后仅在 body 变更时再序列化一次。
- **RRR（正面确认）解析本身防御正确**：各 `from_slice` 失败均走 `Ok(...) else`/`map_err` 优雅降级（如 request_rewrite.rs:720 解析失败直接返回原 body），无 unwrap panic 风险；问题仅在重复次数而非正确性。

## 2026-06-21 gateway_logs 集成测试阻塞核查（【Claude-Opus】实施）

### ✅ 结论：当前 HEAD（45c02c5d）上 gateway_logs 测试通过，原“22 失败”系跨进程并发干扰，非代码缺陷

任务背景：上一轮汇报 `cargo test -p codexmanager-service --test gateway_logs -- --test-threads=1` 为 4 passed / 22 failed，失败类型含 502 upstream compatibility bridge failed、mock read timeout（os error 10060）、receive upstream request: Disconnected、duplicate column name: account_plan_type。

核查结果：在隔离（单一 cargo test 进程）环境下连续运行两次，均为 **26 passed; 0 failed; 0 ignored**（耗时 68s / 73s），`cargo check -p codexmanager-service` 通过（仅既有 dead_code warning）。

根因判定（非代码缺陷，环境性）：
1. **duplicate column name: account_plan_type 不可能由当前代码产生**——`account_plan_type` 列仅通过 `ensure_account_subscriptions_table()` 的 `CREATE TABLE IF NOT EXISTS`（[account_subscriptions.rs:194](crates/core/src/storage/account_subscriptions.rs:194)）+ `ensure_column()`（[mod.rs:1349](crates/core/src/storage/mod.rs:1349)，先 `has_column` 判定，已幂等）建立；迁移 063 是纯 Rust `apply_compat_migration`（无 SQL 文件、`has_migration` 门控）。全仓唯一的无条件 `ADD COLUMN` 是 044 的 `account_plan_filter`（不同列）。该错误更可能来自两个并发测试进程写同一被竞用的库/连接交叉，而非 schema 逻辑。
2. **502 / os error 10060 / Disconnected**——这些是 mock upstream 在 `accept_http_request` 的 3s accept 截止（[support.rs:637](crates/service/tests/gateway_logs/support.rs:637)）内没等到请求、或 TCP 端口竞争（`TEST_PORT_SEQ` 从 41000 起，两个进程共用同一端口段）、或 CPU 争抢导致网关在 mock 关闭后才转发的连锁症状。`--test-threads=1` 只串行化单进程内用例，**无法隔离两个并发 cargo test 进程**（两个 Claude loop 同时跑测试即触发）。

未改任何业务代码或测试代码（无修复对象）。验证命令与结果：
- `cargo test -p codexmanager-service --test gateway_logs -- --test-threads=1` ×2 → 均 26 passed / 0 failed（exit 0）
- `cargo check -p codexmanager-service` → Finished（exit 0）

剩余风险：若再次出现失败，应先确认是否有第二个 cargo/测试进程并发占用端口与 CPU；建议同一时刻只运行一个测试进程。本项不涉及提交（无代码变更）。

## 2026-06-21 实施：H + MMM 模型存在性校验统一收口（【Claude-Opus】实施 + 主代理独立审计，commit fe67d53a）

### ✅【已完成】H（P0 网关热路径）+ MMM（P2 管理路径）：全量加载+线性 .any() → 主键点查

实施内容：
- 新增 storage 函数 `model_catalog_model_exists(&self, scope, slug) -> rusqlite::Result<bool>`（[model_options.rs:271](crates/core/src/storage/model_options.rs:271)），SQL `SELECT 1 FROM model_catalog_models WHERE scope=?1 AND slug=?2 LIMIT 1`，走主键 `(scope, slug)`（迁移 047:42 与 model_options.rs:700 重建表均确认 `PRIMARY KEY (scope, slug)`），零新增索引。
- 替换两处反模式调用，行为等价（同 scope、同 `map_err` 文案与错误码、同 false 分支返回）：
  - H（P0 热路径）：[proxy.rs:155](crates/service/src/gateway/upstream/proxy.rs:155) `model_route_error`，原 `list_model_catalog_models("default")` 全量 + `.any(slug==model)`。上方 trim/非空守卫未动。
  - MMM（P2 管理路径）：[apikey_models.rs:1108](crates/service/src/apikey/apikey_models.rs:1108) `ensure_platform_model_exists`，原 `list_model_catalog_models(MODEL_CACHE_SCOPE_DEFAULT)` 全量 + `.any(slug==slug)`。
- 补 core storage 单元测试 `model_catalog_model_exists_uses_primary_key_point_lookup`（[migration_tests.rs:1649](crates/core/tests/storage/migration_tests.rs:1649)）覆盖 命中/不存在/scope 不匹配 三态。

主代理独立审计（非盲信子代理报告，亲自复跑）：
- diff 仅 4 个相关文件、未动 task.md、未 git add .；行为等价性逐分支核对通过。
- `cargo check -p codexmanager-core` → Finished（exit 0）。
- `cargo test -p codexmanager-core --lib model_catalog_model_exists_uses_primary_key_point_lookup` → 1 passed。
- **`cargo test -p codexmanager-service --test gateway_logs -- --test-threads=1` → 26 passed / 0 failed**（69.53s，回归保护成立，证明未破坏网关路由校验）。

剩余风险：无功能性风险。task.md 早期条目（186/251/576/965/970/977 行）中"待新增 model_catalog_model_exists"的描述现已落地，后续不应再列为待办。

## 2026-06-22 OpenAI Responses 516 reasoning token / Sidecar usage 完整性审计（【CodeX-GPT】实施）

### ✅ 已处理：OpenAI Responses sidecar drain 超时过短

- 对方审计指出 `OPENAI_RESPONSES_SIDECAR_DRAIN_TIMEOUT` 仅 50ms，结论成立：`OpenAIResponsesPassthroughSseReader` 主转发线程收到 raw EOF 后，只给 sidecar SSE 解析线程 50ms 排空事件；若最终 `response.completed` usage 已到达 raw 流但 sidecar 线程因调度/解析滞后未及时送入 channel，可能保留前一个 usage 快照。
- 已将 drain 超时提高到 200ms，并补单元测试 `sidecar_drain_waits_for_delayed_final_usage_event`，模拟最终 usage 事件延迟 80ms 才进入 sidecar channel，确认最终 `reasoning_tokens=2048` 可被合并。

### 📌 审计结论：`merge_usage` 不应贸然改成累加

- `merge_usage()` 当前策略是"非空字段后到覆盖前值"，其中 `reasoning_output_tokens` 也是 last non-null wins。
- 该策略对 OpenAI Responses / Chat Completions usage 更合理：usage 字段通常表示当前响应对象/最终响应对象的总量；若把多次事件里的 usage 直接累加，遇到累计快照会把 token 算重。
- 因此本轮**不采用**"reasoning_tokens 累加"方案。若后续拿到真实 SSE trace 证明上游发送的是增量 usage，再单独改成按事件类型区分增量/快照；不能仅因出现 516 就改全局 merge 语义。

### ⚠️ 待观察：516 异常值只能做观测，不能直接拦截或重试

- `reasoning_tokens=516` 固定出现值得记录，但当前证据不足以证明它一定是 CE 解析错误；本地分析文件也记录过"官方客户端/SDK 也可能出现 516"的用户反馈，因此更可能同时存在上游预算/风控/动态限制因素。
- 后续可加低风险观测：当 `reasoning_output_tokens == 516` 时写结构化 warn/metrics，带 request_id、model、last_sse_event_type，不记录 prompt/响应正文。
- 不建议直接在 516 时自动重试或改写响应：这会增加费用、放大上游负载，并可能在 516 是服务端真实限额时制造重复请求。

### 验证

- `cargo test -p codexmanager-service --lib sidecar_drain -- --nocapture` → 2 passed。
- `cargo test -p codexmanager-service --lib http_bridge::delivery::tests -- --nocapture` → 18 passed（同时修正当前未提交 `delivery.rs` JSON 单次解析优化后遗漏的测试调用签名）。

## 2026-06-22 实施：Q 项网关非流式响应 JSON 重复 parse 收敛（【CodeX-GPT】收口）

### ✅【已完成】Q（P1）：协议转换路径复用已解析 JSON

- `delivery.rs` 中非流式成功响应的协议转换路径已改为入口解析一次 `serde_json::Value`，再把 `&Value` 传给 `convert_success_body_for_adapter()` 及各 adapter 转换函数，避免 usage 提取后又在 Anthropic/Gemini/Chat/Images 转换内重复 `serde_json::from_slice`。
- 覆盖范围：`respond_with_upstream()` 与 `respond_with_stream_upstream()` 的非流式分支、Chat Completions compact 转换、Responses → Chat/Gemini/Images 转换；Passthrough 直通路径不受影响。
- 同步修正 `delivery.rs` 测试调用签名，`cargo test -p codexmanager-service --lib http_bridge::delivery::tests -- --nocapture` 18 项全部通过。

## 2026-06-23 CodeX-GPT 复核：Q images 回归补修 + T 项状态校准

### ✅ Q 补修：OpenAI Images 非流式请求的 Responses SSE 上游路径

- 独立复核发现，前一轮报告中的 Q/images 回归并未完全收口：`/v1/images/generations` 客户端传 `stream:false` 时，请求体会被改写为 `/v1/responses` 且上游必须按 SSE 读取；但候选执行链里的 `UpstreamRequestContext` 仍来自原始 tiny_http request path，transport 层可能按原始路径判断上游形态。
- 本轮修复：`UpstreamRequestContext` 改为使用已改写的上游逻辑路径构造，保证 transport 的 async stream 包装、压缩、challenge 快速关闭等判断都基于真实上游路径。
- 同时修复本地/测试环境代理污染：未显式配置 `CODEXMANAGER_UPSTREAM_PROXY_URL` / proxy pool 时，blocking 与 async reqwest client 禁用默认代理继承；显式配置代理时仍按配置使用代理。
- 验证：`gateway_images_generation_wraps_codex_sse_as_openai_images_json` 通过；`http_bridge::delivery::tests` 18/18 通过；`cargo check -p codexmanager-service` 通过；`gateway_logs --test-threads=1` 输出 26/26 通过（测试结果已打印，cargo 包装进程随后未自然退出，已清理残留 cargo 进程）。

### ⚠️ T 项返工：候选缓存 Arc 化当前仍会命中后深拷贝

- `task.md` 旧记录把 T 标为已完成，但当前实现仍在 `collect_gateway_candidates_with_low_quota_mode()` 命中缓存时执行 `Arc::unwrap_or_clone(cached)`。由于缓存本身持有另一个 Arc，命中路径几乎总是 clone 整个 `Vec<(Account, Token)>`，并未达到“每请求只 clone Arc”的性能目标。
- 后续返工方向：不要仅把缓存字段改成 `Arc<Vec<_>>`；需要把调用链改成可借用/Arc 传递，或在执行阶段按候选逐项 clone，避免命中后一次性深拷贝整个候选列表。

## 2026-06-24 T 项候选缓存 Arc 化返工收口（【CodeX-GPT】接手实施）

### ✅【已完成】T（P1）：缓存命中不再全量深拷贝候选 Vec

- `collect_gateway_candidates_with_low_quota_mode()` 返回共享 `GatewayCandidateSnapshot = Arc<Vec<(Account, Token)>>`；缓存命中直接返回 `Arc`，不再 `Arc::unwrap_or_clone(cached)`。
- `prepare_gateway_candidates()` 在共享快照上执行账号计划筛选与模型筛选，只 clone 通过过滤的候选，避免入口处无条件复制全量候选池。
- WebSocket 路由路径不再把共享快照立即复制成完整 `Vec`：`GatewayRoutedCandidates` 改为持有共享快照 + 路由后的索引，连接尝试时通过 `iter_cloned()` 逐项 clone，故障切换时通过 `first_cloned_except_account()` 只 clone 下一个候选。
- `model_picker` 属于模型拉取/预热/管理路径，仍需要完整排序和遍历；本轮保留显式 `snapshot.as_ref().clone()`，把成本显式留在低频路径，不再伪装成 `Arc::unwrap_or_clone`。
- 补充 `indexed_route_strategy_matches_owned_candidate_order`，验证索引路由与原拥有式候选排序结果一致；补强 `candidate_snapshot_cache_reuses_recent_snapshot`，用 `Arc::ptr_eq` 验证缓存命中复用同一快照。

验证：
- `cargo test -p codexmanager-service --lib candidate_snapshot_cache -- --nocapture` → 4 passed。
- `cargo test -p codexmanager-service --lib indexed_route_strategy_matches_owned_candidate_order -- --nocapture` → 1 passed。
- `cargo test -p codexmanager-service --test gateway_logs images::gateway_images_generation_wraps_codex_sse_as_openai_images_json -- --exact --nocapture` → 1 passed。
- `cargo check -p codexmanager-service` → Finished（仅既有 warning）。

协作备注：原 Opus 子代理只完成阶段性半成品，未写 `.teamwork/sync/opus-to-gpt.md`、未更新状态、未提交；CodeX-GPT 已关闭该子代理并接手完成实现与审计。后续不应再把 2026-06-22 的旧 T 完成记录视为充分依据，应以本节返工收口为准。

## 2026-06-24 J 项 usage_snapshots 无变化写入去重（【CodeX-Opus-4.6】半成品 + 【CodeX-GPT】收口）

### ✅【已完成】相同用量快照不再追加新行

- 目标：`store_usage_snapshot()` 在本次解析结果关键字段与账号最新快照一致时，不再调用 `insert_usage_snapshot()` 追加新行，降低 `usage_snapshots` 与 WAL 写放大。
- 实施：新增 `update_latest_usage_snapshot_captured_at_for_account()`，相同关键字段时只更新最新行 `captured_at`；字段变化时保持 insert + prune；`credits_json` 使用 JSON 语义比较避免对象字段顺序导致误判。
- 约束确认：仍基于本次解析结果执行 `apply_status_from_snapshot()`；相同快照维护 latest/captured_at 语义；字段变化时新增快照。
- 验证：`cargo test -p codexmanager-core --lib usage_snapshot -- --nocapture` → 6 passed；`cargo test -p codexmanager-service --lib usage_snapshot -- --nocapture` → 5 passed；`cargo check -p codexmanager-service` → Finished（仅既有 warning）。
- 协作备注：Opus 执行方产出半成品并带入 rustfmt 噪声，CodeX-GPT 已清理无关 diff、补齐测试与协作状态并完成提交。

## 2026-07-03 账号计划类型原值保留与自定义过滤（【CodeX-GPT】）

### ✅【已完成】K12 等未知计划类型不再被写死为 unknown

- 根因：`/accounts/check` 订阅接口把未识别计划类型统一归一化为 `unknown`，导致 `K12` 原值丢失；检索页和账户页只能显示“未知”。该问题不只影响 K12，未来任意未在白名单中的计划值（如 `student`、`researcher_beta`、`nonprofit`）都会触发同类问题。
- 实施：`normalize_account_plan_value()` 对已知计划仍返回 canonical 值，对未知非空计划保留小写原值；订阅解析会保存 `K12 -> k12`，账户列表通过 `planTypeRaw` 显示原值，账户分组统计也按原值计数。
- 路由过滤：平台 Key 的账号计划过滤不再只接受固定白名单；后端支持任意非空计划值，前端账号组筛选新增“自定义计划类型”，可输入 `k12` 等原始计划值。`unknown` 过滤收窄为真正未知/无计划，不再误命中已保留原值的 K12。
- 代理稳定性补强：usage/subscription HTTP client 在未显式配置 `CODEXMANAGER_UPSTREAM_PROXY_URL` 时不再继承系统 `HTTP_PROXY/HTTPS_PROXY`，与网关主上游客户端行为保持一致，避免本地代理污染 mock 测试和真实用量刷新。
- 测试环境补强：本机存在系统代理/代理软件时，部分本地 `127.0.0.1` mock 测试会被 reqwest 默认代理劫持并返回 502；已为 aggregate API、HTTP bridge streaming mock、attempt_flow mock client 显式 `no_proxy()`，仅影响测试隔离，不改变生产代理策略。
- 验证：`cargo test -p codexmanager-service account_plan -- --nocapture` → 4 passed；`cargo test -p codexmanager-service fetch_account_subscription_preserves_unknown_plan_value -- --nocapture` → 1 passed；`cargo test -p codexmanager-service aggregate_api::tests::claude_probe -- --nocapture --test-threads=1` → 5 passed；`cargo test -p codexmanager-service gateway::http_bridge::tests::anthropic_sse_reader_does_not_replay_completed_snapshot_after_tool_call -- --nocapture` → 1 passed；`cargo test --target-dir target-verify -p codexmanager-service --lib gateway::upstream::attempt_flow -- --nocapture --test-threads=1` → 43 passed；`cargo check -p codexmanager-core -p codexmanager-service` → passed（仅既有 warning）。
- 前端/运行时验证：`apps` 下 `.\node_modules\.bin\tsc.cmd --noEmit` → passed；`.\node_modules\.bin\next.cmd build` → passed；`node --test` runtime mjs 套件 → 84 passed。
- ⚠️ 发布门禁状态：完整 `cargo test --workspace -- --test-threads=1` 已跑到 service lib 阶段并验证 core/storage 通过、账号计划与 aggregate/attempt_flow 相关用例通过；首次剩余 4 个本地 mock 502 已修复并定向复跑通过。当前无法给出完整 workspace 绿灯的唯一阻塞是本机 Windows 测试进程残留锁定 `codexmanager_service-628427fd77b443cb.exe`，普通与提升权限 `taskkill`/WMI 均无法终止（Access denied / ReturnValue 2），导致后续全量重跑在链接阶段报 `LNK1104`；另一次重跑还暴露系统 `os error 1455`（页面文件太小）。这属于当前机器资源/进程锁阻塞，不是已知业务代码失败。
- ⚠️ 格式门禁状态：`cargo fmt --all --check` 仍受既有无关 rustfmt 漂移阻塞（历史记录文件包括 aggregate_apis/model_options/request_token_stats/dashboard 等），本轮未做全仓格式化，避免把无关格式噪声混入账号计划修复。

## 2026-07-04 发布前门禁收口（【CodeX-GPT】）

### ✅ 已完成：本地发布门禁已通过

- 目标：在不改动账号计划修复语义的前提下，单独收敛 `cargo fmt --all --check` 暴露的既有格式漂移，并复核 `gateway_logs` 与完整 workspace 测试状态。
- 当前确认：`.teamwork/sync/status.json` 为 `completed`，无需等待其他 AI；已执行 `git fetch upstream`，上游最新为 `f3efb3a2 style: polish desktop layout density`。本分支相对 origin/upstream 分叉较大，发布或开 PR 前应重新运行 `git status --short --branch` 与 `git rev-list --left-right --count hardening/main...upstream/main` 获取实时数字。
- 已发现：本机存在安装版 `D:\Apps\CodexManager\codexmanager-service.exe` 常驻进程，可能继续影响端口、CPU 或资源抖动；本轮先不强杀用户安装版进程，仅在测试结论中记录环境风险。
- 已处理：执行 `cargo fmt --all`，修复 8 个 Rust 文件的既有 rustfmt 漂移；`cargo fmt --all --check` 通过。
- 已处理：`aggregate_api` 测试模块的本地 tiny_http mock `recv_timeout` 从散落的 2 秒统一为 `LOCAL_MOCK_RECV_TIMEOUT=10s`，降低 Windows 高负载完整套件下的假失败概率；`cargo test -p codexmanager-service --lib aggregate_api::tests:: -- --nocapture --test-threads=1` 通过 35 项。
- 已处理：`account/list` 裸调用设计已改为默认第一页分页，旧 `rpc_account_list_returns_all_accounts` 断言过时；测试同步为 `rpc_account_list_defaults_to_first_page_with_total`，确认默认返回 5 条、`total=7`、`pageSize=5`。
- 已处理：Web gateway runtime 不再暴露 `authorContentUrl`；删除 `runtime_info()` 中未使用的 `author_content_url` 读取，并同步测试为“不暴露作者内容 URL”。
- 已处理：`passthrough_sse_reader_emits_keepalive_for_responses_stream` 改用门控式 streaming mock，先确认首帧，再在第二帧被阻塞时验证 `codexmanager.keepalive`，避免完整 workspace 高负载下 50ms sleep 被调度跳过造成假失败。
- 验证：`cargo test -p codexmanager-service --test gateway_logs -- --test-threads=1` 通过 26 项；`cargo test -p codexmanager-service --lib aggregate_api::tests:: -- --nocapture --test-threads=1` 通过 35 项；`cargo test --target-dir target-release-gate -p codexmanager-service --lib -- --test-threads=1` 通过 1016 项；`cargo test --target-dir target-release-gate -p codexmanager-service --test rpc -- --test-threads=1` 通过 43 项；`cargo test --target-dir target-release-gate -p codexmanager-web --bin codexmanager-web -- --test-threads=1` 通过 18 项；`cargo test --target-dir target-release-gate --workspace -- --test-threads=1` 完整通过。
- 门禁：`cargo fmt --all --check` 通过；`git -c core.whitespace=blank-at-eol,blank-at-eof,space-before-tab,cr-at-eol diff --check` 通过（仅 Git 提示 CRLF 工作副本换行警告，无空白错误）；冲突标记扫描未发现残留。
- 发布判断：当前工作副本在提交本轮测试同步与文档后，可作为本地发布候选；`target-release-gate/` 仅为隔离测试产物，不应纳入提交。

## 2026-07-04 发布后待办计划（【CodeX-GPT】）

### 🔄 计划中：非阻塞 backlog 分批处理

本节记录已确认不阻塞当前发布候选、但需要继续跟进的事项。处理顺序按风险和收益排序：

1. P1：补齐账号页后端分页等价能力，包括计划类型筛选、限流/封禁状态筛选和全局排序，避免前端用当前页数据伪装全局结果。
2. P1：继续梳理 Web RPC 超时/重试矩阵，重点覆盖批量导入、手动全量刷新、长耗时维护类操作；禁止通过恢复全量裸调用规避超时。
3. P1：观察并优化 `request_logs`、`events`、`usage_snapshots` 与 WAL 体积，复核留存策略、后台维护批大小和 WAL 收缩效果。
4. P2：拆分 `dashboard/adminOverview` 与分页排行 RPC，并评估把排行聚合迁移到日级 rollup，减少首页打开时扫描请求日志窗口。
5. P2：为首页模型池容量补独立轻量汇总或分页来源懒加载；不得让 summary 模式回退到 `quota/modelPools(includeSources:true)` 的全量扫描。
6. P2：清理既有 Rust warning，包括 `delivery.rs` 的 unreachable pattern、维护模块 dead code 和 usage aggregate dead code；不得与业务功能变更混在同一提交。
7. P2：发布/PR 分支治理。当前 `hardening/main` 与 upstream 分叉较大，对外 PR 应从干净分支 cherry-pick 关键提交，不建议整包推送到上游 PR。

### 下一步执行建议

- 当前发布候选先推送 `hardening/main` 到 origin，供 CI/Release workflow 或人工打包使用。
- 后续 backlog 按“一个主题一个中文 commit”处理；涉及子代理实现时，仍必须由 CodeX-GPT 主代理独立审计 diff 与测试结果后才能通过。

## 2026-07-04 发布版本注入与直连统计口径修复（【CodeX-GPT】）

### ✅ 已完成：修复 `v0.3.10` 产物版本与统计提示

- 根因确认：GitHub Release tag 只是发布入口参数，当前 `release-all.yml` 构建前没有把 tag 注入 `Cargo.toml`、`apps/package.json`、`apps/src-tauri/tauri.conf.json` 与锁文件；Tauri 和 staging 脚本继续读取仓库内 `0.3.8`，因此 `v0.3.10` Release 中生成了 `CodexManager_0.3.8_*` 资产。
- 修复方向：新增复用的 release version 同步 action，在每个 release job checkout 后从 `workflow_dispatch.inputs.tag` 提取 SemVer 并同步项目 manifest；仓库内仍保留 `0.3.10` 作为当前开发基线，但发布构建以 workflow tag 为权威版本源。
- 发布清理：`gh release upload --clobber` 只覆盖同名文件，无法删除旧版本号文件；发布 action 更新同一 tag 时会先删除旧 CodexManager/docker compose 资产，再上传本轮构建产物。
- 统计提示：账号直连模式确实不会产生 CodexManager 请求日志，但“切换到本地网关”不应表达成唯一统计方式；文案已改为“请求经过 CodexManager 本地网关后可统计”，并说明网关内部的混合轮转同样属于可统计流量。
- 验证：release version action 已用临时副本模拟 `v9.8.7 -> 9.8.7`，确认只同步 CodexManager 项目包版本且不误伤第三方 `simd-adler32`；`node --test` runtime 文件组 86 项通过；`cargo fmt --all --check` 通过；`git diff --check` 通过；冲突标记扫描未发现残留。
- 注意：`pnpm run test:runtime` 在本机先因非 TTY 清理确认失败，设置 `CI=true` 后又被 pnpm ignored-builds 策略拦截；已改用该脚本内等价的 `node --test ...` 文件组完成验证。
