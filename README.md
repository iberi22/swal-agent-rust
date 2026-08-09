# swal-agent-rust

> **Hermes harness clone in Rust — ultra-efficient agent runtime.**
> One Rust codebase → native CLI/TUI + WASM PWA (replaces `swal-agent-runner`).
> Reuses gestalt, synapse-agentic and xavier. Nothing replicated, everything composed.

## Why

Hermes (Python) uses ~225–345 MB per process instance (3 processes ≈ 820 MB + node LSP).
A native Rust harness targets 15–40 MB/instance with 5x less memory and 13–43x better
throughput than Python agent frameworks (AutoAgents/Rig benchmarks, 2026), ms cold start.

## What it is

A conversational agent loop (LLM → tool_calls → execute → feedback) with:
- **Tools**: terminal, file, git, web, memory, cron, subagents, MCP client
- **Skills**: 2-layer cache + file loader (same patterns as Hermes skills)
- **Sessions**: SQLite (native) / IndexedDB (web), persisted
- **Memory**: xavier via HTTP/MCP (never embedded)
- **Gateway**: HTTP/WS + MCP server exposing the loop (Telegram/Discord adapters later)
- **Web**: same core compiled to wasm32, PWA dashboard (Leptos), WebLLM in-browser
- **Parallelism**: Tokio for I/O, Rayon for CPU-bound work (separate pools)

## Canonical docs

| Doc | Purpose |
|-----|---------|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Crate layout, data flow, runtime decisions |
| [REUSE-MAP.md](docs/REUSE-MAP.md) | Hermes feature → existing piece mapping |
| [PLAN.md](docs/PLAN.md) | Waves 0–3, done-criteria, % progress |

## Reused projects (no fork, path-deps only)

| Project | What we take |
|---------|--------------|
| `gestalt` (Rust) | ToolRegistry (shell/read/write/git), skills 2-layer cache, SQLite+DashMap state, worktree router, CLI/REPL |
| `synapse-agentic` (Rust) | Hive/EventBus actors, LLM providers (OpenRouter/DeepSeek/Gemini/Grok), MCP client, compaction |
| `xavier` (Rust) | RAG memory, sessions, code graph — via HTTP/MCP, as a client |

## Status

- [x] Wave 0 — architecture unification & docs (this repo)
- [ ] Wave 1 — native core (~55%)
- [ ] Wave 2 — services (~30%)
- [ ] Wave 3 — WASM/PWA (~15%)
