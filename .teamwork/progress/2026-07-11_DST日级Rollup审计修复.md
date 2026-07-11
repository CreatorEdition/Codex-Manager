# DST 日级 Rollup 审计修复进度

## 状态

✅ 已完成，等待主代理独立审计

## 审计结论

- 原实现以“当前本地零点 + 固定 86400 秒”反推历史日边界，在 America/New_York 春季切换日会错过 1 小时，秋季切换日会跨入相邻自然日。
- SQLite 表中的 `day_start` 足以保存真实本地日零点，不需要修改表结构；问题位于边界生成与 mixed 查询切分。
- 修复采用显式 `(day_start, day_end)` 区间，由服务层使用 `chrono::Local` 生成本地自然日，core 只按传入边界聚合。

## 实施项

- [x] 增加最早未固化明细查询，维护任务从该明细所在本地日开始按日期顺序 rollup。
- [x] 增加显式本地日区间的维护入口，保留原固定秒数入口兼容既有调用。
- [x] 增加显式日区间的 daily / user / source mixed 查询。
- [x] Dashboard 默认七日范围按本地日期偏移，不再以秒数回退。
- [x] 增加 America/New_York 春季 23 小时与秋季 25 小时测试，覆盖明细删除前后汇总一致性。
- [x] 完成格式化、core/service 定向测试与提交。

## 验证结果

- `cargo fmt --all --check`：通过。
- `cargo test -p codexmanager-core storage::request_logs::tests::`：17 项通过。
- `cargo check -p codexmanager-service`：通过，无警告。
- `cargo test -p codexmanager-service dashboard`：首次运行的 4 项 dashboard 单元测试全部通过；命令随后未正常退出并持续占用测试可执行文件。再次链接时出现 `LNK1104` 文件占用，属于本机残留测试进程锁，不是编译错误。

## 风险控制

- 未修改 SQLite 表结构与历史 rollup 数据。
- 无法解析本地午夜时返回错误并停止维护，不使用 UTC 对齐猜测替代本地日边界。
- 维护仍受原批次上限约束，未完成的日级明细会留待下轮继续处理。
