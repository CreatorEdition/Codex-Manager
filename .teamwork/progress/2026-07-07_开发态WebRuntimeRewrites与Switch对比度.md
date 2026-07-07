# 2026-07-07 开发态 Web Runtime Rewrites 与 Switch 对比度

## 状态

- ✅ 已拉取上游并确认 `upstream/main = 0f1b03db style: improve switch contrast`。
- ✅ 已从 `09223f6f` 拆出开发态 Web runtime rewrites 小项。
- ✅ 已明确排除 `/api/author-content`，不恢复作者内容代理或远程作者内容能力。
- ✅ 已语义移植 `0f1b03db` 的 Switch 对比度小改。

## 变更范围

- `apps/next.config.ts`
- `apps/src/components/ui/switch.tsx`
- `apps/tests/next-dev-runtime-rewrites.test.mjs`
- `apps/tests/switch-contrast-style.test.mjs`
- `apps/package.json`
- `README.md`
- `docs/zh-CN/report/环境变量与运行配置说明.md`
- `docs/zh-CN/CHANGELOG.md`
- `task.md`
- `.teamwork/discussions/2026-07-07_上游差异与CE清理结论.md`

## 验证

- ✅ `node --test tests\next-dev-runtime-rewrites.test.mjs tests\switch-contrast-style.test.mjs tests\dashboard-direct-mode.test.mjs tests\release-notes-action.test.mjs`
- ✅ `node --test tests\i18n-page-coverage.test.mjs tests\startup-snapshot.test.mjs tests\dashboard-direct-mode.test.mjs`
- ✅ `npm.cmd run test:runtime`，108 项通过
- ✅ `apps` 下 `.\node_modules\.bin\tsc.cmd --noEmit`
- ✅ `cargo fmt --all --check`
- ✅ `git diff --check`
- ✅ `rg "^(<<<<<<<|=======|>>>>>>>)" apps crates docs .github .teamwork task.md README.md` 无命中

## 追加收口

- `test:runtime` 首次全量执行时暴露既有 i18n 缺口和首页启动快照显式性缺口。
- 已补齐英/韩/俄翻译，并保持托盘预览仍可通过 `includeApiModels=false` 使用轻量快照。
