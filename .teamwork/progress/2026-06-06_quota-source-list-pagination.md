# 2026-06-06 配额来源列表默认分页

## 角色

- 【CodeX-GPT】

## 背景

`quota/sourceList` 仍是旧式裸 RPC：无参数时会读取全部 API Key、全部聚合 API、全部 OpenAI 账号，并逐账号读取最新用量。几千账号或外部脚本裸调用时容易触发超时、CPU 峰值和 SQLite 读放大。

## 变更

- `quota/sourceList` 新增 `sourceKind/sourceIds/page/pageSize`。
- 裸调用默认 `sourceKind=all`、`page=1`、`pageSize=100`，返回 `items/total/page/pageSize`。
- API Key 分支只对当前页 Key 查询 quota limit 与 token usage。
- OpenAI 账号分支只对当前页账号批量读取最新 usage 快照。
- 聚合 API 与模型分配也只装饰当前页来源。
- Tauri 命令与 Web 客户端透传分页参数。

## 验证

- 已通过 `cargo fmt --all --check`
- 已通过 `cargo test -p codexmanager-service quota_source_list_bare_call_defaults_to_first_page`
- 已通过 `git diff --check`
- 未执行前端 `pnpm` 校验：当前 PowerShell PATH 中没有 `pnpm`
