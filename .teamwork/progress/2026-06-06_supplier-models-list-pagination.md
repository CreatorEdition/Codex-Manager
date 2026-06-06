# 2026-06-06 聚合 API 供应商模型列表分页与 SQL 下推

## 角色

- 【CodeX-GPT】

## 背景

`aggregateApi/supplierModels/list` 即使传入 `supplierKey/providerType`，storage 也会先读取整张 `aggregate_api_supplier_models` 表，再在 Rust 层过滤。供应商模型模板增多后，聚合 API 模型池弹窗会产生不必要的全表扫描。

## 变更

- storage 层将 `supplier_key/provider_type` 过滤下推到 SQL。
- 列表新增 `page/pageSize`，裸调用默认第一页 100 条。
- 返回结果新增 `total/page/pageSize`，旧前端仍可通过 `items` 兼容读取。
- import 路径保留读取指定供应商全部模板的语义。

## 验证

- 已通过 `cargo fmt --all --check`
- 已通过 `cargo test -p codexmanager-core supplier_model_list_filters_in_sql_and_paginates`
- 已通过 `cargo test -p codexmanager-service supplier_models_list_bare_call_defaults_to_first_page`
- 已通过 `cargo check -p codexmanager-service`
- 已通过 `git diff --check`
- 未执行前端 `pnpm` 校验：当前 PowerShell PATH 中没有 `pnpm`
