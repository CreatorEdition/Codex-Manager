# 2026-07-07 Release 正文同步 Changelog

## 状态

- ✅ 已确认 `v0.3.11` tag 指向包含 0.3.11 changelog 的提交。
- ✅ 已重新验证线上 `v0.3.11` Release 正文曾只包含 GitHub 自动生成的 Full Changelog，缺少 CE 与上游断开点说明。
- ✅ 已使用当前 changelog 的 `0.3.11` 小节直接回填线上 `v0.3.11` GitHub Release 正文，并用无 BOM UTF-8 重新同步。
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

- 后续 release 由 action 自动同步 changelog；发布完成后继续用 `gh release view <tag>` 读回正文，确认包含 Fork / Upstream 与 CE 断开点说明。

## 验证

- ✅ `node --test tests\release-notes-action.test.mjs tests\codex-cli-onboarding-density.test.mjs tests\models-search-style.test.mjs`
- ✅ `apps` 下 `.\node_modules\.bin\tsc.cmd --noEmit`
- ✅ `cargo fmt --all --check`
- ✅ `git diff --check`
- ✅ `rg "^(<<<<<<<|=======|>>>>>>>)" apps crates docs .github .teamwork task.md README.md` 无命中
- ✅ Git Bash 登录 shell 中复刻 action 的 `awk` 抽取逻辑，成功抽取 `v0.3.11` 对应小节并命中 CE / upstream 关键说明。
- ✅ `gh release view v0.3.11 --repo CreatorEdition/Codex-Manager --json body` 读回确认：正文以 `## [0.3.11]` 开头，包含 `### Fork / Upstream` 和 CE 断开点，不再是自动生成的 Full Changelog。
