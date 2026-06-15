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
  "message": "context extracted",
  "context_snippet": {
    "code_snippet": "async fn main() -> Result<(), Box<dyn std::error::Error>> {\n    ...\n}",
    "start_line": 6,
    "end_line": 23
  },
  "latency_ms": 0
}
```

---

## 🔌 Neovim Lua Client (Integration)

You can integrate Nilsu directly into Neovim using its built-in Lua API. The client communicates with the daemon over UDS, extracts the Rust context enclosing your cursor, and displays it in a clean floating window.

Create a file named `nilsu.lua` in your Neovim `lua/` directory (or use the root `nilsu.lua` file provided in this repository):

```lua
local M = {}

local socket_path = "/tmp/nilsu.sock"

function M.get_context()
  local uv = vim.loop or vim.uv
  local client = uv.new_pipe(false)
  
  local file = vim.api.nvim_buf_get_name(0)
  local cursor_line = vim.api.nvim_win_get_cursor(0)[1]

  if file == "" then
    print("[Nilsu] Buffer has no file name")
    return
  end

  local request = vim.json.encode({
    action = "get_context",
    file = file,
    cursor_line = cursor_line
  })

  client:connect(socket_path, function(err)
    if err then
      vim.schedule(function()
        print("[Nilsu] Connection error: " .. tostring(err))
      end)
      return
    end

    client:write(request, function(write_err)
      if write_err then
        vim.schedule(function()
          print("[Nilsu] Write error: " .. tostring(write_err))
        end)
        client:close()
        return
      end
    end)

    local response_chunks = {}
    client:read_start(function(read_err, chunk)
      if read_err then
        vim.schedule(function()
          print("[Nilsu] Read error: " .. tostring(read_err))
        end)
        client:close()
        return
      end

      if chunk then
        table.insert(response_chunks, chunk)
      else
        -- EOF
        client:close()
        local full_response = table.concat(response_chunks)
        vim.schedule(function()
          local ok, decoded = pcall(vim.json.decode, full_response)
          if not ok then
            print("[Nilsu] Failed to decode response: " .. full_response)
            return
          end

          if decoded.status == "ok" and decoded.context_snippet then
            local snippet = decoded.context_snippet
            M.show_floating_window(snippet.code_snippet, snippet.start_line, snippet.end_line)
          else
            print("[Nilsu] No context found or error: " .. (decoded.message or "unknown"))
          end
        end)
      end
    end)
  end)
end

function M.show_floating_window(content, start_line, end_line)
  local lines = vim.split(content, "\n")
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  
  -- Set options for display
  vim.api.nvim_buf_set_option(buf, "filetype", "rust")
  vim.api.nvim_buf_set_option(buf, "bufhidden", "wipe")

  -- Calculate size and position
  local width = math.min(80, vim.o.columns - 4)
  local height = math.min(#lines, vim.o.lines - 4)
  local row = math.floor((vim.o.lines - height) / 2)
  local col = math.floor((vim.o.columns - width) / 2)

  local opts = {
    relative = "editor",
    width = width,
    height = height,
    row = row,
    col = col,
    style = "minimal",
    border = "rounded",
    title = string.format(" Nilsu Context (Lines %d-%d) ", start_line, end_line),
    title_pos = "center",
  }

  local win = vim.api.nvim_open_win(buf, true, opts)
  
  -- Map 'q' to close the window
  vim.api.nvim_buf_set_keymap(buf, 'n', 'q', ':q<CR>', { noremap = true, silent = true })
end

return M
```

To bind this context compilation trigger to a shortcut, add the following to your Neovim configuration (`init.lua`):
```lua
vim.keymap.set('n', '<leader>nc', function() require('nilsu').get_context() end, { desc = "Get Nilsu Context" })
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
