# 2026-07-07 Release 正文同步 Changelog

## 状态

- ✅ 已确认 `v0.3.11` tag 指向包含 0.3.11 changelog 的提交。
- ✅ 已确认当前本机 `gh` token 失效，无法直接读取或编辑 GitHub Release 正文。
- ✅ 已修正发布 action：创建或更新 GitHub Release 时，正文从 `docs/zh-CN/CHANGELOG.md` 的对应版本小节生成。
- ✅ 缺少对应版本小节时发布动作会失败，避免正式 Release 页面没有 CE 与上游分叉说明。

## 变更范围

- `.github/actions/publish-github-release/action.yml`
- `apps/tests/release-notes-action.test.mjs`
- `apps/package.json`
- `docs/zh-CN/CHANGELOG.md`
- `docs/zh-CN/release/构建发布与脚本说明.md`
- `docs/zh-CN/release/发布与产物说明.md`
- `task.md`

## 后续操作

- 使用有效 GitHub token 后，可重跑 `release-all.yml` 的 `v0.3.11` 或手动编辑 Release 正文，使线上发布页同步本仓库 changelog。

## 验证

- ✅ `node --test tests\release-notes-action.test.mjs tests\codex-cli-onboarding-density.test.mjs tests\models-search-style.test.mjs`
- ✅ `apps` 下 `.\node_modules\.bin\tsc.cmd --noEmit`
- ✅ `cargo fmt --all --check`
- ✅ `git diff --check`
- ✅ `rg "^(<<<<<<<|=======|>>>>>>>)" apps crates docs .github .teamwork task.md README.md` 无命中
- ✅ Git Bash 登录 shell 中复刻 action 的 `awk` 抽取逻辑，成功抽取 `v0.3.11` 对应小节并命中 CE / upstream 关键说明。
