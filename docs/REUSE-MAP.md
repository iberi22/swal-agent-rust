# Reuse Map — Hermes harness feature → existing piece

> Rule: consume via path-dep (crates) or HTTP (services). Never copy code.
> Source of truth for what is NEW vs REUSED. Verified 2026-08-08.

## Native (waves 1–2)

| Hermes feature | Mapping | Action |
|----------------|---------|--------|
| REPL / CLI driver | gestalt `gestalt_cli` (REPL, run, serve) | **REUSE** as-is |
| Tool dispatch (shell/read/write/git/scan) | gestalt `gestalt_core::application::agent::tools` (`ToolRegistry`) | **REUSE** as-is |
| Skills (load/select) | gestalt skills.rs 2-layer cache (LRU + snapshot) | **REUSE**; file loader is NEW (~100 LOC) |
| Event bus / actor runtime | synapse-agentic `framework` (Hive, EventBus) | **REUSE** |
| LLM providers | synapse-agentic providers (OpenRouter/DeepSeek/Gemini/Grok) | **REUSE** |
| Compaction / context trimming | synapse-agentic `compaction` | **REUSE** |
| RAG memory / sessions | xavier HTTP/MCP server | **REUSE** (client call, never embed) |
| Message/session store | NEW schema on gestalt `gestalt-state` (SQLite + DashMap) | **NEW** (schema only) |
| Conversational loop (LLM→tools→feedback) | — | **NEW** (`swal-loop`, the core) |
| Cron scheduler | — | **NEW** (tokio cron loop, ~100 LOC) |
| Gateway adapters (Telegram/Discord) | — | **NEW** thin webhook→EventBus shims |
| Subagent isolation | gestalt `gestalt-router` worktrees + synapse actors | **REUSE** isolation; per-agent ctx NEW |

## Don't reuse (over-engineered for a minimal clone)

- synapse-agentic full memory stores (SurrealDB/Postgres/pgvector) — xavier covers memory
- synapse-agentic MCP *serve* — Hermes role is MCP client
- gestalt heavier routing if it pulls workspace management — keep only what the loop needs
- maloca WASM logic — different concern
- swal-agent-runner TS codebase — replaced by this project (PWA port)

## Path-deps graph

```
swal-agent (bin)
  ├── swal-loop          (new)
  ├── swal-gateway       (new)
  ├── swal-sched         (new)
  └── gestalt (path)     gestalt_cli, gestalt-state, gestalt_core
swal-loop
  ├── synapse-agentic (path)
  ├── gestalt_core (path)      ToolRegistry + skills
  └── xavier (HTTP client, no dep)
```

Web adds: swal-core (loop, wasm-clean), swal-tools-web, swal-store, swal-sync.
