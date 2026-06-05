# 2026-06-05 网关候选池 trace 限载

## 负责人

- 【CodeX-GPT】

## 背景

- 运行版 `D:\Apps\CodexManager` 的 `gateway-trace.log` 中，`CANDIDATE_POOL` 会在单次请求里展开数百个候选账号。
- 该路径会在路由前构造完整 `Vec<String>`，同时写入包含完整账号 ID 的长日志；在几千账号场景下会放大 CPU、内存分配、磁盘写入和敏感信息暴露风险。

## 变更

- `request_setup.rs` 只为候选池 trace 采样前 12 个候选，不再为全量候选池构造字符串。
- `trace_log.rs` 将 `CANDIDATE_POOL` 输出改为 `candidate_count/sample_limit/truncated_count/sampled_candidates`。
- 候选池样本账号改为 `account_fp` 短指纹，避免在池级日志中泄露完整账号 ID。

## 验证

- ✅ `cargo fmt --all --check`
- ✅ `cargo test -p codexmanager-service candidate_pool`
- ✅ `git diff --check`
