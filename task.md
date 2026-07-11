# Codex-Manager CE 当前任务看板

> 本文件只保留仍在进行或尚未收口的工作。已完成变更进入 `docs/zh-CN/CHANGELOG.md`，上游差异结论进入 `.teamwork/discussions/2026-07-07_上游差异与CE清理结论.md`。

## 当前待处理（2026-07-07）

0. P0 定价分层与刷新状态补充修复（🔄 进行中）
   - 定价：按请求最终 `effective_service_tier` 区分 Standard / Priority，补齐官方 Priority 种子与回归测试；历史费用重算不在本轮自动执行。
   - 刷新：修复 `refresh_token_expired` 永久过滤遗漏；Token 刷新成功后立即触发用量验证并缩短状态恢复延迟；明确 disabled/banned 的后台刷新边界。
   - 区域诊断：参考 Clash Verge 多 IP 服务映射与失败切换，但仅用于出口诊断；启动检查、区域阻断事件触发检查和低频兜底均不得直接替代上游接口判定。
   - 前端：刷新完成后同步账号实体状态，并展示可验证的出口诊断信息与刷新结果。
   - 主代理负责对子代理补丁独立审计、集成测试与 PR 更新。

1. P0 审计问题修复与独立复核（✅ 已完成）
   - 前端/Web：补齐 Web command 映射、移除错误重复 RPC、恢复 direct-mode 门禁、修正桌面构建陈旧产物判断。
   - 后端：修复账号删除后的候选缓存失效、成员账号池权限边界、OAuth 日志脱敏及多模式候选缓存隔离。
     - 后端子项已经主代理独立审计并集成，权限、缓存与日志定向测试通过。
   - 发布/工具：修正 CE GHCR 镜像归属，移除数据库工具的本机硬编码与默认破坏性行为。
   - 日级统计：改用显式本地自然日边界执行 rollup 与 mixed 查询，覆盖 DST 23/25 小时切换日。
     - 已知粒度限制：历史自定义范围若只覆盖某自然日首尾半日，且该日明细已按 retention 清理，只能使用整日 rollup，无法精确还原半日数据；接口不得把整日汇总伪装成该半日结果。
   - 主代理已逐提交审计、退回并修复 DST 稀疏空日性能问题，完成前后端构建与集成测试；待 PR 合并后从当前看板移除。

2. P2 上游差异巡检
   - 当前上游基准：`upstream/main = a614b559 docs: tidy repository links in readme`。
   - 已确认：`09223f6f` / `f3efb3a2` 不能整包移植，只能拆成页面或组件级小项；`a614b559` 为 README 链接整理但包含 AtomGit / Gitee / 官网 / 赞助入口，不按 CE 当前 README 直接移植。
   - 已完成拆分小项：模型页搜索框 focus 反馈、Codex CLI 引导弹窗密度压缩、开发态 Web runtime rewrites、Switch 对比度。
   - 当前无明确可直接移植的小项；后续新增上游提交继续先拆分评估。
   - 禁止项：作者页、赞助、远程 author content、AtomGit 推广、上游整包 README/docs 推广内容。
   - 保留项：README 中的 Linux.do 认可社区入口需要保留，不能按作者/赞助推广残留误删。

3. P2 分支 / PR 治理
   - 当前 fork 与 upstream 分叉较大，对外 PR 应从干净分支 cherry-pick 关键提交。
   - 不建议整包提交当前 CE 主线到 upstream；先按主题拆分，确保每个 PR 都能独立审计。

4. P2 低优先级性能观察
   - 候选缓存 stale-while-revalidate 可选评估：single-flight 已完成，SWR 还需确认是否会延长低额度 / 封禁账号的旧快照使用窗口。
   - 请求体 JSON parse 深水区继续观察：本地校验、多候选 `prompt_cache_key` 提取、compact transport、非原生 Responses 默认 `stream=true` 后文本长度校验复用、Official Responses 标准化后 Value 复用、request rewrite 输出 Value 旁路已收敛；local count tokens、WebSocket 包装等路径仍需按风险继续拆小项评估。

## 固定发布门禁

1. 前端：相关 `node --test tests\*.mjs`、`apps` 下 `tsc --noEmit`，必要时补浏览器截图验收。
2. 后端：`cargo fmt --all --check`、相关 `cargo test` 定向用例；发布前按资源情况跑完整 workspace 门禁。
3. Git：禁止 `git add .`，按主题精确暂存；提交信息使用中文。
4. Release：发布 tag 必须在 `docs/zh-CN/CHANGELOG.md` 存在对应 `## [<version>]` 小节；GitHub Release 正文由该小节同步，缺失时不得发布；发布后必须用 `gh release view <tag>` 验证正文包含 Fork / Upstream 与 CE 断开点说明，不能只剩 GitHub 自动生成的 Full Changelog。
