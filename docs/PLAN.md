# Plan — Waves to 100%

> Design: Kimi k3 (2026-08-08). Waves ordered by dependency. Done-criteria are verifiable.
> Total = 3 waves (+ Wave 0 already done: this repo's architecture unification).

## Wave 0 — Architecture unification ✅ (done)
- Unified architecture & docs: README, ARCHITECTURE, REUSE-MAP, PLAN
- Workspace skeleton (crates below, empty libs, workspace compiles)
- Reuse map agreed (gestalt / synapse-agentic / xavier roles fixed)

## Wave 1 — Native core (~55%) — 🟢 21% done (foundation/core/store merged)
**Goal:** working CLI agent end-to-end.
**Crates:** swal-loop, swal-agent (CLI only), gestalt reuse (ToolRegistry, skills, SQLite, worktrees), skills file loader, session schema.
**Done-criteria:**
- [x] Workspace git deps + CI (PR #13) — feat-workspace-foundation 100%
- [x] swal-core Tool trait + ToolRegistry wasm32-clean (PR #15) — feat-tool-registry 100%
- [x] swal-store Store trait + SessionStore SQLite WAL (PR #14) — feat-session-store 100%
- [ ] `swal-agent run "task"` executes LLM → tool → feedback loop
- [ ] Tools callable via gestalt ToolRegistry (shell, read, write, git)
- [ ] Sessions persisted in SQLite (gestalt-state)
- [ ] Skills loaded from filesystem (2-layer cache)
- [ ] `cargo test -p swal-loop` green

## Wave 2 — Services (~30%)
**Goal:** remote + scheduled operation.
**Crates:** swal-gateway (HTTP/WS + MCP), swal-sched (cron + subagents), xavier HTTP/MCP client, synapse-agentic reuse (Hive/providers/compaction).
**Done-criteria:**
- [ ] External MCP client completes a task via gateway
- [ ] Cron fires a job on schedule (tokio timers)
- [ ] Subagent runs in isolated worktree (gestalt-router)
- [ ] Compaction triggers on long session (synapse-agentic)
- [ ] Memory: session search + store round-trip via xavier HTTP/MCP

## Wave 3 — WASM/PWA (~15%)
**Goal:** browser agent — replaces swal-agent-runner.
**Crates:** swal-core extraction (loop, wasm-clean), swal-tools-web (OPFS/WebContainers/isomorphic-git), swal-store trait + IndexedDB impl, Leptos PWA + Comlink worker + WebLLM, swal-sync CRDT.
**Done-criteria:**
- [ ] wasm32 target compiles without tokio/fs/process
- [ ] PWA runs a loop on WebLLM (or OpenAI-compatible remote)
- [ ] State syncs to xavier via CRDT (WebSocket/WebRTC → EdgeMesh relay)

## Progress tracking

| Wave | % | Status |
|------|---|--------|
| 0 | — | ✅ |
| 1 | 21 | 🟢 in progress (groups B-F pending) |
| 2 | 0 | ⏳ pending |
| 3 | 0 | ⏳ pending |
| **Total** | **21** | 🟢 |
