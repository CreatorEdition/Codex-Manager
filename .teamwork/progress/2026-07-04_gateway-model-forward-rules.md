# P1 网关模型转发规则统一语义移植

执行者：【CodeX-GPT gpt-5.4 子代理】
主审计：【CodeX-GPT】

## ✅ 已完成

- 旧 `gateway.compact_model_forward_rules` 在运行时同步时合并进 `gateway.model_forward_rules`，重复规则去重，旧键写空清理。
- 请求改写统一调用普通模型转发规则，compact 请求不再读取独立 compact 规则。
- compact 子代理本地校验路径不再通过隐藏 compact 规则生成模型覆盖，交由后续统一请求改写处理。
- 设置页移除 compact 独立规则编辑器；普通规则编辑器新增行时保留已有空行，便于逐步填写。
- 主审计补强：legacy compact 规则为空时不触发重复迁移；`local_validation` compact 分支保留显式 compact model override，但不再读取隐藏 compact 转发规则。

## ✅ 验证

- `cargo test -p codexmanager-service app_settings_get_migrates_legacy_compact_model_forward_rules -- --nocapture`：通过。
- `cargo test -p codexmanager-service app_settings_get_ignores_empty_legacy_compact_model_forward_rules -- --nocapture`：通过。
- `cargo test -p codexmanager-service --lib model_forward_rules -- --nocapture`：9 项通过。
- `node --test apps\tests\settings-page-helpers.test.mjs`：4 项通过。
- `apps` 下 `.\node_modules\.bin\tsc.cmd --noEmit`：通过。
- `cargo fmt --all --check`：通过。
- `git -c core.whitespace=blank-at-eol,blank-at-eof,space-before-tab,cr-at-eol diff --check`：通过。

## ⚠️ 未验证

- 未跑完整 workspace 测试。
