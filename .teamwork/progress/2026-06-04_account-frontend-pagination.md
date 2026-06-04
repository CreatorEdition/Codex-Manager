# 账号页前端分页接入进度

## 范围

- `useAccounts()` 接收账号列表分页参数，并将 `page/pageSize/query/filter` 作为 React Query key 维度。
- 账号管理页通过后端 `account/list` 返回的 `total/page/pageSize/items` 驱动表格、页脚和翻页，不再先全量加载再本地 `filter + slice`。
- 搜索下推到后端 `query`；状态筛选仅下推当前后端已支持的 `available -> active`、`low_quota -> low`。
- 启动快照仅在第一页、空搜索、空筛选时作为占位数据，并按当前 pageSize 截取。
- 禁用依赖全量账号列表的计划类型筛选和全局排序按钮，避免把当前页数据误当作全局数据写回。
- 导出、清理等全局操作的提示改为不再使用当前页数量伪装全量数量。

## 非范围

- 本次不改启动快照全量预载。
- 本次不改 `account/usage/list` 全量 usage 读取。
- 本次不实现计划类型、限流、封禁的后端全局分页筛选。
- 本次不重构全局排序接口；后续需要后端排序 API 或完整全局排序数据。

## 验证

- ✅ `corepack pnpm -C apps run test:runtime`
- ✅ `corepack pnpm -C apps run build`
- ✅ `cargo fmt --all --check`
- ⚠️ `corepack pnpm -C apps run test:e2e -- tests/accounts-toolbar.spec.ts` 未能执行到断言阶段：当前机器缺少 Playwright Chromium 浏览器二进制 `chrome-headless-shell.exe`；测试启动链路已通过临时 `pnpm` shim 进入 Playwright。
