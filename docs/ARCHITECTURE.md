# Architecture

> swal-agent-rust — Hermes clone, minimalist, Rust-native.
> Design authority: Kimi k3 (2026-08-08) + web research (AutoAgents/Rig benchmarks, rmcp, wasm-bindgen, PostHog tokio+rayon).

## Principles

1. **Reuse, don't replicate** — every piece that exists in gestalt/synapse-agentic/xavier is consumed via path-deps or HTTP. New code is only what doesn't exist.
2. **Minimalism** — 4 native crates + platform shims. No gold-plating. If a crate can be 100 LOC, it is.
3. **One loop, two targets** — the agent loop is platform-agnostic; native and web are thin shells.
4. **Tokio for I/O, Rayon for CPU** — separate pools. Rayon gets full core count, Tokio ~half. CPU-bound work goes through `rayon::spawn` + `tokio::sync::oneshot` (PostHog pattern: 2s → 94ms). Never call `par_iter` inside a Tokio worker thread directly — wrap in `spawn_blocking`.

## Native crates (waves 1–2)

```
┌──────────────────────────────────────────────────────────┐
│ swal-agent (bin) — TUI/CLI entry                         │
│   wires gestalt SQLite state, skills cache, starts loop  │
├──────────────────────────────────────────────────────────┤
│ swal-loop (lib) — conversational agent loop              │
│   prompt → LLM → tool_calls → execute → feed back        │
│   drives synapse Hive actors + gestalt ToolRegistry      │
│   compaction via synapse-agentic                         │
├──────────────────────────────────────────────────────────┤
│ swal-gateway (lib) — HTTP/WS + MCP server                │
│   exposes the loop remotely; MCP client to xavier        │
├──────────────────────────────────────────────────────────┤
│ swal-sched (lib) — cron ticks + subagent spawn           │
│   subagent = isolated loop instance, same registry       │
│   tokio timers; rayon for CPU work                       │
└──────────────────────────────────────────────────────────┘
```

**Data flow:** TUI → loop → synapse provider (LLM) → tool result (gestalt/xavier) → loop → session persisted (SQLite); gateway mirrors TUI; sched injects cron prompts and spawns subagent loops.

## Web crates (wave 3)

```
swal-core (wasm32)          — agent loop + tools trait, platform-agnostic,
                              no tokio/fs/process; async via wasm-bindgen-futures
swal-tools-native (native)  — shell, fs, git via git2
swal-tools-web (wasm32)     — git→isomorphic-git (JS interop), fs→OPFS,
                              shell→WebContainers bridge; NO subprocess tool
swal-store (lib)            — Store trait + backends: rusqlite (native),
                              rexie/IndexedDB (web); shared serde schema
swal-sync (wasm-clean)      — SyncEngine behind transport trait:
                              native→EdgeMesh TCP/QUIC; web→WS/WebRTC to relay;
                              same CRDT/merge logic
```

**PWA UI:** Leptos (single Rust codebase → WASM) reusing `swal-core` types.
Comlink worker runs the agent loop + WebLLM off the main thread.
WebLLM (MLC-LLM compiled to WASM + WebGPU) provides in-browser inference —
same OpenAI-style API, so `swal-core` LLM provider is unchanged.

## Explicit non-goals (do not build)

- No new memory engine — xavier via HTTP/MCP only
- No MCP *server* in the loop — Hermes role is MCP *client* (gateway exposes it, but loop consumes)
- No mesh/P2P logic — edge-mesh exists
- No maloca WASM — out of scope
- No full agent routing/worktree orchestration — gestalt-router already does that

## Parallelism spec (Tokio + Rayon)

| Workload | Runtime | Notes |
|----------|---------|-------|
| LLM HTTP calls, WS, MCP, file I/O, timers | Tokio (worker threads ~ cores/2) | never block in `.await` |
| JSON parsing, BM25/similarity, embedding, diff, regex sweeps | Rayon (full cores) | `rayon::spawn` → `oneshot` back to Tokio |
| Subagent isolation | gestalt worktree router (native) | wasm: WebWorkers |

Bound Rayon concurrency with a semaphore when tasks are many and small
(to avoid starving Tokio blocking pool).
