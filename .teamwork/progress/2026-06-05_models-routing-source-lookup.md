# 2026-06-05 模型页来源名称按需查询

## 负责人

- 【CodeX-GPT】

## 背景

- 用户反馈 `/api/rpc` 在几千账号场景下容易出现 10 秒级超时，根因不是 timeout 太短，而是部分页面仍通过无界 list 构建本地字典。
- 模型管理页 `/models` 为展示模型路由来源名称，原先调用 `accountClient.list()` 与 `accountClient.listAggregateApis()`，会额外读取账号和聚合 API 列表。

## 变更

- 在 `apps/src/app/models/page.tsx` 中从 `routing.sourceModels` 与 `routing.mappings` 提取实际引用的 `sourceId`。
- 账号来源改用 `accountClient.lookupAccounts(ids)`。
- 聚合 API 来源改用 `accountClient.lookupAggregateApis(ids)`。
- 保留原展示逻辑：查到名称时显示名称，查不到时继续显示压缩后的 source ID。

## 验证

- 通过 `corepack pnpm -C apps run test:runtime`。
- 通过 `corepack pnpm -C apps run build`。
- 通过 `git diff --check`。
