# Requirements — swal-agent-rust

> Canonical requirements (REQ-NNN) ↔ user stories (US-NNN) ↔ features (`.gitcore/features.json`).
> 37 features = 100% scope. Verified 2026-08-09.

## REQ-001: Git deps (gestalt_core, synapse-agentic)
Root workspace.dependencies with git deps to gestalt workspace members.
**Features:** feat-git-deps · **Stories:** US-001 · **Wave:** W1

## REQ-002: CI pipeline (fmt+clippy+test)
GitHub Actions: cargo fmt --check, clippy -D warnings, test --workspace on push/PR.
**Features:** feat-ci-pipeline · **Stories:** US-002 · **Wave:** W1

## REQ-003: Tool trait (wasm32-clean)
Platform-agnostic Tool trait: name, description, JSON-schema input, async execute. No tokio/fs/process.
**Features:** feat-tool-trait · **Stories:** US-003 · **Wave:** W1

## REQ-004: ToolRegistry (DashMap)
Register/list/execute tools via DashMap-backed registry in swal-core.
**Features:** feat-tool-registry · **Stories:** US-004 · **Wave:** W1

## REQ-005: Store trait (session CRUD)
Backend-agnostic Store trait: create/append/read/list/delete sessions.
**Features:** feat-store-trait · **Stories:** US-005 · **Wave:** W1

## REQ-006: SessionStore SQLite (rusqlite WAL)
SQLite backend: sessions+messages schema, WAL, bundled rusqlite, data/swal-agent.db.
**Features:** feat-session-sqlite · **Stories:** US-006 · **Wave:** W1

## REQ-007: LLM Provider trait (async)
Async Provider trait (Message list → ProviderResponse w/ tool_calls) + ProviderError.
**Features:** feat-provider-trait · **Stories:** US-007 · **Wave:** W1

## REQ-008: MockProvider (deterministic)
Scripted deterministic provider for tests: canned responses incl. tool_calls. No network.
**Features:** feat-mock-provider · **Stories:** US-008 · **Wave:** W1

## REQ-009: Skills file loader (frontmatter)
Load SKILL.md from filesystem, parse name/description frontmatter, body as content.
**Features:** feat-skills-loader · **Stories:** US-009 · **Wave:** W1

## REQ-010: Skills 2-layer cache (LRU+snapshot)
Snapshot in memory + LRU hot cache; no re-read disk per turn.
**Features:** feat-skills-cache · **Stories:** US-010 · **Wave:** W1

## REQ-011: AgentLoop (LLM→tools→feedback)
Conversational loop: prompt→LLM→tool_calls→execute→feedback until final or max_steps.
**Features:** feat-agent-loop · **Stories:** US-011 · **Wave:** W1

## REQ-012: CLI run + config (env/file)
swal-agent run "task" with config (model/provider/session dir) from CLI>env>defaults.
**Features:** feat-cli-run · **Stories:** US-012 · **Wave:** W1

## REQ-013: Native tools wiring + session persist
gestalt shell/read/write/git tools via adapter + session persistence wiring in CLI.
**Features:** feat-native-tools · **Stories:** US-013 · **Wave:** W1

## REQ-014: Gateway HTTP server (axum)
REST: GET /health, POST /run → runs loop, returns content+steps.
**Features:** feat-gateway-http · **Stories:** US-014 · **Wave:** W2

## REQ-015: Gateway WebSocket server
WS: client sends task → streams result via AgentHandle.
**Features:** feat-gateway-ws · **Stories:** US-015 · **Wave:** W2

## REQ-016: Gateway MCP server (routes)
MCP-style endpoints (tools/list, tools/call) exposing loop to MCP clients.
**Features:** feat-gateway-mcp-server · **Stories:** US-016 · **Wave:** W2

## REQ-017: MCP client (JSON-RPC/rmcp)
Loop as MCP client: connect, initialize, list_tools, call_tool.
**Features:** feat-mcp-client · **Stories:** US-017 · **Wave:** W2

## REQ-018: MCP tools in loop (external)
External MCP server tools callable from AgentLoop via client.
**Features:** feat-mcp-tools · **Stories:** US-018 · **Wave:** W2

## REQ-019: Cron ticker (tokio timers)
Scheduler: interval-based task firing via tokio::time.
**Features:** feat-cron-ticker · **Stories:** US-019 · **Wave:** W2

## REQ-020: Scheduled tasks (RunTask)
ScheduledTask list + RunTask trait decoupling scheduler from loop.
**Features:** feat-cron-tasks · **Stories:** US-020 · **Wave:** W2

## REQ-021: Subagent spawner (isolated)
Spawn isolated loop instances; semaphore-bounded concurrency.
**Features:** feat-subagent-spawn · **Stories:** US-021 · **Wave:** W2

## REQ-022: Subagent isolation (worktree)
Native isolation via gestalt-router worktrees; web via WebWorkers.
**Features:** feat-subagent-isolation · **Stories:** US-022 · **Wave:** W2

## REQ-023: Compaction (synapse reuse)
Compact long contexts via synapse-agentic compaction; should_compact heuristic.
**Features:** feat-compaction · **Stories:** US-023 · **Wave:** W2

## REQ-024: Xavier memory store (HTTP client)
POST /v1/memories with X-Xavier-Token; XavierTransport trait.
**Features:** feat-xavier-store · **Stories:** US-024 · **Wave:** W2

## REQ-025: Xavier memory search (round-trip)
POST /v1/memories/search; store→search round-trip test.
**Features:** feat-xavier-search · **Stories:** US-025 · **Wave:** W2

## REQ-026: swal-core loop on wasm32
Agent loop compiles to wasm32 without tokio/fs/process; async via wasm-bindgen-futures.
**Features:** feat-wasm-core-loop · **Stories:** US-026 · **Wave:** W3

## REQ-027: Tool trait on wasm32
Platform-agnostic Tool trait verified building for wasm32 target.
**Features:** feat-wasm-core-tools · **Stories:** US-027 · **Wave:** W3

## REQ-028: Web fs tools (OPFS)
File tools backed by Origin Private File System (JS interop).
**Features:** feat-tools-opfs · **Stories:** US-028 · **Wave:** W3

## REQ-029: Web git tools (isomorphic-git)
Git tools via isomorphic-git JS interop (no git2 in browser).
**Features:** feat-tools-git-web · **Stories:** US-029 · **Wave:** W3

## REQ-030: Web shell (WebContainers)
Shell tool via WebContainers bridge; NO subprocess in web.
**Features:** feat-tools-shell-web · **Stories:** US-030 · **Wave:** W3

## REQ-031: IndexedDB store backend (rexie)
Store trait impl on IndexedDB via rexie; same serde schema.
**Features:** feat-store-indexeddb · **Stories:** US-031 · **Wave:** W3

## REQ-032: Shared serde schema (native+web)
One session/message schema shared by rusqlite and IndexedDB backends.
**Features:** feat-store-schema-shared · **Stories:** US-032 · **Wave:** W3

## REQ-033: Leptos PWA dashboard
Leptos PWA reusing swal-core types; single Rust codebase → WASM.
**Features:** feat-pwa-leptos · **Stories:** US-033 · **Wave:** W3

## REQ-034: Comlink worker (loop off main)
Agent loop + WebLLM run in Comlink worker off the main thread.
**Features:** feat-pwa-comlink · **Stories:** US-034 · **Wave:** W3

## REQ-035: WebLLM in-browser inference
WebLLM (MLC-LLM→WASM+WebGPU) OpenAI-style API; remote fallback.
**Features:** feat-pwa-webllm · **Stories:** US-035 · **Wave:** W3

## REQ-036: CRDT sync engine
SyncEngine with same CRDT/merge logic across native and web.
**Features:** feat-sync-crdt · **Stories:** US-036 · **Wave:** W3

## REQ-037: Sync transport (EdgeMesh/WS)
Transport trait: native→EdgeMesh TCP/QUIC; web→WS/WebRTC relay to xavier.
**Features:** feat-sync-transport · **Stories:** US-037 · **Wave:** W3
