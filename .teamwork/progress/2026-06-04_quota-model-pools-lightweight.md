# 模型池接口轻量化进度

## 范围

- `quota/modelPools` 新增可选参数：
  - `includeSources`：是否返回每个模型的 sources 明细，默认 `true` 兼容旧调用。
  - `includeConfig`：是否返回 capacity templates 与 account overrides，默认 `true` 兼容旧调用。
  - `sourceKind`：可限制为 `aggregate_api` 或 `openai_account`。
- 首页模型池概览调用 `includeSources:false`、`includeConfig:false`，只读取模型汇总。
- 聚合 API 页调用 `sourceKind:"aggregate_api"`、`includeConfig:false`，避免加载账号池 sources。
- 桌面 Tauri 命令透传新增参数，Web RPC 继续原样透传。
- 新增 RPC 回归测试覆盖 summary-only 不返回 sources/config，以及 aggregate-only 不返回账号来源。

## 非范围

- 本次不新增 `modelPoolSources` 分页详情接口。
- 本次不重写模型池汇总 SQL。
- 本次不处理平台 Key 页面 `usageStats` 全量聚合。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service rpc_quota_model_pools_supports_lightweight_source_filters`
- ✅ `corepack pnpm -C apps run test:runtime`
- ✅ `corepack pnpm -C apps run build`
