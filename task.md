# Codex-Manager CE 当前任务看板

> 本文件只保留仍在进行或尚未收口的工作。已完成变更进入 `docs/zh-CN/CHANGELOG.md`，上游差异结论进入 `.teamwork/discussions/2026-07-07_上游差异与CE清理结论.md`。

## 当前待处理（2026-07-07）

1. P2 上游 UI 小项语义评估
   - 当前上游基准：`upstream/main = 6ac01a2a fix: correct dialog layout positioning`。
   - 已确认：`09223f6f` / `f3efb3a2` 不能整包移植，只能拆成页面或组件级小项。
   - 候选小项：Codex CLI 引导弹窗密度压缩、开发态 Web runtime rewrites。
   - 禁止项：作者页、赞助、远程 author content、AtomGit 推广、上游整包 README/docs 推广内容。

2. P2 分支 / PR 治理
   - 当前 fork 与 upstream 分叉较大，对外 PR 应从干净分支 cherry-pick 关键提交。
   - 不建议整包提交当前 CE 主线到 upstream；先按主题拆分，确保每个 PR 都能独立审计。

3. P2 低优先级性能观察
   - query-secret 日志脱敏统一。
   - 候选缓存 single-flight / stale-while-revalidate。
   - 流式 usage collector 锁中毒容错。
   - 冷启动首轮后台 loop 错峰。
   - 请求体多次 JSON parse 收敛。

## 固定发布门禁

1. 前端：相关 `node --test tests\*.mjs`、`apps` 下 `tsc --noEmit`，必要时补浏览器截图验收。
2. 后端：`cargo fmt --all --check`、相关 `cargo test` 定向用例；发布前按资源情况跑完整 workspace 门禁。
3. Git：禁止 `git add .`，按主题精确暂存；提交信息使用中文。
