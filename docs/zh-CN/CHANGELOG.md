# 更新日志

本文件用于记录 CodexManager 的对外可见变更，作为版本历史的唯一事实源。
格式参考 Keep a Changelog，并结合当前项目的实际维护方式做最小收敛。

## [Unreleased]

### Changed
- README 恢复保留 Linux.do 认可社区入口，清理策略只排除作者赞助、远程 author content 与发行推广内容，不再误删社区来源说明。
- 网关本地校验阶段会复用同一次请求 JSON 解析结果生成 service tier 诊断、请求 metadata 与最终文本长度校验；多候选上游尝试也会缓存原始 `prompt_cache_key` 提取结果，降低大请求体重复 parse 开销。
- 网关候选账号快照缓存新增 single-flight 重建协调，同一缓存窗口 miss 时只允许一个线程重建候选池，其余请求等待后复用新快照，降低并发 miss 下的 DB 与 usage snapshot 查询放大。
- Compact transport 路径剥离 `service_tier` 与提取 `prompt_cache_key` 复用同一次 JSON 解析，保持大请求体不参与 prompt key 提取的旧限制。
- 服务启动时 usage polling、gateway keepalive、token refresh 与 warmup cron 后台 loop 会错峰执行首轮任务，降低冷启动瞬时 DB / 网络压力；后续轮询周期保持原配置不变。
- 模型页搜索框补齐 focus 边框与 ring 反馈，输入框自身保持无边框，避免和外层搜索容器出现双边框。
- Codex CLI 首次接入引导弹窗收紧最大宽高、间距和代码块高度，在保留完整引导内容的前提下降低小窗口溢出风险。
- GitHub Release 发布动作改为读取 `docs/zh-CN/CHANGELOG.md` 中对应 `## [版本号]` 小节作为 Release 正文；创建和重跑同一 tag 都会同步正文，缺少版本小节时直接失败，避免正式发布页缺失 CE 与上游分叉说明。
- 已回填线上 `v0.3.11` GitHub Release 正文，发布页会展示 CE 与上游断开点、语义移植清单和不移植范围，而不是只显示 GitHub 自动生成的 Full Changelog。
- 开发态 Next 服务会把 `/api/runtime`、`/api/rpc`、`/api/events/*` 和登录状态路由代理到 `codexmanager-web`，便于源码开发时直接使用 Web 运行壳；CE 版继续跳过 `/api/author-content`。
- Switch 未选中态补齐可见边框、背景和 thumb ring，对浅色/深色主题的开关对比度更稳定。
- 补齐账号排序、模型目录自动拉取与 Web RPC 超时提示的英/韩/俄翻译，并让首页启动快照显式声明完整模型目录需求，恢复 `test:runtime` 全量门禁。

### Fixed
- 请求日志和失败 trace 会统一移除上游 URL 的 query 与 fragment，避免聚合 API `query-secret` 或 `username/password query pair` 密钥进入 DB、UI 或磁盘日志。
- 流式用量采集器遇到 `Mutex` 锁中毒时会记录告警并恢复已有 collector 状态，避免上游 SSE/Responses 转换路径静默丢失 usage、终止事件或错误信息。

## [0.3.11] - 2026-07-07

### Fork / Upstream
- 分叉记录：`v0.3.11` 是 CE 公开发布中明确与上游 `qxcnm/Codex-Manager` 断开直接合并的起点；从该版本起，CE 不再直接 merge upstream，而是按功能语义逐项移植。
- 断开点：CE 发布 tag 为 `v0.3.11`，本轮上游复核基准为 `upstream/main = 6ac01a2a fix: correct dialog layout positioning`；该上游提交只作为语义复核基准，不代表 CE 会整包合并上游。
- 本版本已语义移植网关模型转发规则、模型目录自动远端拉取开关、模型删除防自动拉回、平台密钥今日用量、托盘恢复导航、Web 启动设置读取、Dialog 布局定位和桌面启动渲染性能等低风险功能项。
- 明确不移植作者页、赞助导流、远程 author content、AtomGit 推广和上游整包 README/docs 推广内容；`09223f6f` / `f3efb3a2` 这类产品设计和 UI 密度改造只能拆成页面或组件级小项继续评估。

### Added
- 设置页主题选择改为结构化预览色卡，深色/浅色主题会展示对应界面缩略预览，减少只看单个圆点时的辨识成本。
- 账号页补齐后端分页等价能力，计划类型、状态、搜索和大号/小号优先排序均按全量数据生效。
- 新增模型目录自动远端拉取开关，可关闭“本地目录为空时自动从远端拉取”的行为，仍保留显式远端并入能力。
- 平台密钥用量统计新增今日 token 与费用估算字段，并沿用当前页 Key 限载和成员权限过滤。
- 新增默认关闭的数据库空闲物理压缩保护，可在网关空闲、满足间隔和空闲页阈值时执行 `VACUUM`，压缩期间新网关聊天请求返回 `429`。

### Changed
- Web 运行壳为批量导入、维护、刷新、插件安装等长耗时 RPC 配置独立超时并默认不重试，避免默认 10 秒超时或自动重试放大重操作。
- 观测维护从请求写日志热路径移回后台调度，降低 retention 清理和 WAL checkpoint 对网关请求的同步影响。
- Dashboard 管理员概览与排行/趋势加载拆分，首页模型池容量改走轻量汇总接口，减少首屏和区间刷新时的全量聚合成本。
- Token 日级 rollup 接入后台维护，Dashboard 趋势和排行使用日级汇总 + live mixed 查询，降低长期明细扫描成本。
- 桌面启动与运行渲染路径优化：减少 Next dev 重目录监听、拆分 Zustand store 订阅、跳过无变化状态写入，并使用轻量托盘预览快照。
- `task.md` 收敛为仅记录仍在进行和待处理事项，版本更新所需摘要改由本文件维护。

### Fixed
- 修复 `v0.3.10` 发布产物仍可能使用旧 `0.3.8` 文件名的问题，Release workflow 会从 GitHub tag 注入构建版本，并在上传前清理同 tag 旧资产。
- 修复账号计划类型被写死为 `unknown` 的问题；`K12` 等未来未知但非空的计划会保留原值并可参与筛选。
- 修复请求未经过 CodexManager 本地网关时的统计提示歧义，明确只有经过本地网关的请求才能统计日志、Token 和费用，网关内部混合轮转仍可统计。
- 修复大批量账号导入单个 RPC 过大导致卡死或超时的问题，Web、桌面端直接导入、文件导入和目录导入都会按最多 10 条或 4MB 分批调用。
- 账号导入兼容更多 Token 中转工具常见 JSON 输出，包括 `{accounts:[...]}`、`cpa_batch.tokens[]`、Sub2API `credentials`、9Router `providerSpecificData`、OpenAI session `user/account` 嵌套字段。
- 修复账号页及平台密钥、聚合 API、请求日志、插件自定义源、环境变量设置页等工具栏在中等宽度下被挤出视口的问题。
- 清理作者页、赞助、远程 author content、残留推广静态资源和 `author.qxnm.top` CSP 白名单，避免 CE 版重新引入上游个人推广内容。
- 修复 Dialog 在部分窗口尺寸下的定位和滚动问题，内容区改为可滚动，按钮 class 合并顺序允许调用方覆写布局。
- 修复轻量关闭后从托盘、单实例唤起或系统命令恢复时 debug 主窗口不导航到桌面开发服务根页的问题。

## [0.3.10] - 2026-07-04

### Fixed
- 修复 `v0.3.10` 发布产物仍使用 `0.3.8` 文件名的问题，同步 workspace、前端包、Tauri 桌面端与锁文件版本。
- 发布动作更新同一 tag 时会先清理旧的 CodexManager 产物，避免同一个 Release 中混入旧版本文件。

## [0.3.8] - 2026-05-30

### Fixed
- 修复代理只保留 `x-codex-turn-state` 且请求仍带 `prompt_cache_key` 时，本地 prompt cache 账号绑定无法复用的问题，并补齐 `anchor_mode` 诊断日志。
- 修复 `/v1/messages/count_tokens` 可能被误转发成真实 Responses 生成的问题，改为本地 token 估算。
- 增强 Anthropic-native / Claude Code 兼容路径对 Cloudflare challenge 的处理：禁用兼容流 `zstd`、加入 challenge 冷却与候选切换，并让低额度账号只作为尾部兜底。
- 将 Claude Code 2.x 会话中段的 `system` message 归一为 Responses `developer` role，降低 ChatGPT Codex 后端触发 Cloudflare challenge 的概率。

### Changed
- 发布版本提升到 `0.3.8`，同步更新 workspace、前端包、Tauri 桌面端与锁文件。

## [0.3.7] - 2026-05-27

### Added
- `release-all.yml` 新增 Linux arm64 发布目标，统一进入多平台发版流水线。

### Fixed
- 修复 OpenAI 兼容 `/v1/responses` 的 `prompt_cache_key` 本地账号路由不稳定问题；相同客户端 key 会绑定复用同一上游账号，并且上游请求继续保留客户端原始 key。
- 修复短 `prompt_cache_key`（如 `pc_1`）未参与本地账号绑定的问题，避免 balanced 轮询把同一缓存前缀分散到不同账号。
- 修复 ChatGPT 上游传输错误未按候选账号继续 failover 的问题。
- 修复 Responses WebSocket 上游握手头缺失 beta 标记的问题。
- 修复 `/v1/messages` 兼容流对 `message_stop` 终止事件识别不一致的问题。
- 修复账号 upsert 时已有 token 可能被清空的问题。

### Changed
- 发布版本提升到 `0.3.7`，同步更新 workspace、前端包、Tauri 桌面端与锁文件。

## [0.3.6] - 2026-05-25

### Added
- 新增定时账号预热功能，可按配置周期主动预热账号，降低首次请求时的冷启动失败概率。
- 新增实验性 ChatGPT `/v1/responses` WebSocket 上游传输，设置 `CODEXMANAGER_USE_WEBSOCKET_UPSTREAM=1` 后会优先尝试 WebSocket，失败时回退 HTTP 流式路径，并沿用上游代理、连接超时和协议帧校验。

### Changed
- 请求日志“密钥”列改为展示密钥名称而不是 ID，便于排查调用来源。
- 发布版本提升到 `0.3.6`，同步更新 workspace、前端包、Tauri 桌面端与锁文件。

## [0.3.5] - 2026-05-24

### Added
- 设置页新增 compact 模型转发规则配置，支持仅对 `/v1/responses/compact` 请求按规则改写模型。
- 网关 trace/error 日志补齐 request gate 等待、首包耗时和慢请求 stdout 诊断，便于定位高并发流式请求延迟。

### Fixed
- 修复 Anthropic SSE 在工具调用后重复回放 completed 快照，导致客户端重复显示助手文本的问题。
- 修复 Chat Completions 走 Responses 后 reasoning summary 丢失的问题，流式和非流式响应都会回填兼容 reasoning 字段。
- 修复 region-blocked token refresh 会被后台轮询反复重试的问题，区域受限账号会暂停自动刷新。
- 修复 Aggregate API / 模型同步后陈旧 source routes 和孤儿自动目录模型未清理的问题，避免已删除模型重新出现。
- 为 request gate 提供可选等待上限，设置 `CODEXMANAGER_REQUEST_GATE_WAIT_TIMEOUT_MS=5000` 后可避免同 key/path/model 高并发流式请求长时间队头阻塞。

### Changed
- 发布版本提升到 `0.3.5`，同步更新 workspace、前端包、Tauri 桌面端与锁文件。

### Added
- 聚合 API 余额检测现在会跟随账号池用量轮询周期自动刷新，刷新间隔和设置页“用量轮询线程”保持一致。
- 新增 Codex 图片生成兼容链路：默认按官方 Codex 行为为 `/v1/responses` 自动注入 `image_generation` tool，支持显式 tool 透传，并提供 `/v1/images/generations` 与 `/v1/images/edits` 兼容入口，默认图片工具模型为 `gpt-image-2`。
- Codex CLI 首次接入引导新增 `auth.json` 配置步骤，明确平台 Key、`auth.json` 与 `config.toml` 的关系。

### Fixed
- 修复官方返回的 Spark 专属额度未展示的问题，附加额度会按 `additional_rate_limits[].rate_limit` 继续解析并显示。
- 调整额度详情弹窗布局，附加额度较多时可按两列展示并滚动查看。

### Changed
- 发布版本提升到 `0.3.2`，同步更新 workspace、前端包、Tauri 桌面端与锁文件。
- 发布版本提升到 `0.2.6`，同步更新 workspace、前端包、Tauri 桌面端与锁文件。
- README 不再展示最近提交块，首页只保留稳定的功能与文档入口。
- 设置页恢复“上游总超时”入口，`CODEXMANAGER_UPSTREAM_TOTAL_TIMEOUT_MS` 可通过网关传输设置直接查看和修改，默认 `0` 表示不按总时长截断。
- Nginx 示例配置新增 `/v1/images/` 专用代理块，覆盖图片上传、大体积 `b64_json` 响应与长耗时图片生成场景。
- 请求日志费用估算同步官方 `gpt-5.5`、`gpt-5.5-pro` 与 `gpt-image-*` 价格；图片模型因当前 usage 无 modality 分桶，按官方 Image token 单价保守估算。

## [0.2.3] - 2026-04-15

### Fixed
- 修复原生 Codex 在线程续接场景下的锚点优先级回归：当请求已携带 `conversation_id` 或 `x-codex-turn-state` 时，不再让请求体里的 `prompt_cache_key` 抢占主导权，减少兼容模式下 resume / 上下文续接异常。
- 补齐原生锚点、显式 `prompt_cache_key`、冲突锚点与 Anthropic 原生链路的专项回归测试，避免后续再次把兼容层字段提升到高于原生 Codex 语义的优先级。

### Changed
- 发布版本提升到 `0.2.3`，同步更新 workspace、前端包、Tauri 桌面端与版本锁文件。

## [0.2.0] - 2026-04-12

### Added
- 主页面导航新增后台 keep-alive 缓存与更显眼的整区加载遮罩，桌面端、Web 版与 Docker 部署在页面切换和空闲后回访时的体感更稳定。
- 请求链路排查补齐针对 `service_tier`、Cloudflare challenge 与兼容转发的专项回归测试，覆盖原生 Codex、Claude Code 与 Gemini CLI 关键路径。

### Fixed
- 收敛 Codex 原生直通路径，默认保持官方请求形状，只保留账号选择、认证替换、路由、会话亲和与必要的内部字段处理；Claude / Gemini 继续走协议适配。
- 对齐官方 Codex 出站行为：发往 `chatgpt.com/backend-api/codex/responses` 的 `service_tier=fast` 现会正确映射为上游 `priority`，`/responses/compact` 不再错误携带 `service_tier`。
- 修复 Claude 兼容链路里 `fast` 服务等级映射与错误流模型回显不稳定的问题，减少 `403` 排查时的误导信息。

### Changed
- 发布版本提升到 `0.2.0`，同步更新 workspace、前端包、Tauri 桌面端与对外版本说明。

## [0.1.19] - 2026-04-08

### Added
- 聚合 API 现已支持多认证类型与自定义 `action` 配置，透传路由会按认证方式与动作正确命中对应上游。
- 国际化新增语言持久化与拆分后的消息目录，补齐了仪表盘、模态窗、侧边栏和用量标签等剩余多语言文案。

### Fixed
- 修复聚合 API 透传流里对 Anthropic `message_stop` 事件的识别问题，减少流式响应提前终止或状态误判。
- 修复网关继续把不受支持的 `service_tier` 发往上游的问题；标准 Responses 请求现在只保留受支持值，避免上游拒绝。
- 调整协作与安全入口页为默认中文正文，并整理多语言文档入口，减少发版后从根文档跳转时的断层感。

### Changed
- 正式文档已按语言目录重组，根入口文档与多语言首页文案同步整理。
- 发布版本提升到 `0.1.19`，同步更新 workspace、前端包、Tauri 桌面端、锁文件、README 与 CHANGELOG 的版本说明。

## [0.1.18] - 2026-04-06

### Added
- 新增账号列表直接排序控制与“低额度优先”排序，排查额度紧张账号时更直接。
- 新增 Gemini CLI 基础兼容，补齐 `tools` 流式、MCP 工具名、SSE `tool_call` 等关键链路。

### Fixed
- 修复账号页“额度详情”悬浮卡位置偏移问题，右侧浮层改为按额度概览卡片中线对齐。
- 修复 Gemini completed 工具结果误当正文、流式缓存词元日志采集、请求适配兼容与 token 刷新边界等问题。

### Changed
- 对齐 Gemini → Codex / Responses 请求链路到 CPA 兼容方向，请求会补齐 developer message、tool name 映射、FIFO `call_id`、`reasoning`、`include`、`parallel_tool_calls` 等字段。
- 清理 Gemini 路线未使用代码，并补充 CPA 鸣谢与版本文档说明。
- 发布版本提升到 `0.1.18`，同步更新 workspace、前端包、Tauri 桌面端、锁文件、README 与 CHANGELOG 的版本说明。

## [0.1.17] - 2026-04-05

### Added
- 请求日志新增“最终生效服务等级”口径，HTTP / WS 日志现在会同时保留客户端显式 `service_tier` 和请求改写后的最终值，方便核对平台 Key 默认 `Fast` 是否真正发往上游。
- 设置页新增全局“模型转发规则”，支持使用 `pattern=target` 形式做模型名改写，并在运行时请求改写阶段生效。

### Changed
- 普通平台 Key 的协议类型收敛为“通配兼容 (Codex / Claude Code)”，默认按请求路径自动选择 Claude 或 Codex / OpenAI 语义，减少重复维护多套 Key 的成本。
- 发布版本提升到 `0.1.17`，同步更新 workspace、前端包、Tauri 桌面端、锁文件、README 与 CHANGELOG 的版本说明。

## [0.1.16] - 2026-04-05

### Added
- 新增 `/v1/responses` WebSocket 请求支持，补齐传输类型识别、请求头归一化、代理运行时与请求日志链路。
- 账号页与用量弹窗新增附加额度窗口展示；刷新后会统一显示标准额度与 Code Review / Spark 等额外额度的剩余额度和重置时间。
- 网关 trace 新增 `CLIENT_SERVICE_TIER` 事件，记录 HTTP / WS 原始请求是否显式携带 `service_tier`、原始值以及日志归一化值，便于快速区分客户端显式 `fast` 与平台 Key 默认服务等级。

### Fixed
- 修复 HTTP 与 WS 请求日志中 `service_tier` 口径不一致的问题；现在仅当客户端请求自己显式携带 `service_tier` 时才记录 `fast`，不再把平台 Key 默认值误记成请求显式开启。
- 修复日志页服务等级展示与网关 on-wire 值不一致的问题；`priority` 会统一展示为 `fast`，未显式携带服务等级的请求继续显示为 `auto`。

### Changed
- 发布版本提升到 `0.1.16`，同步更新 workspace、前端包、Tauri 桌面端、锁文件、README 与 CHANGELOG 的版本说明。

## [0.1.15] - 2026-04-03

### Changed
- 发布版本提升到 `0.1.15`，同步更新 workspace、前端包、Tauri 桌面端、运行文档与 README 的版本说明。

## [0.1.14] - 2026-03-30

### Added
- 设置页新增“系统推导”按钮和“单账号并发上限”，可以按当前机器资源一键回填并立即生效。
- 入口层新增短队列等待与超载快速退化，避免高并发直接拖死服务进程。

### Changed
- README、workspace、前端包、Tauri 桌面端与版本一致性校验脚本统一提升到 `0.1.14`。

## [0.1.13] - 2026-03-25

### Added
- 新增“聚合 API”管理页，支持供应商名称、顺序优先级、按 `Codex / Claude` 分类、连通性测试与最小转发上游管理。
- 平台密钥新增 `账号轮转 / 聚合 API 轮转` 策略，聚合 API 轮转会按顺序优先命中对应供应商，再继续下一个渠道。

### Fixed
- 修复桌面端服务启动与页面切换时的自动恢复行为，避免关停后被切页重新拉起，也避免断连时仪表盘误清空数据。

### Changed
- README、workspace、前端包、Tauri 桌面端与版本一致性校验脚本统一提升到 `0.1.13`。

## [0.1.12] - 2026-03-20

### Fixed
- 修复平台密钥名称编辑链路在桌面端未完整透传的问题；现在 Web 与桌面端都能正确保存并回显名称，且支持中文名称。
- 修复平台密钥列表中密钥 ID 默认被截断的问题；现在会直接完整显示，便于核对与排查。

### Changed
- 发布版本提升到 `0.1.12`，同步更新 workspace、前端包、Tauri 桌面端、版本一致性校验脚本与 README 最新版本说明。

## [0.1.11] - 2026-03-20

### Added
- 账号管理新增封禁识别、封禁筛选与“一键清理封禁账号”入口；`account_deactivated` 与 `workspace_deactivated` 会被自动识别为不可用信号，并可在列表中直接筛选和清理。
- 账号列表的 5 小时 / 7 天额度列现在会展示各自窗口的重置时间；仅返回 7 天窗口的 free 账号也会把重置时间显示到 7 天列。
- 平台密钥新增服务等级配置：`跟随请求`、`Fast`、`Flex`，其中 `Fast` 会映射为上游 `priority`，`Flex` 会直传为 `flex`。

### Fixed
- 修复桌面端平台密钥创建 / 编辑时 `serviceTier` 未透传导致“服务等级”保存后不生效、不回显的问题。
- 修复 Web 端在非首页刷新时偶发下载错误文件的问题，并修复部分运行环境下复制 API Key / 登录链接时 `navigator.clipboard.writeText` 不可用导致的复制失败。
- 修复设置页“检查更新”按钮在自动静默检查更新时持续错误转圈的问题；现在只有手动点击时才显示加载状态。

### Changed
- 网关主链路继续向 Codex-first 收口：会话绑定、自动切号即切线程、`originator` / `User-Agent` / 请求压缩等出站语义已进一步对齐，并移除了旧兼容路径遗留的 upstream cookie 链路。
- 设置页补回服务监听地址切换，可在 `localhost` 与 `0.0.0.0` 之间切换；README 与文档也已同步收口到当前主线路径。
- 发布版本提升到 `0.1.11`，同步更新 workspace、前端包、Tauri 桌面端、版本一致性校验脚本与 README 最新版本说明。

## [0.1.10] - 2026-03-18

### Fixed
- 修复 Web / Docker 版误走桌面专属命令分支、账号启用 / 禁用缺少 `sort` 参数导致无法切换状态，以及账号详情刷新失败后状态列不及时回刷的问题。
- 修复禁用账号仍参与手动批量刷新与后台用量轮询的问题；批量刷新与后台轮询现已跳过手动禁用账号，并按并发 worker 执行。
- 修复账号状态语义混乱问题：手动禁用统一为 `disabled`，额度用尽与 `usage endpoint 401` 统一为 `unavailable`，`refresh token 401` 相关链路也统一落成 `unavailable`，前端状态展示同步收口为“已禁用 / 不可用”。
- 修复 Windows 本地 Web 启动器关闭控制台窗口后 `codexmanager-service` / `codexmanager-web` 仍残留后台的问题；启动器现在会通过 Job Object 一并回收子进程。

### Changed
- 发布版本提升到 `0.1.10`，同步更新 workspace、Tauri 桌面端、前端包版本、README 最新版本说明和版本一致性测试。

## [0.1.9] - 2026-03-18

### Added
- 请求日志现在支持后端分页、后端统计、首尝试账号和尝试链路展示，便于区分实际命中账号与 failover 后的最终账号。
- 设置页新增 free / 7 天单窗口账号使用模型配置，free 类账号会统一按设置模型发起请求。

### Fixed
- 修复桌面端启动误判、`/rpc` 空响应、`spawn_blocking` 缺失导致的刷新失败、用量弹窗刷新不同步、首次切页卡顿、Hydration 不一致等稳定性问题。
- 修复 refresh token 误摘号、free 账号请求模型未正确改写、优先账号行为不稳定，以及 `503 no available account` 缺少上下文诊断的问题。
- 修复 release workflow 中 pnpm 版本与当前锁文件不匹配导致的 verify 失败问题。

### Changed
- 旧前端已移除，桌面端与 Web 管理界面统一收口到新的 `apps` 前端；账号管理、平台密钥、请求日志、设置页和导航布局都做了整轮桌面优先重构。
- Codex 请求链路继续按实际 on-wire 行为收口：登录 / callback / workspace 校验、refresh 语义、`/v1/responses` 与 `/v1/responses/compact` 重写、线程锚点、请求压缩、错误摘要和 fallback 诊断均已继续对齐。
- 网关失败诊断和磁盘日志继续收敛，compact 假成功体、HTML/challenge 页、`401 refresh` 子类和 exhausted 候选链路都会输出更明确的摘要。
- 统一将发版版本提升到 `0.1.9`，同步更新 workspace、Tauri 桌面端、`tauri.conf.json` 与前端包版本。
- GitHub Release workflow 中固定的 Tauri CLI 版本已对齐到当前 Rust 侧实际使用版本，减少打包阶段的 CLI / crate 漂移风险。
- 发布文档与 README 已同步更新到 `v0.1.9`，并修正前端静态导出目录说明为 `apps/out`。

## [0.1.8] - 2026-03-11

### Fixed
- Removed the default `https://api.openai.com/v1` fallback path for ChatGPT-backed requests; upstream `challenge` and `403` outcomes are now returned from the primary login-account path instead of being rewritten into local fallback errors.
- ChatGPT login-account requests now recover from `401` by refreshing the local `access_token` with the stored `refresh_token` and retrying the current request once.

### Changed
- ChatGPT login-account turns now use `access_token` directly on the primary upstream path and no longer mix in `api_key_access_token` semantics.
- Synthetic gateway terminal failures now return structured OpenAI-style `error.message / error.type / error.code` payloads while keeping the existing trace and error-code headers.

## [0.1.7] - 2026-03-11

### Added
- 设置页新增网关传输参数：支持直接配置上游流式超时与 SSE keepalive 间隔，并在 service 运行时热生效。
- 桌面端启动快照补齐：仪表盘统计、账号用量状态、请求日志首屏会优先恢复最近一次快照，减少源码运行或服务重启后的全 0 / 未知状态。

### Fixed
- 修复 `codexmanager-web` 的访问密码会话跨重启仍可继续使用的问题；关闭并重新打开 Web 进程后，旧登录 Cookie 会失效，需要重新验证密码。
- 修复源码运行 `codexmanager-web` 时的启动与根路由兼容问题，减少 Web 静态资源与根路径在 Axum 路由下的不一致行为。
- 修复长输出场景下的 SSE 空闲断流重连问题，降低长时流式响应被误判中断的概率。
- 修复设置页保存上游代理、平台密钥创建弹窗关闭与重复提交、登录成功后账号表格未刷新等桌面交互问题。
- 修复模型拉取默认附加版本参数导致的部分上游兼容性问题，模型请求改为默认不附带版本号。
- 修复账号导入与登录回调两条链路的账号归并逻辑不一致问题，统一按同一身份规则新增或更新账号。
- 修复 Claude / Anthropic `/v1/messages` 适配在多 MCP server 场景下的工具截断问题；不再因前 16 个工具占满而丢失后续 server 的工具。
- 修复 Claude / Anthropic `/v1/messages` 链路缺少长工具名缩短与响应还原的问题，避免 MCP 工具名过长时映射不稳定。

### Changed
- 网关失败响应增加结构化 `errorCode` / `errorDetail` 字段，并同步补充 `X-CodexManager-Error-Code`、`X-CodexManager-Trace-Id` 响应头，便于客户端与日志系统追踪失败链路。
- 协议适配继续对齐 Codex / OpenAI 兼容生态：进一步统一 `/v1/chat/completions`、`/v1/responses`、Claude `/v1/messages` 的转发语义，并稳固 `tools` / `tool_calls`、thinking / reasoning、流式桥接和响应还原链路。
- 设置页与运行时配置继续收敛：背景任务、网关传输、上游代理、Web 安全等高频配置统一由 `app_settings` 持久化并回填到当前进程。
- 桌面与 service 启动链路继续治理，收敛 Web / service / desktop 之间的启动边界与启动顺序，减少源码运行与打包运行的行为分叉。
- 项目内部继续推进长期维护向的重构治理：前端主入口、设置页、请求日志视图、Tauri 命令注册、service 生命周期、gateway protocol adapter、HTTP bridge、upstream attempt flow 等区域已进一步拆分模块边界，减少大文件与根层门面耦合。
- service / gateway 目录结构继续收敛，更多通配导入、跨层直连和超长门面清单已被显式依赖与分层模块替代，后续维护和协议回归定位成本更低。
- 发布链路继续收敛到 `release-all.yml` 单入口，并复用前端构建产物与协议回归基线，减少重复构建与发布时的协议回归风险。

## [0.1.6] - 2026-03-07

### Fixed
- 修复 `release-all.yml` 在手动关闭 `run_verify` 时仍强依赖预构建前端工件的问题；各平台任务缺少 `codexmanager-frontend-dist` 时会自动回退到本地 `pnpm install + build`。

### Changed
- Windows 桌面端发布产物继续收敛，仅保留 `CodexManager-portable.exe` 便携版，不再额外生成 `CodexManager-windows-portable.zip`。
- 完善 SOCKS5 上游代理支持与归一化，并补充设置页中的代理协议提示文案。

## [0.1.5] - 2026-03-06

### Added
- 新增“按文件夹导入”：桌面端可直接选择目录，递归扫描其中 `.json` 文件并批量导入账号。
- 新增 OpenAI 上游代理配置与请求头收敛策略开关，可在设置页直接保存并即时生效。
- 补充 chat tools 命中探针脚本，便于本地验证工具调用是否真正命中与透传。

### Fixed
- 修复 `tool_calls` / `tools` 相关回归：补齐 chat 聚合路径中的工具调用保留、工具名缩短与响应还原链路，避免工具调用在 OpenAI 兼容返回、流式增量和适配转换中丢失或名称错乱。
- 完善 OpenClaw / Anthropic 兼容返回适配，确保工具调用、SSE 增量和非流式 JSON 响应都能按兼容格式正确还原。
- 请求日志追踪增强，补充原始路径、适配路径和更多上下文，便于定位 `/v1/chat/completions -> /v1/responses` 转发与协议适配问题。

### Changed
- 网关协议适配进一步对齐 Codex CLI：`/v1/chat/completions` 与 `/v1/responses` 两条链路统一收敛到 Codex `responses` 语义，上游流式/非流式行为与官方更接近，兼容 Cherry Studio 等客户端的 OpenAI 兼容调用。
- 设置页顶部常用配置改为统一的三列行布局，代理配置与其保持一致；同时支持关闭窗口后隐藏到系统托盘运行。
- 发布流程整合为单一一键多平台 workflow，并收敛桌面端产物形态；Windows 直接提供 portable exe，macOS 统一使用 DMG 分发。

## [0.1.4] - 2026-03-03

### Added
- 新增“一键移除不可用 Free 账号”：批量清理“不可用 + free 计划”账号，并返回扫描/跳过/删除统计。
- 新增“导出用户”：支持选择本地目录并按“一个账号一个 JSON 文件”导出。
- 导入兼容增强：支持 `tokens.*`、顶层 `*_token`、camelCase 字段（如 `accessToken` / `idToken` / `refreshToken`）自动识别。

### Fixed
- 兼容旧 service：前端导入前会自动归一化顶层 token 格式，避免旧版后端报 `missing field: tokens`。

### Changed
- 账号管理页操作区整合为单一“账号操作”下拉菜单，替代右侧多按钮堆叠，界面更简洁。

[Unreleased]: https://github.com/CreatorEdition/Codex-Manager/compare/v0.3.11...HEAD
[0.3.11]: https://github.com/CreatorEdition/Codex-Manager/compare/v0.3.10...v0.3.11
[0.3.10]: https://github.com/CreatorEdition/Codex-Manager/compare/v0.3.8...v0.3.10
[0.3.8]: https://github.com/CreatorEdition/Codex-Manager/compare/v0.3.7...v0.3.8
[0.3.7]: https://github.com/CreatorEdition/Codex-Manager/compare/v0.3.6...v0.3.7
[0.3.6]: https://github.com/CreatorEdition/Codex-Manager/compare/v0.3.5...v0.3.6
[0.3.5]: https://github.com/CreatorEdition/Codex-Manager/compare/v0.3.4...v0.3.5
[0.2.6]: https://github.com/CreatorEdition/Codex-Manager/compare/v0.2.3...v0.2.6
[0.2.3]: https://github.com/CreatorEdition/Codex-Manager/compare/v0.2.0...v0.2.3
[0.2.0]: https://github.com/CreatorEdition/Codex-Manager/releases/tag/v0.2.0
[0.1.19]: https://github.com/CreatorEdition/Codex-Manager/releases/tag/v0.1.19
[0.1.17]: https://github.com/CreatorEdition/Codex-Manager/releases/tag/v0.1.17
[0.1.16]: https://github.com/CreatorEdition/Codex-Manager/releases/tag/v0.1.16
[0.1.15]: https://github.com/CreatorEdition/Codex-Manager/releases/tag/v0.1.15
[0.1.14]: https://github.com/CreatorEdition/Codex-Manager/releases/tag/v0.1.14
[0.1.13]: https://github.com/CreatorEdition/Codex-Manager/releases/tag/v0.1.13
[0.1.12]: https://github.com/CreatorEdition/Codex-Manager/releases/tag/v0.1.12
[0.1.11]: https://github.com/CreatorEdition/Codex-Manager/compare/v0.1.10...v0.1.11
[0.1.10]: https://github.com/CreatorEdition/Codex-Manager/releases/tag/v0.1.10
[0.1.9]: https://github.com/CreatorEdition/Codex-Manager/releases/tag/v0.1.9
[0.1.8]: https://github.com/CreatorEdition/Codex-Manager/releases/tag/v0.1.8
[0.1.7]: https://github.com/CreatorEdition/Codex-Manager/releases/tag/v0.1.7
[0.1.6]: https://github.com/CreatorEdition/Codex-Manager/releases/tag/v0.1.6
[0.1.5]: https://github.com/CreatorEdition/Codex-Manager/releases/tag/v0.1.5
[0.1.4]: https://github.com/CreatorEdition/Codex-Manager/releases/tag/v0.1.4
