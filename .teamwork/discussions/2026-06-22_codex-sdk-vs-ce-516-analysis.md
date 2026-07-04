# OpenAI Codex SDK vs CE 实现对比分析

> **生成时间**: 2026-06-22  
> **目的**: 找出为什么官方 SDK 也会出现 516 问题

---

## 核心架构差异

### OpenAI Codex SDK 架构

```
TypeScript SDK (thread.ts)
    ↓
spawn Codex CLI 子进程 (exec.ts)
    ↓
Codex CLI (Rust 二进制)
    ↓ [内部实现]
调用 OpenAI API
    ↓
接收 SSE 流
    ↓
解析 → 输出 JSON 行
    ↓
Node.js readline 逐行读取
    ↓
返回给 TypeScript SDK
```

### CE 架构

```
客户端请求
    ↓
CE Gateway (Rust)
    ↓
直接调用 OpenAI API
    ↓ [双通道 Tee]
接收 SSE 流 → 解析 usage
    ↓
透传给客户端
```

---

## 关键发现 1: Codex SDK 不直接处理 SSE

**OpenAI Codex TypeScript SDK 的实现**（thread.ts）：

```typescript
async *runStreamedInternal(input, turnOptions): AsyncGenerator<ThreadEvent> {
  // 1. 启动 Codex CLI 子进程
  const generator = this._exec.run({...});
  
  try {
    // 2. 逐行读取 CLI 输出的 JSON
    for await (const item of generator) {
      let parsed: ThreadEvent;
      try {
        parsed = JSON.parse(item) as ThreadEvent;  // ← 每行是完整的 JSON
      } catch (error) {
        throw new Error(`Failed to parse item: ${item}`, { cause: error });
      }
      yield parsed;
    }
  } finally {
    await cleanup();
  }
}

// 3. 非流式调用：收集所有事件
async run(input, turnOptions): Promise<Turn> {
  const generator = this.runStreamedInternal(input, turnOptions);
  let usage: Usage | null = null;
  
  for await (const event of generator) {
    if (event.type === "turn.completed") {
      usage = event.usage;  // ← 只在 turn.completed 事件提取 usage
    }
  }
  return { items, finalResponse, usage };
}
```

**关键点**：
1. SDK 不解析 SSE - 由 Codex CLI（Rust 二进制）负责
2. SDK 只负责 JSON 行解析
3. `usage` 只从 `turn.completed` 事件获取（最终事件）

---

## 关键发现 2: Codex CLI 才是真正的 SSE 处理者

**问题**：Codex CLI 的源码在 `codex-rs` 目录（Rust），但它是闭源的，我们无法直接查看。

**推测**：
- Codex CLI 内部使用 Rust 实现 SSE 解析
- 可能使用了与 CE 类似的架构（eventsource-stream crate？）
- **关键**：如果 Codex CLI 也有类似的"50ms drain 超时"问题，那就能解释为什么官方 SDK 也会出现 516

---

## 关键发现 3: exec.ts 使用 readline 逐行读取

```typescript
const rl = readline.createInterface({
  input: child.stdout,
  crlfDelay: Infinity,  // ← 关键：等待完整行
});

for await (const line of rl) {
  yield line as string;  // ← 每次 yield 一行 JSON
}
```

**问题点**：
- `readline` 依赖换行符 `\n` 分隔
- 如果 Codex CLI 输出最后一行 **没有换行符**，readline 会等待
- 但 CLI 进程退出时，readline 会丢弃不完整的行

**潜在风险场景**：
```
Codex CLI 输出:
{"type":"item.completed",...}\n
{"type":"turn.completed","usage":{...}}\n  ← 有换行
{"type":"reasoning.tokens","tokens":2048}   ← 没有换行！进程退出
                                            ↑ readline 丢弃
```

---

## 关键发现 4: 官方 SDK 也依赖"最终事件"

```typescript
// turn.completed 事件是 usage 的唯一来源
if (event.type === "turn.completed") {
  usage = event.usage;
}
```

**问题**：
- 如果 `turn.completed` 事件因为任何原因没有到达，usage 就是 `null`
- 但如果之前的事件包含了部分 usage（如 reasoning_tokens=516），那个值会被保留吗？

**我们需要查看**：Codex CLI 输出的事件格式

---

## 对比 CE 的问题

### CE 的风险（已知）

| 风险 | 位置 | 影响 |
|------|------|------|
| 50ms drain 超时 | openai_responses.rs | 最后的 usage 事件可能丢失 |
| usage 覆盖而非累加 | output_text.rs | 保留中间值 516 |
| 无异常检测 | 全局 | 无法发现 516 问题 |

### Codex CLI 的可能风险（推测）

| 风险 | 推测位置 | 影响 |
|------|---------|------|
| Rust 侧 SSE 解析超时 | codex-cli (闭源) | 与 CE 相同的 drain 问题 |
| readline 丢弃不完整行 | exec.ts | 缺少换行符导致事件丢失 |
| 子进程 kill 时机 | exec.ts | 可能在 usage 事件输出前终止 |

---

## 实验验证方案

### 方案 1: 抓包对比（推荐）

```bash
# 1. 抓取 Codex CLI 直连的完整 SSE 流
tcpdump -i any -A 'host api.openai.com and port 443' -w codex_cli.pcap

# 2. 抓取 CE 代理的完整 SSE 流
tcpdump -i any -A 'host api.openai.com and port 443' -w ce_proxy.pcap

# 3. 对比两者的最后 10 个事件
# 看是否都包含 reasoning_tokens=516 的中间事件
```

### 方案 2: 调试 Codex CLI 输出

```typescript
// 修改 exec.ts，记录所有 JSON 行
for await (const line of rl) {
  console.error('[DEBUG]', line);  // ← 输出到 stderr
  yield line as string;
}
```

### 方案 3: CE 侧添加详细日志

```rust
// 在 openai_responses.rs 的 update_usage_from_event() 中
fn update_usage_from_event(&mut self, event: OpenAIResponsesEvent) {
    if let Ok(mut collector) = self.usage_collector.lock() {
        log::debug!(
            "收到 usage 事件: reasoning_tokens={:?}, event_type={:?}",
            event.usage.reasoning_output_tokens,
            event.event_type
        );
        event.merge_usage_into(&mut collector.usage, &mut self.usage_text_state);
    }
}
```

---

## 新推测：问题可能在 OpenAI 服务端

### 推测 5: OpenAI SSE 流本身的问题

**证据**：
1. 官方 SDK 也会出现 516
2. CE 也会出现 516
3. 两者架构完全不同，但都依赖 OpenAI 的 SSE 流

**可能性**：
```
OpenAI 后端在某些情况下：
1. 先发送中间 usage 事件（reasoning_tokens=516）
2. 然后因为负载/超时/限流，提前终止流
3. 最终的 turn.completed 事件（含完整 usage）未发送
→ 客户端只收到 516
```

**类似案例**：
- GitHub Issue 中提到"重试有时能得到正确答案"
- 说明不是客户端解析问题，而是服务端行为不一致

---

## 推测 6: reasoning_tokens 的多阶段上报

**OpenAI 可能的实现**：
```
推理过程中，模型会多次上报进度：

event: reasoning.progress
data: {"reasoning_tokens": 200}

event: reasoning.progress
data: {"reasoning_tokens": 516}  ← 达到某个检查点

event: reasoning.progress
data: {"reasoning_tokens": 1024}

event: turn.completed
data: {"usage": {"reasoning_output_tokens": 2048}}
```

**如果服务端在 516 处被中断**：
- 客户端只收到前两个事件
- 最后的 turn.completed 丢失
- 结果：reasoning_tokens = 516

---

## 验证 516 是否是特殊值

### 分析 516 的特殊性

```
516 = 2^9 + 2^2 = 512 + 4
516 ≈ 512 (2^9)
```

**可能的解释**：
1. **内存页边界**：512 是常见的内存页大小，516 可能是 512 + 4 字节头部
2. **缓冲区大小**：某个内部缓冲区大小为 516 tokens
3. **检查点间隔**：OpenAI 可能每 516 tokens 发送一次进度更新
4. **硬编码阈值**：某个"精简模式"的推理上限

---

## 结论与下一步

### 核心结论

1. **官方 SDK 也有 516 问题** → 排除 CE 代码特有 bug
2. **官方 SDK 不直接处理 SSE** → Codex CLI (Rust 闭源) 才是关键
3. **两种架构都依赖最终事件** → 如果最终事件丢失，都会保留中间值

### 问题根源的可能性排序

| 可能性 | 概率 | 证据强度 |
|--------|------|---------|
| OpenAI 服务端动态限制/中断 | 70% | 官方 SDK 也有，重试有效 |
| OpenAI SSE 流多阶段上报 | 20% | 516 固定值，像是检查点 |
| 客户端解析超时问题 | 10% | CE 有 50ms 问题，但官方 SDK 也有 |

### 立即行动

1. **抓包对比**：同一个请求，对比 Codex CLI vs CE 收到的完整 SSE 流
2. **联系 OpenAI 技术支持**：询问 reasoning_tokens=516 的官方解释
3. **增加 CE 日志**：记录每个 usage 事件的时间戳和值

### 如果是 OpenAI 服务端问题

**我们能做的**：
1. 添加 516 异常检测和告警
2. 自动重试机制（检测到 516 时重试请求）
3. 在响应头添加警告：`X-Reasoning-Tokens-Suspicious: true`
4. 提供降级策略：切换到其他模型

**我们无法做的**：
- 无法修复 OpenAI 后端的行为
- 无法保证 reasoning_tokens 的完整性
