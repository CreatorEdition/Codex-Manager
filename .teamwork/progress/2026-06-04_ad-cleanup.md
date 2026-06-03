# 2026-06-04 广告与推广清理进度

## 来源

子代理 A 初稿 + 主窗口【CodeX-GPT】审计补漏。

## 已处理范围

- 删除 README 与多语言文档中的赞助商、打赏、联系方式、外部推广和生态推荐内容。
- 删除前端作者页、侧边栏入口、keep-alive 路由、相关测试和 i18n 文案。
- 删除 app settings 中 `authorSponsors`、`authorServerRecommendations` 以及 `authorContent/get` RPC 暴露。
- 删除 Web 运行壳 `/api/author-content` 代理和 runtime 中的作者内容 URL。
- 删除支付宝、微信、社群二维码和 sponsor 图片等静态推广资源。

## 主窗口审计要求

- 已执行广告关键词残留扫描，无命中。
- 已执行 `corepack pnpm -C apps run test:runtime`，60 个测试通过。
- 已执行 `corepack pnpm -C apps run build`，Next 静态构建通过且路由列表不再包含 `/author`。
- 已执行 `cargo check -p codexmanager-service -p codexmanager-web`，通过。
- 已执行 `cargo test -p codexmanager-service app_settings`，相关测试通过。
- 已执行 `cargo test -p codexmanager-web runtime_info_reports_web_gateway_capabilities` 和 `cargo test -p codexmanager-web web_auth_allows_static_assets_without_session`，均通过。
- `cargo fmt --all --check` 仍失败，但剩余差异限定在 `crates/service/src/gateway/observability/request_log.rs` 与 `crates/service/src/rpc_dispatch/requestlog.rs`，属于既有无关格式问题，后续按 CI/rustfmt 独立提交处理。
