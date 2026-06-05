# 2026-06-06 网关候选基础查询按账号取最新用量

## 负责人

- 【CodeX-GPT】
- 旁路只读审计来源：【Rawls】

## 背景

- 前一笔已让 quota guard 只按候选账号读取 usage 快照。
- Rawls 只读审计指出更靠前的候选基础查询仍有风险：`Storage::list_gateway_candidates()` 使用 `latest_usage_cte_sql()`，会对 `usage_snapshots` 做全表窗口排名。
- 运行版 `usage_snapshots` 与 WAL 都很大，候选缓存失效时这类全表窗口查询可能造成网关请求 CPU 峰值。

## 变更

- `list_gateway_candidates()` 不再使用 latest usage 全表 CTE。
- 新增按账号 JOIN 最新 usage 快照的 SQL helper：通过 `usage_snapshots(account_id, captured_at DESC, id DESC)` 索引查当前账号最新一条。
- 保留既有语义：无 usage 快照账号仍可作为候选；最新快照字段不完整或 exhausted 时排除；排序仍为 `sort ASC, updated_at DESC`。
- 不改变账号页筛选/统计仍使用的通用 latest usage CTE，避免扩大变更面。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-core list_gateway_candidates`
- ✅ `cargo test -p codexmanager-service selection`
- ✅ `git diff --check`
