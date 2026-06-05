# 2026-06-05 启动快照客户端默认轻量化

## 负责人

- 【CodeX-GPT】

## 背景

- 后端 `startup/snapshot` 裸 RPC 已改为轻量默认，但前端公共封装 `serviceClient.getStartupSnapshot()` 仍默认显式传入 `includeUsageAggregate/includeTodaySummary/includeRecentLogs/includeApiModels: true`。
- 这会让后续调用即使不理解负载边界，也自动请求完整快照，容易在几千账号和大量日志场景下触发 10 秒级 RPC 超时。

## 变更

- `apps/src/lib/api/service-client.ts` 默认只传账号/API Key 预载上限，不再默认开启四个重型 include。
- `apps/src/hooks/useDashboardStats.ts` 作为首页仪表盘调用方，显式 opt-in 完整启动快照，保留首页统计语义。
- `apps/tests/startup-snapshot.test.mjs` 新增源码级回归测试，防止公共客户端默认值重新变重。
- `apps/package.json` 将新测试纳入 `test:runtime`。

## 验证

- 通过 `corepack pnpm -C apps run test:runtime`。
- 通过 `corepack pnpm -C apps run build`。
- 通过 `git diff --check`。
