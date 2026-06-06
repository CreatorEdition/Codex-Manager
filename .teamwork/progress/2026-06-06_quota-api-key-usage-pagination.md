# 2026-06-06 API Key 配额用量默认分页

## 角色

- 【CodeX-GPT】

## 背景

`quota/apiKeyUsage` 仍是裸全量 RPC：无参数时会读取全部 API Key、全部 quota limit、全量 token stats 汇总，以及全部 key+model 用量明细。Key 多或 token stats/rollup 大时，响应体和 CPU 都会放大。

## 变更

- 新增 `keyIds/page/pageSize/includeModels` 入参。
- 裸调用默认只返回第一页 100 条。
- 默认不返回模型明细；只有显式 `includeModels=true` 时才对当前页 Key 聚合 key+model 用量。
- quota limit、token stats 汇总都按当前页 Key ID 下推。
- Tauri 命令与 Web 客户端透传分页参数。

## 验证

- 已通过 `cargo fmt --all --check`
- 已通过 `cargo test -p codexmanager-service quota_api_key_usage_bare_call_defaults_to_first_page`
- 已通过 `git diff --check`
- 未执行前端 `pnpm` 校验：当前 PowerShell PATH 中没有 `pnpm`
