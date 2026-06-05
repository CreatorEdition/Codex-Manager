# 2026-06-05 模型池 summary 避免来源全量扫描

## 负责人

- 【CodeX-GPT】

## 背景

- 用户抓包显示裸 RPC：`quota/modelPools` 只带 `addr` 时仍会超时。
- 既有默认已经不返回 `sources/config`，但后端仍可能为容量汇总扫描全部聚合 API、账号、tokens、usage snapshots、subscriptions 和容量配置。
- 几千账号场景下，这种“返回轻量但计算全量”的路径仍会拖慢首页和外部裸 RPC。

## 变更

- `quota/modelPools` 在 `includeSources=false` 且未指定 `sourceKind` 时只返回模型/价格 skeleton，不再扫描账号池或聚合 API 来源。
- 显式传 `sourceKind` 或 `includeSources=true` 时保留现有来源容量计算语义。
- 回归测试断言裸调用不累加 `sourceCount`，也不计算 `totalRemainingTokens`。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service rpc_quota_model_pools_defaults_to_summary_without_sources_or_config`
- ✅ `cargo test -p codexmanager-service rpc_quota_model_pools`
- ✅ `git diff --check`
