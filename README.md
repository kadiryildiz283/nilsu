## 🚀 Nilsu

**LLM Semantic Context Compiler Daemon (Rust, Zero-Graph MVP)**

Nilsu is an ultra-lightweight **context extraction daemon** designed for local LLM agents.
It accepts cursor-based requests coming from Neovim or any other editor and generates **token-optimized context envelopes (Context Envelope)** using minimal AST analysis based on Tree-sitter.

> “From cursor movement to semantic context in <200ms — without LSP, without graph, without overhead.”

## ⚡ Problem

Modern LLM tooling:

* Excessively heavy LSP dependency
* Slow context generation
* Memory + CPU spikes
* Unpredictable latency under concurrency

Result:

> “Intelligent coding tools that are ironically slow.”

---

## 🧠 Solution

Nilsu radically solves this problem:

* ❌ No LSP
* ❌ No Graph
* ❌ No background watcher (V1)
* ❌ No stateful cache

✔ On-demand AST parsing
✔ Tree-sitter boundary extraction
✔ Unix Domain Socket API
✔ Backpressure-controlled async runtime

---

## 🏗 Architecture (V1)

```
Neovim / CLI / Agent
        ↓
Unix Domain Socket (/tmp/nilsu.sock)
        ↓
Tokio Async Server
        ↓
Semaphore (backpressure gate)
        ↓
Request Handler
        ↓
Tree-sitter Parser (on-demand)
        ↓
Context Envelope JSON Response
```

---

## 📡 API Contract

### Request

```json
{
  "action": "get_context",
  "file": "src/main.rs",
  "cursor_line": 42
}
```

### Response

```json
{
  "status": "ok",
  "request_id": "uuid",
  "message": "request received",
  "latency_ms": 87
}
```

---

## 🧪 Performance Guarantee (V1 Target)

| Metric        | Target                    |
| ------------- | ------------------------- |
| P95 latency   | < 200ms                   |
| Concurrency   | 64 req max                |
| Crash rate    | 0 under malformed input   |
| Memory growth | flat under sustained load |

---

## 🧪 Benchmark

```bash
cargo run --release
cargo run --example stress_test
```

---

## 🔥 Design Principles

* **Fail fast, not slow**
* **No hidden state**
* **No background magic**
* **Every request is independent**
* **Backpressure > optimism**

---

## 🧪 Status

> V1: Stable Core (Socket + Concurrency + JSON Contract)
> V2: Tree-sitter semantic expansion
> V3: Context intelligence layer (graph, LSP optional)

---

## ⚠️ Non-Goals

* Code completion engine
* AI model inference
* IDE replacement
* Semantic graph reasoning (V1)

---

## 🧠 Why it matters

Nilsu is not a feature.
It is a **context runtime primitive** for LLM systems.

---

---

# 🧪 Benchmark Harness — 1000 Request Stress Test

This is no longer a "test script", it is a **proof of OSS reliability**.

---

## 🎯 Purpose

Measuring the system's:

* timeout behavior
* latency distribution
* crash resilience

under a load of 1000 concurrent requests.

---

## 🧱 Architecture

```
[Load Generator]
      ↓
1000 concurrent clients
      ↓
Unix Socket (/tmp/nilsu.sock)
      ↓
Nilsu Daemon
      ↓
Response collector
      ↓
Latency analyzer
      ↓
Histogram output
```

---

## ⚙️ Test Parameters

```text
TOTAL_REQUESTS = 1000
CONCURRENCY = 100
TIMEOUT = 200ms
PAYLOAD = random valid JSON requests
```

---

## 🧪 Test Scenarios

### 1. Baseline load

* 1000 valid requests
* uniform distribution

### 2. Burst test

* 100 concurrent spikes every 50ms

### 3. Malformed injection

* 10% invalid JSON mixed in

### 4. Timeout stress

* artificially delayed processing (sleep injection optional toggle)

---

## 📊 Output Metrics

### 1. Latency histogram

```
P50: 45ms
P90: 120ms
P95: 180ms
P99: 240ms
```

### 2. Throughput

```
req/sec: X
```

### 3. Failure rate

```
invalid json handled: OK
timeouts: N
crashes: 0
```

---

## 📈 Visualization (Rust-side optional)

* histogram bins:

  * 0–50ms
  * 50–100ms
  * 100–150ms
  * 150–200ms
  * 200ms+

---

## 🧪 Acceptance Criteria

Benchmark PASS if:

* ❌ no crash
* ❌ no deadlock
* ❌ no memory leak
* ✔ all malformed requests handled
* ✔ p95 < 200ms

---

## 🔥 Why this benchmark matters

This test proves that:

> “Nilsu is not a toy socket server. It is a concurrency-safe runtime primitive.”

---
