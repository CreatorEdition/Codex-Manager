# Codex-Manager CE 当前任务看板

> 本文件只保留仍在进行或尚未收口的工作。已完成变更进入 `docs/zh-CN/CHANGELOG.md`，上游差异结论进入 `.teamwork/discussions/2026-07-07_上游差异与CE清理结论.md`。

## 当前待处理（2026-07-07）

1. P2 上游差异巡检
   - 当前上游基准：`upstream/main = a614b559 docs: tidy repository links in readme`。
   - 已确认：`09223f6f` / `f3efb3a2` 不能整包移植，只能拆成页面或组件级小项；`a614b559` 为 README 链接整理但包含 AtomGit / Gitee / 官网 / 赞助入口，不按 CE 当前 README 直接移植。
   - 已完成拆分小项：模型页搜索框 focus 反馈、Codex CLI 引导弹窗密度压缩、开发态 Web runtime rewrites、Switch 对比度。
   - 当前无明确可直接移植的小项；后续新增上游提交继续先拆分评估。
   - 禁止项：作者页、赞助、远程 author content、AtomGit 推广、上游整包 README/docs 推广内容。
   - 保留项：README 中的 Linux.do 认可社区入口需要保留，不能按作者/赞助推广残留误删。

2. P2 分支 / PR 治理
   - 当前 fork 与 upstream 分叉较大，对外 PR 应从干净分支 cherry-pick 关键提交。
   - 不建议整包提交当前 CE 主线到 upstream；先按主题拆分，确保每个 PR 都能独立审计。

3. P2 低优先级性能观察
   - 候选缓存 stale-while-revalidate 可选评估：single-flight 已完成，SWR 还需确认是否会延长低额度 / 封禁账号的旧快照使用窗口。
   - 请求体 JSON parse 深水区继续观察：request rewrite / transport 层仍有独立解析路径；已先收敛本地校验阶段和多候选 `prompt_cache_key` 提取。

## 固定发布门禁

1. 前端：相关 `node --test tests\*.mjs`、`apps` 下 `tsc --noEmit`，必要时补浏览器截图验收。
2. 后端：`cargo fmt --all --check`、相关 `cargo test` 定向用例；发布前按资源情况跑完整 workspace 门禁。
3. Git：禁止 `git add .`，按主题精确暂存；提交信息使用中文。
4. Release：发布 tag 必须在 `docs/zh-CN/CHANGELOG.md` 存在对应 `## [<version>]` 小节；GitHub Release 正文由该小节同步，缺失时不得发布；发布后必须用 `gh release view <tag>` 验证正文包含 Fork / Upstream 与 CE 断开点说明，不能只剩 GitHub 自动生成的 Full Changelog。
