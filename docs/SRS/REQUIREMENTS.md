# Requirements — swal-agent-rust

> Canonical requirements (REQ-NNN) ↔ user stories (US-NNN) ↔ features (`.gitcore/features.json`).
> IEEE 830-style. Verified 2026-08-09.

## REQ-001: Workspace foundation
El workspace debe compilar con `gestalt_core` y `synapse-agentic` como dependencias git, y un pipeline CI (fmt, clippy, test) debe verificar cada PR.
**Features:** feat-workspace-foundation · **Stories:** US-001

## REQ-002: Tool abstraction (swal-core)
`swal-core` debe exponer un trait `Tool` (nombre, descripción, schema JSON-schema de entrada, `execute` async → `ToolResult`) y un `ToolRegistry` (DashMap) con register/list/execute. Debe ser wasm32-clean: sin tokio, fs ni process.
**Features:** feat-tool-registry · **Stories:** US-002

## REQ-003: Session store (swal-store)
`swal-store` debe proveer un trait `Store` (CRUD de sesiones) y una implementación `SessionStore` sobre rusqlite (bundled, WAL, `data/swal-agent.db`) con schema `sessions(id, created_at, updated_at, summary)` y `messages(id, session_id, role, content, ts)`.
**Features:** feat-session-store · **Stories:** US-003

## REQ-004: Agent loop (swal-loop)
`swal-loop` debe implementar el ciclo conversacional prompt → LLM → tool_calls → execute → feedback, iterando hasta respuesta final o máximo de pasos.
**Features:** feat-agent-loop · **Stories:** US-004

## REQ-005: LLM providers
Debe existir un trait `Provider` async con `MockProvider` (determinista, sin red) y adaptadores a los providers de synapse-agentic (OpenRouter/DeepSeek/Gemini/Grok) vía path-dep.
**Features:** feat-llm-providers · **Stories:** US-005

## REQ-006: Skills loader
El loop debe cargar skills desde filesystem con cache de 2 capas (snapshot en memoria + LRU), sin releer disco en cada turno.
**Features:** feat-skills-loader · **Stories:** US-006

## REQ-007: CLI run + config
`swal-agent run "<task>"` debe ejecutar el loop end-to-end leyendo configuración (modelo, provider, session dir) de archivo/env con defaults, y persistir la sesión.
**Features:** feat-cli-run · **Stories:** US-007

## REQ-008: Native tools reuse
Las tools shell/read/write/git de `gestalt_core` (ToolRegistry) deben registrarse en el loop por path-dep, sin copiar código.
**Features:** feat-native-tools · **Stories:** US-008

## REQ-009: Gateway HTTP/WS + MCP
`swal-gateway` debe exponer el loop vía HTTP, WebSocket y MCP server; un cliente MCP externo debe poder completar una tarea.
**Features:** feat-gateway · **Stories:** US-009

## REQ-010: MCP client
El loop debe actuar como cliente MCP (rmcp) consumiendo tools de servidores externos (incluido xavier). Hermes es cliente MCP, no servidor (el gateway lo expone).
**Features:** feat-mcp-client · **Stories:** US-010

## REQ-011: Cron scheduler
`swal-sched` debe disparar prompts del loop en horarios/intervalos vía timers tokio (sin cron de sistema).
**Features:** feat-cron-scheduler · **Stories:** US-011

## REQ-012: Subagents aislados
Debe poder lanzarse un subagente como instancia aislada del loop (gestalt-router worktrees en native, WebWorkers en web) con el mismo registry.
**Features:** feat-subagents · **Stories:** US-012

## REQ-013: Compaction
El loop debe compactar contextos largos reutilizando el módulo compaction de synapse-agentic.
**Features:** feat-compaction · **Stories:** US-013

## REQ-014: Xavier memory client
El loop debe hacer round-trip store/search contra xavier vía HTTP/MCP client, sin embeber el motor.
**Features:** feat-xavier-memory · **Stories:** US-014

## REQ-015: wasm32 core
`swal-core` (loop + tools trait) debe compilar a wasm32 sin tokio/fs/process, con async vía wasm-bindgen-futures.
**Features:** feat-wasm-core · **Stories:** US-015

## REQ-016: Web tools
`swal-tools-web` debe implementar la interfaz de tools en el navegador: fs→OPFS, git→isomorphic-git (JS interop), shell→WebContainers bridge. Prohibido subprocess en web.
**Features:** feat-tools-web · **Stories:** US-016

## REQ-017: IndexedDB store
`swal-store` debe tener un backend IndexedDB (rexie) con el mismo schema serde compartido que el backend nativo.
**Features:** feat-store-web · **Stories:** US-017

## REQ-018: PWA Leptos
Debe existir una PWA Leptos (Rust → WASM) reutilizando tipos de `swal-core`, con el loop en un Comlink worker y WebLLM en-browser (o remote OpenAI-compatible).
**Features:** feat-pwa-leptos · **Stories:** US-018

## REQ-019: CRDT sync
`swal-sync` debe sincronizar estado vía SyncEngine detrás de transport trait (native→EdgeMesh TCP/QUIC; web→WS/WebRTC relay) con la misma lógica CRDT/merge.
**Features:** feat-sync-crdt · **Stories:** US-019
