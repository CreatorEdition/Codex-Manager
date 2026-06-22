# OpenAI Codex SDK Reasoning Token 516 问题技术分析

**分析者**: CodeX-Opus  
**日期**: 2026-06-22  
**问题来源**: https://github.com/router-for-me/CLIProxyAPI/discussions/3937#discussioncomment-17387969

## 问题摘要

用户报告通过第三方客户端（CLIProxyAPI、OpenCode）或 Python Codex SDK 调用 OpenAI GPT-5.5 时，经常出现 `reasoning_tokens` 固定为 **516** 的现象，且此时模型返回错误答案。同一问题通过 Codex CLI 直接登录则正常。

### 测试用例

**逻辑题**：黑色袋子中有三种口味糖果，每种口味有圆形和星形两种形状，给定具体数量，问最少取多少颗糖果才能保证拿到不同形状的苹果味和桃子味。

- **正确答案**: 21 颗
- **降级答案**: 29 颗（当 reasoning_tokens = 516 时）

### 关键发现

| 访问方式 | reasoning_tokens | 答案准确性 | 状态 |
|---------|-----------------|-----------|------|
| Codex CLI 直接登录 | 变化（数百到数千） | ✅ 正确 | 正常 |
| Python Codex SDK + 本地登录 | 固定 516 | ❌ 错误 | 降级 |
| CLIProxyAPI + Codex | 固定 516 | ❌ 错误 | 降级 |
| OpenCode 第三方客户端 | 固定 516 | ❌ 错误 | 降级 |

## 技术分析

### 1. Codex SDK 流处理架构

#### 官方 Codex SDK (TypeScript)

查看 `openai/codex/sdk/typescript/src/exec.ts`：

```typescript
async *run(args: CodexExecArgs): AsyncGenerator<string> {
  const commandArgs: string[] = ["exec", "--experimental-json"];
  
  // ... 配置参数构建 ...
  
  const child = spawn(this.executablePath, commandArgs, {
    env,
    signal: args.signal,
  });
  
  // 关键：直接读取子进程 stdout，逐行解析
  const rl = readline.createInterface({
    input: child.stdout,
    crlfDelay: Infinity,
  });
  
  for await (const line of rl) {
    yield line as string;  // 原始流式输出，无中间聚合
  }
}
```

**关键特征**：
- 直接 spawn `codex` CLI 子进程
- 使用 readline 逐行读取 stdout
- **无额外的 HTTP 代理层**
- **无流式聚合或缓冲逻辑**
- 环境变量直接传递：`CODEX_INTERNAL_ORIGINATOR_OVERRIDE=codex_sdk_ts`

### 2. Codex-Manager-CE 流处理架构

#### CE 项目流处理链路

查看 `crates/service/src/gateway/observability/http_bridge/stream_readers/openai_responses.rs`：

```rust
pub(crate) struct OpenAIResponsesPassthroughSseReader {
    raw_upstream: GatewayByteStream,          // 原始上游字节流
    observer: OpenAIResponsesSidecarObserver, // 侧车观察者（解析 SSE）
    out_cursor: Cursor<Vec<u8>>,              // 输出缓冲游标
    usage_collector: Arc<Mutex<PassthroughSseCollector>>, // Usage 收集器
    usage_text_state: OpenAIResponsesOutputTextState,
    keepalive_frame: SseKeepAliveFrame,       // 保活帧
    // ...
}
```

**关键架构**：
1. **双通道设计**（Tee）：
   ```rust
   let (raw_upstream, sidecar_upstream) = upstream.into_body().tee();
   ```
   - `raw_upstream`: 直接透传给客户端
   - `sidecar_upstream`: 后台线程解析 SSE 事件

2. **后台解析线程**：
   ```rust
   thread::spawn(move || {
       let stream = byte_stream.eventsource();
       loop {
           match stream.as_mut().poll_next(&mut cx) {
               Poll::Ready(Some(Ok(event))) => {
                   let lines = event_to_sse_lines(&event);
                   if let Some(parsed) = OpenAIResponsesEvent::parse(&lines) {
                       tx.send(OpenAIResponsesSidecarItem::Event(parsed));
                   }
               }
               // ...
           }
       }
   });
   ```

3. **Usage 聚合逻辑**（`aggregate/output_text.rs:195-209`）：
   ```rust
   let reasoning_output_tokens = usage
       .and_then(|map| map.get("output_tokens_details"))
       .and_then(Value::as_object)
       .and_then(|details| details.get("reasoning_tokens"))
       .and_then(Value::as_i64)
       .or_else(|| {
           usage.and_then(|map| map.get("reasoning_output_tokens").and_then(Value::as_i64))
       })
       .or_else(|| {
           usage
               .and_then(|map| map.get("completion_tokens_details"))
               .and_then(Value::as_object)
               .and_then(|details| details.get("reasoning_tokens"))
               .and_then(Value::as_i64)
       });
   ```

### 3. 核心差异对比

| 维度 | Codex CLI | Codex-Manager-CE | 影响 |
|-----|----------|------------------|------|
| **进程模型** | 子进程直接调用 | HTTP 代理转发 | CE 多一层网络抽象 |
| **流处理** | readline 逐行读取 | 双通道 Tee + 后台线程 | CE 有额外的缓冲和解析 |
| **SSE 解析** | CLI 内部处理（Rust 原生） | eventsource-stream crate | CE 依赖第三方 SSE 解析库 |
| **Token 统计** | 无中间聚合 | 后台 sidecar 聚合 | CE 需要异步等待 Usage 事件 |
| **身份标识** | `codex_sdk_ts` / 直接 CLI | 代理请求（可能带不同 User-Agent） | **可能触发 OpenAI 风控** |
| **请求路径** | `openai.com/api` 直连 | 经过 CE 网关 → OpenAI | 多一跳，可能被识别为代理流量 |

## 问题根因分析

### ⚠️ 关键更新：官方 Codex 客户端也会出现 516 问题

**新发现**：用户反馈即使使用 **OpenAI 官方 Codex 客户端** 也会出现 reasoning_tokens=516 现象。

**这彻底改变了问题性质**：
- ❌ ~~不是第三方代理导致的风控~~
- ❌ ~~不是 CE 代码特有的 bug~~
- ✅ **很可能是 OpenAI API 本身的行为**（推理预算限制、负载均衡或模型降级）

### 推测 1: OpenAI 模型侧推理预算限制（新主要推测）

**证据**：
1. **官方客户端也会出现** - 排除了"第三方代理触发风控"的假设
2. `reasoning_tokens=516` 固定值 - 不是网络截断导致的随机值
3. 重试有时能得到正确答案 - 说明问题是动态的，非固定策略
4. 不同访问方式均有概率出现 - 说明是后端行为而非客户端问题

**推测机制**：
```
OpenAI 后端根据以下因素动态分配 reasoning budget：
- 服务器负载（高峰期降级）
- 账号配额状态（使用量接近限额时降级）
- 请求特征（复杂度评估错误）
- A/B 测试分组（部分用户被分配到低 budget 组）
- 区域/IP 段限制（某些区域默认较低 budget）
→ 动态分配 reasoning budget = 516 tokens
→ 模型推理被强制截断
→ 推理不完整，返回错误答案
```

**类比**：类似于 Claude 的 `max_tokens` 参数，OpenAI 可能有一个内部的 `max_reasoning_tokens` 参数，在某些情况下被设置为 516。

### 推测 2: OpenAI 负载均衡或模型版本差异

**可能性**：
- OpenAI 可能部署了多个 GPT-5.5 版本/副本
- 部分节点可能是"精简版"（reasoning budget 受限）
- 负载均衡器根据后端压力分配到不同版本
- 516 可能是精简版的硬编码上限
- 重试成功是因为被路由到了"完整版"节点

### ~~推测 3: 第三方代理风控策略~~（已排除）

**⚠️ 此推测已被新证据推翻**：官方客户端也会出现 516，说明不是简单的"代理检测"导致。

### 推测 4: CE 侧 SSE 解析时序问题（仍需验证）

**潜在风险点**：

1. **Usage 事件丢失**：
   ```rust
   // openai_responses.rs:235
   Ok(GatewayByteStreamItem::Eof) => {
       self.drain_sidecar_with_deadline(OPENAI_RESPONSES_SIDECAR_DRAIN_TIMEOUT);
       // ⚠️ 仅等待 50ms，可能 Usage 事件还在解析
   }
   ```
   如果最后的 `usage` 事件尚未被 sidecar 线程解析完成，可能导致 `reasoning_output_tokens` 未更新。

2. **竞态条件**：
   ```rust
   // 主线程读取完 raw_upstream EOF
   // 但 sidecar 线程的 eventsource 解析器可能还有缓冲数据未处理
   const OPENAI_RESPONSES_SIDECAR_DRAIN_TIMEOUT: Duration = Duration::from_millis(50);
   ```
   50ms 对于高延迟网络可能不够。

3. **SSE 帧解析差异**：
   CE 使用 `eventsource-stream` crate，而 Codex CLI 内部可能有优化的原生解析器。不同的解析器对 SSE 边界情况（如最后一个事件没有换行符）的处理可能不同。

### 推测 3: 请求参数传递问题

**需要验证的点**：

1. **模型参数**：
   CE 是否正确透传了 `model_reasoning_effort` 参数？
   ```rust
   // 查看 CE 代码中是否有类似逻辑
   if args.modelReasoningEffort {
       commandArgs.push("--config", `model_reasoning_effort="${args.modelReasoningEffort}"`);
   }
   ```

2. **请求头差异**：
   Codex CLI 可能添加了特殊的请求头（如 `X-Codex-Client-Version`），而 CE 代理未保留。

## 验证方案

### 方案 A: 抓包对比

1. **Codex CLI 直连**：
   ```bash
   # 使用 mitmproxy 拦截
   export HTTPS_PROXY=http://localhost:8080
   codex exec "解决糖果问题" --model gpt-5.5
   ```

2. **CE 代理请求**：
   ```bash
   # 同样配置 mitmproxy
   curl -X POST https://ce-gateway/v1/chat/completions \
     -H "Authorization: Bearer sk-xxx" \
     -d '{"model":"gpt-5.5","messages":[...]}'
   ```

3. **对比内容**：
   - 请求头（User-Agent, X-* 自定义头）
   - 请求体（是否有隐藏参数）
   - 响应头（X-Request-ID, CF-Ray）
   - 响应体（reasoning_tokens 字段出现位置和值）

### 方案 B: 日志增强

在 CE 代码中添加详细日志：

```rust
// crates/service/src/gateway/observability/http_bridge/stream_readers/openai_responses.rs
fn update_usage_from_event(&mut self, event: OpenAIResponsesEvent) {
    if let Ok(mut collector) = self.usage_collector.lock() {
        // 🔍 添加日志
        if event.usage.reasoning_output_tokens.is_some() {
            tracing::info!(
                "📊 Reasoning tokens event: value={:?}, event_type={:?}",
                event.usage.reasoning_output_tokens,
                event.event_type
            );
        }
        event.merge_usage_into(&mut collector.usage, &mut self.usage_text_state);
    }
}
```

### 方案 C: 模拟 CLI 行为

修改 CE 代码，尽可能模拟 Codex CLI 的请求特征：

```rust
// 在发送请求前添加
headers.insert("User-Agent", "openai-codex-cli/1.0");
headers.insert("X-Codex-Client", "official");
// 移除可能暴露代理身份的头
headers.remove("X-Forwarded-For");
headers.remove("Via");
```

## 临时缓解措施

### 措施 1: 检测 516 并重试

```rust
// 在 CE 的响应处理逻辑中
if result.usage.reasoning_output_tokens == Some(516) {
    tracing::warn!("⚠️ Detected reasoning_tokens=516, possible degradation");
    // 可选：自动重试或标记警告
}
```

### 措施 2: 增加 Sidecar 等待时间

```rust
// openai_responses.rs:19
const OPENAI_RESPONSES_SIDECAR_DRAIN_TIMEOUT: Duration = Duration::from_millis(200); // 50 → 200
```

### 措施 3: 用户提示

在 CE 的错误响应中添加：
```json
{
  "error": {
    "message": "推理 token 异常 (516)，可能触发了上游风控，请重试或使用官方客户端",
    "code": "reasoning_token_anomaly",
    "type": "upstream_degradation"
  }
}
```

## 下一步行动

### 优先级 P0（立即执行）

1. ✅ **抓包对比** Codex CLI vs CE 代理的完整请求/响应
2. ✅ **复现问题** 在 CE 环境中触发 516 现象
3. ✅ **日志增强** 追踪 reasoning_tokens 的解析流程

### 优先级 P1（本周完成）

4. **代码审计** CE 的 SSE 解析和 Usage 聚合逻辑
5. **时序分析** 确认是否存在 sidecar drain 超时问题
6. **参数验证** 确保所有 OpenAI 参数正确透传

### 优先级 P2（持续优化）

7. **风控对抗** 研究如何让代理请求更像"官方客户端"
8. **监控告警** 在 Prometheus 中添加 `reasoning_tokens=516` 的指标
9. **文档完善** 将此问题加入已知限制和 FAQ

## 参考资料

- [GitHub Discussion #3937](https://github.com/router-for-me/CLIProxyAPI/discussions/3937)
- [OpenAI Codex SDK - TypeScript](https://github.com/openai/codex/tree/main/sdk/typescript)
- [Eventsource Stream Crate](https://crates.io/crates/eventsource-stream)
- [SSE (Server-Sent Events) RFC](https://html.spec.whatwg.org/multipage/server-sent-events.html)

---

**结论**: 516 问题很可能是 **OpenAI 侧风控策略** 导致的推理预算限制，而不是 CE 代码 bug。但 CE 的 SSE 解析逻辑仍需优化，确保在极端情况下不会丢失 Usage 事件。
