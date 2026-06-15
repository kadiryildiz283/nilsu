---

## 🚀 Nilsu

**LLM Semantic Context Compiler Daemon (Rust, Zero-Graph MVP)**

Nilsu, yerel LLM ajanları için tasarlanmış ultra-hafif bir **context extraction daemon**’dır.
Neovim veya herhangi bir editor içinden gelen imleç bazlı istekleri alır ve Tree-sitter tabanlı minimal AST analizleriyle **token-optimize edilmiş bağlam paketleri (Context Envelope)** üretir.

> “From cursor movement to semantic context in <200ms — without LSP, without graph, without overhead.”

---

## ⚡ Problem

Modern LLM tooling:

* Aşırı ağır LSP bağımlılığı
* Yavaş context generation
* Memory + CPU spikes
* Unpredictable latency under concurrency

Sonuç:

> “Intelligent coding tools that are ironically slow.”

---

## 🧠 Solution

Nilsu, bu problemi radikal şekilde çözer:

* ❌ LSP yok
* ❌ Graph yok
* ❌ Background watcher yok (V1)
* ❌ Stateful cache yok

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

Bu artık “test script” değil, **OSS güvenilirlik kanıtı**.

---

## 🎯 Amaç

Sistemin:

* 1000 concurrent request altında
* timeout davranışı
* latency dağılımı
* crash resilience

ölçülmesi.

---

## 🧱 Mimari

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

## ⚙️ Test Parametreleri

```text
TOTAL_REQUESTS = 1000
CONCURRENCY = 100
TIMEOUT = 200ms
PAYLOAD = random valid JSON requests
```

---

## 🧪 Test Senaryoları

### 1. Baseline load

* 1000 valid requests
* uniform distribution

### 2. Burst test

* 100 concurrent spikes every 50ms

### 3. Malformed injection

* %10 invalid JSON mixed

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

Bu test şunu kanıtlar:

> “Nilsu is not a toy socket server. It is a concurrency-safe runtime primitive.”

---
