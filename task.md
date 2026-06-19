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

### ⚠️ 待处理

- `cargo test --workspace` 尚未全量执行，后续安全/CI 阶段再跑完整工作区测试。
- 旧工作副本 `C:\code\CodeX\Codex-Manager` 仅保留为审计参考，实际修改转入 `Codex-Manager-CE`。
- 账号页计划类型筛选、限流/封禁状态筛选和全局排序还缺后端分页等价能力，本次前端避免用当前页数据伪装全局筛选。
- `dashboard/adminUsageSummary` 已完成默认 TopN SQL 下推；后续仍应拆 `dashboard/adminOverview` 与分页排行 RPC，并考虑把排行聚合进一步迁移到日级 rollup，减少每次首页打开扫描请求日志窗口。
- 运行版只读诊断显示 `events` / `usage_snapshots` / WAL 是体积主因；后台用量轮询、token refresh 候选、用量列表裸调用、usage aggregate、网关候选配额保护、网关候选基础查询、用量快照维护剪枝、用量刷新失败事件降噪、失败账号轮询冷却、按 Key token_stats 聚合、空 token_stats 写入跳过、观测维护后台化、成功模型列表日志降载、启动迁移轻量化、观测维护分批清理、日志页轮询降载、管理员用量排行 SQL 下推、聚合 API 余额后台轮询限载和账号/API Key 裸列表默认分页已限载/下推/移出请求线程/减少写入，后续仍需继续审计 request_logs 留存策略与 WAL 收缩效果。
- Web RPC 仍需继续按方法梳理超时/重试配置，特别是批量导入、手动全量刷新和长耗时维护类操作；不得通过恢复全量裸调用来规避超时。
- 首页模型池卡片在 summary 模式下容量数字会显示未知；后续如要展示容量，应通过独立轻量汇总或分页来源接口懒加载，不能回退到裸 RPC 全量扫描。
- ✅【已完成 2026-06-19 commit dd794ab3】更新检测默认地址已从 `qxcnm/Codex-Manager` 统一改为 `CreatorEdition/Codex-Manager`，覆盖 `DEFAULT_UPDATE_REPO`、设置页 fallback release URL、环境变量目录默认值、github.rs 测试夹具和多语言文档链接，`CODEXMANAGER_UPDATE_REPO` 覆盖能力保持不变。审计备注：`items.rs` 提交带入 CRLF 行尾符噪声（真实变更仅 2 行但 diff 显示 146 行），index 已一致不再持续产生噪声，逻辑无误。
- 🔄【进行中 2026-06-19】前端 conflict marker 收敛：13 个文件中 package.json/useApiKeys.ts/ko.ts/ru.ts 已收敛（commit b4c5e092，保留 HEAD 分页优化与上游 codexProfile/平台模式/i18n 新功能）；剩余 9 个文件（apikeys/page.tsx 9处、useAccounts.ts 5处、model-catalog-modal.tsx 3处、account-client.ts 3处、runtime-capabilities.ts 3处、settings/page.tsx 2处、transport-web-commands.test.mjs 2处、logs/page.tsx 大范围、useManagedModels.ts 1处）正由子代理逐文件收敛提交。注意是多路嵌套冲突（HEAD + 82970aaa + cf306b11 + 49d70518 + fccf5a63），且存在跨文件依赖（useManagedModels 的价格规则方法依赖 account-client）。
- 2026-06-19 只读诊断确认用量分析 / 区间消耗没有已结束日期缓存：现有 `request_token_stat_rollups` 只按 `(key_id, account_id, model)` 聚合，没有 `day_start`、用户、实际来源、状态桶、请求数维度；`dashboard/adminUsageSummary`、成员仪表盘趋势、日志今日摘要、配额今日用量仍会扫描 `request_token_stats` 或 join `request_logs`。后续应新增日级 rollup 表，已结束日读缓存，当前日读 live stats 并加短 TTL。
- ✅【已完成 2026-06-19 commit 8c94c84c】`account/usage/aggregate` SQL 下推已接回：`read_usage_aggregate_summary()` 改为直接调用 `storage.usage_aggregate_summary()`，移除 `list_accounts()` + `latest_usage_snapshots_by_account()` 的 Rust 全量聚合。storage 层 `usage_aggregate_summary_matches_bucket_semantics` 测试覆盖 SQL 路径桶语义。审计备注：service 层保留的 `compute_usage_aggregate_summary` 及其测试仍验证旧 Rust 路径，两条路径缺交叉一致性测试，后续可补一个对照测试断言 SQL 与 Rust 结果一致以防回退。
- 2026-06-19 只读诊断确认请求日志错误去重只做了用量刷新失败事件节流，日志页仍直出 `request_logs.error` 原文，没有错误类别聚合结果。后续应新增 `requestlog/errorSummary` 或扩展 summary，按规范化 error code / 账号 / 来源 / 6 小时窗口聚合，返回 count、lastSeen、代表样例，避免 401 token refresh、网络断开、stream transport 等重复错误淹没 UI。
- ✅【已完成 2026-06-19 commit 678b04da】`refresh_token_unknown_401` 不再被永久过滤：token refresh 候选（`tokens.rs`）与用量轮询候选（`accounts.rs`）查询中的通配 `refresh_token_invalid:%` 收窄为仅排除确认永久无效子类型（reused/invalidated/invalid_grant/app_session_terminated）。unknown_401 保留进入候选，靠 `next_refresh_at` 退避与 6 小时失败冷却保护。补充 `list_tokens_due_for_refresh_keeps_transient_unknown_401` 回归测试。审计发现独立问题：`crates/service/tests/usage/usage_refresh_status_tests.rs` 未被任何测试入口 `mod` 引入，是孤立文件不会被编译运行，需后续修复测试注册。
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

### B. 前端统计 hook 叠加放大首页 CPU（P0，新发现）

首页 `apps/src/app/page.tsx` 同时挂载三个独立统计 hook，各自触发后端重聚合：
- `useDashboardStats()`（`page.tsx:1023`，STALE 15s）
- `useDashboardAdminUsageSummary(...)`（`page.tsx:1070`，STALE 30s，`useDashboardAdminUsageSummary.ts:50`）
- `useMemberDashboardSummary(true)`（`page.tsx:1476`，STALE 30s，`useMemberDashboardSummary.ts:38`）

问题：三个 hook 的统计窗口高度重叠（今日/区间/账号），但分别打不同 RPC，导致首页打开瞬间触发 3 路独立聚合扫描。后续应合并为单一 `dashboard/adminOverview` 轻量端点或共享缓存，避免重复扫同一时间窗口。

### C. 网关热路径全表扫描（P0，新发现）

- 聚合 API 转发热路径 `resolve_aggregate_api_rotation_candidates()`（`crates/service/src/gateway/upstream/protocol/aggregate_api.rs:726`）每次请求 `list_aggregate_apis()` 全量读取后 Rust 层过滤 `status=active` + `provider_type`，随来源数量线性放大。应新增 `list_active_aggregate_apis_by_provider()` SQL 下推 + `(status, provider_type)` 复合索引，必要时叠加候选缓存（参考网关账号候选缓存 TTL 机制）。

### D. 已结束日期缓存（closed-day daily rollup）缺失（P0，核心方案）

现状：`request_token_stat_rollups` 主键仅 `(key_id, account_id, model)`，无日期/用户/来源/状态桶维度（`crates/core/src/storage/request_token_stats.rs:1094`），无法回答"某天/某区间"缓存查询。

建议方案（一次性下发）：
1. 新增 `request_token_stat_daily_rollups` 表：主键 `(day_start, key_id, account_id, source_kind, source_id, user_id, model, status_bucket)`，列含 input/cached/output/total/reasoning tokens、estimated_cost、request_count、success_count、error_count、source_rows、updated_at。
2. 维护任务把"已结束自然日"的 `request_token_stats` 固化进日级表后再清理明细。
3. 查询策略改为：历史已结束日读日级 rollup（命中即返回），当前日读 live `request_token_stats` 并加 30-60 秒短 TTL；区间 = 历史天缓存 + 今日 live 拼接。
4. `dashboard/adminUsageSummary`、`memberSummary` 趋势、`requestlog` 今日摘要、`quota` 今日用量统一走该策略，避免反复全窗口聚合。

### E. 错误聚合查询缺索引支撑（P1/P2，关联错误去重需求）

现状：`request_logs` 已有 created_at / status_code / method / key_id / account_id / trace_id / actual_source 等多组复合索引（`request_logs.rs:49-815`），但无 `error`、`success` 字段索引。

影响：用户要的"450 条压成 5 类"错误去重汇总，若新增 `requestlog/errorSummary` 按规范化 error code GROUP BY，当前需全表扫描 error 文本。

建议：
1. 写入时落规范化列 `error_code`（复用网关 `errors/mod.rs:65` 与 `usage_http.rs` 的归类逻辑），而非运行时正则匹配原文。
2. 新增 `(error_code, created_at DESC)` 索引支撑错误聚合。
3. `requestlog/errorSummary` 返回 `{error_code, count, last_seen, sample_message}`，日志页展示去重结果。

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

### H. 网关请求路由校验全量加载模型目录（P0，新发现）

`model_route_error()`（`crates/service/src/gateway/upstream/proxy.rs:155`）在请求转发路由校验时，用 `list_model_catalog_models("default")` 全量读取模型目录后 `.any(|item| item.slug == model)` 做线性查找，仅为判断单个 model 是否存在。该函数在 proxy.rs:664 和 1316 两处请求路径被调用，随模型目录规模线性放大 CPU 与内存分配。

建议：新增 `model_catalog_model_exists(scope, slug)` 或 `find_model_catalog_model_by_slug()` storage 单查（带索引），替换全量加载 + Rust 线性查找。

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

- **N（P1 调度器冗余全量）**：插件调度器 `run_due_tasks_once()` 已用 `list_due_plugin_tasks(now, 100)` SQL 下推取到期任务，但随后又调 `list_plugin_installs()` 全量 + `list_plugin_tasks(None)` 全量，仅为计算下次 sleep 秒数，见 [scheduler.rs](crates/service/src/plugin/scheduler.rs:39) 与 [plugins.rs:303](crates/core/src/storage/plugins.rs:303)。优化：新增 SQL 直接查 `MIN(next_run_at)`（限 enabled + 非 manual + 已安装插件），避免每个调度 tick 把全部 installs/tasks 搬入内存。当前无该 SQL（已确认），需新增 `next_plugin_task_due_at()`。

- **O（P0 失败冷却不分类，用户重点关注）**：`schedule_token_refresh_failure_retry()` 对所有 token refresh 失败统一施加固定冷却（默认 6 小时 `DEFAULT_TOKEN_REFRESH_FAILURE_COOLDOWN_SECS`），不区分错误性质，见 [refresh/mod.rs:780](crates/service/src/usage/refresh/mod.rs:780)。问题：永久失败（refresh_token_reused/invalidated/expired/invalid_grant）应长冷却或判死；临时失败（网络错误、5xx、Unknown401 服务端抖动）应短退避 + 探测，不应一律压 6 小时。关键：调用处 [refresh/mod.rs:935](crates/service/src/usage/refresh/mod.rs:935) 已持有 `err` 文本，且 [usage_http.rs](crates/service/src/usage/usage_http.rs:369) 已有 `classify_refresh_token_auth_error_reason()` 分类函数可直接复用，修复成本低——只需把分类结果传入冷却调度并按类别分流。

- **P（P1 无指数退避）**：token refresh 失败无连续失败计数与指数退避，每次失败都重置为固定冷却。`tokens` 表已有 `last_refresh_attempt_at` 列但无 `consecutive_failure_count`（已确认无退避字段），见 [tokens.rs:174](crates/core/src/storage/tokens.rs:174)。优化：新增连续失败计数列，临时失败按 `base * 2^min(n, cap)` 指数退避（如 1min→2min→4min→…→上限 30min），成功即清零；与 O 项配合实现"抖动快速恢复、持续失败逐步退避、永久失败判死"三级策略。

### 📌 用户原始诉求对照（来自本轮对话）

用户明确要求两类优化，对照 task.md 已记录项：
1. **用量分析/区间消耗已结束日期缓存** → 对应 **A（日级 rollup 缓存）** + **B（account/usage/aggregate 接回 SQL 下推）** + 本批确认的首页 P0 入口（dashboard/adminUsageSummary、startup/snapshot）。
2. **错误代码去重汇总（450 条压成 5 类）** → 对应 **E（requestlog/errorSummary 按规范化 error code 聚合）**。
3. **token 失败不应一直刷、区分吊销与服务端抖动** → 对应本批 **O + P**（分类冷却 + 指数退避），unknown_401 先按临时失败处理，永久无效仅限 reused/invalidated/expired/invalid_grant/app_session_terminated。

