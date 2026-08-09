# User Stories — swal-agent-rust

> Canonical user stories for 100% feature scope (Waves 1–3).
> Format: `US-NNN` ↔ `REQ-NNN` ↔ `feat-*` (see `.gitcore/features.json`).
> Verified 2026-08-09. Each story has verifiable acceptance criteria.

---

## US-001: Workspace con dependencias git y CI
**Como** desarrollador del ecosistema SWAL,
**quiero** un workspace Rust que compile con `gestalt_core` y `synapse-agentic` como dependencias git y un pipeline CI (fmt + clippy + test),
**para** que cada PR quede verificado automáticamente y el esqueleto de crates sirva de base a todas las olas.
**Feature:** feat-workspace-foundation · **REQ:** REQ-001
**Aceptación:**
- [ ] `cargo check --workspace` → 0 errores
- [ ] `.github/workflows/ci.yml` existe con jobs fmt/clippy/test
- [ ] `Cargo.toml` raíz declara `gestalt_core` y `synapse-agentic` vía `git =`
- [ ] `cargo fmt --all --check` pasa

## US-002: Registrar y ejecutar tools de forma uniforme
**Como** agente del loop,
**quiero** un trait `Tool` con nombre, descripción, schema JSON de entrada y ejecución async, más un `ToolRegistry` para registrar/listar/ejecutar tools,
**para** que el loop y la CLI invoquen cualquier tool sin conocer su implementación.
**Feature:** feat-tool-registry · **REQ:** REQ-002
**Aceptación:**
- [ ] `pub trait Tool` en `crates/swal-core/src/tool.rs` (no tokio/fs/process imports — wasm32-clean)
- [ ] `ToolRegistry` con `register/list/execute` (DashMap)
- [ ] `cargo test -p swal-core` verde con test de register+execute

## US-003: Persistir sesiones y mensajes en SQLite
**Como** usuario,
**quiero** que cada conversación del agente se guarde en SQLite (tablas `sessions` y `messages`, WAL),
**para** poder retomar sesiones pasadas y auditar el historial.
**Feature:** feat-session-store · **REQ:** REQ-003
**Aceptación:**
- [ ] `Store` trait con CRUD de sesiones (create/append/read/delete)
- [ ] `SessionStore` implementado con rusqlite (bundled, WAL) en `crates/swal-store/src/session.rs`
- [ ] `cargo test -p swal-store` verde (round-trip en `tests/session_test.rs`)

## US-004: Ejecutar el ciclo conversacional completo
**Como** usuario,
**quiero** que `swal-agent run "tarea"` ejecute el ciclo LLM → tool_calls → execute → feedback,
**para** completar tareas reales con tools sin intervención manual.
**Feature:** feat-agent-loop · **REQ:** REQ-004
**Aceptación:**
- [ ] `AgentLoop` en `crates/swal-loop/src/loop.rs` con pipeline prompt→LLM→tools→feedback
- [ ] `cargo test -p swal-loop` verde (MockProvider round-trip: 1 task → ≥1 tool call → respuesta final)
- [ ] El loop itera hasta respuesta final o máximo de pasos configurado

## US-005: Conmutar proveedores LLM sin tocar el loop
**Como** desarrollador,
**quiero** un trait `Provider` async con `MockProvider` para tests y adaptadores a los providers de synapse-agentic (OpenRouter/DeepSeek/Gemini/Grok),
**para** testear sin API keys y desplegar con el proveedor real.
**Feature:** feat-llm-providers · **REQ:** REQ-005
**Aceptación:**
- [ ] `Provider` trait + `MockProvider` en `crates/swal-loop/src/provider.rs`
- [ ] MockProvider devuelve tool_calls y respuestas deterministas
- [ ] `cargo test -p swal-loop` verde sin red ni API keys

## US-006: Cargar skills desde el filesystem con cache
**Como** agente,
**quiero** cargar skills (SKILL.md) desde el filesystem con cache de 2 capas (LRU + snapshot),
**para** reutilizar procedimientos establecidos sin releer disco en cada turno.
**Feature:** feat-skills-loader · **REQ:** REQ-006
**Aceptación:**
- [ ] Loader de skills (~100 LOC) en `crates/swal-loop/src/skills.rs`
- [ ] Cache 2 capas: snapshot en memoria + LRU por acceso
- [ ] Test: carga N skills desde dir temporal, segunda carga usa cache (sin re-leer)

## US-007: Lanzar el agente desde CLI con configuración
**Como** usuario,
**quiero** ejecutar `swal-agent run "tarea"` leyendo configuración (modelo, proveedor, dir de sesiones) de archivo/env con defaults,
**para** operar el agente desde terminal de forma reproducible.
**Feature:** feat-cli-run · **REQ:** REQ-007
**Aceptación:**
- [ ] `cargo run -p swal-agent -- run "tarea"` completa una tarea con MockProvider
- [ ] Config desde `--config <file>` / env / defaults (modelo, provider, session dir)
- [ ] La sesión resultante queda persistida en SQLite

## US-008: Tools nativas shell/read/write/git desde gestalt
**Como** agente,
**quiero** las tools shell, read, write y git del `ToolRegistry` de gestalt registradas en mi loop,
**para** operar sobre el sistema de archivos y repos sin reimplementarlas.
**Feature:** feat-native-tools · **REQ:** REQ-008
**Aceptación:**
- [ ] `swal-agent` registra las tools de gestalt (shell/read/write/git) en `ToolRegistry`
- [ ] Test: el loop ejecuta una tool nativa (p. ej. read) vía MockProvider que la invoca
- [ ] Cero código duplicado: se consume `gestalt_core` por path-dep, no se copia

## US-009: Exponer el loop remotamente vía gateway
**Como** operador,
**quiero** un gateway HTTP/WS + MCP server que exponga el loop,
**para** que clientes externos (Telegram/Discord/webhooks, MCP clients) ejecuten tareas remotamente.
**Feature:** feat-gateway · **REQ:** REQ-009
**Aceptación:**
- [ ] `crates/swal-gateway/src/lib.rs` con servidor HTTP + WebSocket + endpoints MCP
- [ ] Test E2E: un MCP client externo completa una tarea vía gateway
- [ ] `cargo test -p swal-gateway` verde

## US-010: Consumir tools MCP externas desde el loop
**Como** agente,
**quiero** ser cliente MCP (rmcp) y consumir tools de servidores externos (incluido xavier),
**para** ampliar mis capacidades sin código nuevo por cada servicio.
**Feature:** feat-mcp-client · **REQ:** REQ-010
**Aceptación:**
- [ ] Cliente MCP con rmcp conecta a un server y lista tools
- [ ] El loop puede invocar una tool MCP remota
- [ ] `cargo test -p swal-loop` con test de cliente MCP mock

## US-011: Programar tareas con cron interno
**Como** operador,
**quiero** un scheduler tokio (cron ticks, ~100 LOC) que dispare prompts del loop en horarios,
**para** ejecutar tareas periódicas sin depender de cron del sistema.
**Feature:** feat-cron-scheduler · **REQ:** REQ-011
**Aceptación:**
- [ ] `crates/swal-sched/src/lib.rs` con timer tokio que dispara a intervalos/cron
- [ ] Test: tarea programada a 50ms se ejecuta (tokio timers)
- [ ] `cargo test -p swal-sched` verde

## US-012: Lanzar subagentes aislados
**Como** orquestador,
**quiero** lanzar subagentes como instancias aisladas del loop (worktree gestalt en native, WebWorker en web) con el mismo registry,
**para** paralelizar trabajo sin contaminar el contexto principal.
**Feature:** feat-subagents · **REQ:** REQ-012
**Aceptación:**
- [ ] `SubagentSpawner` crea una instancia aislada del loop
- [ ] Native: aislamiento vía gestalt-router worktrees
- [ ] Test: subagente corre en contexto separado y devuelve resultado

## US-013: Compactar sesiones largas
**Como** usuario,
**quiero** que el contexto se compacte automáticamente en sesiones largas (módulo compaction de synapse-agentic),
**para** no perder calidad ni tokens en sesiones extendidas.
**Feature:** feat-compaction · **REQ:** REQ-013
**Aceptación:**
- [ ] Compaction trigger en `swal-loop` cuando la sesión supera umbral
- [ ] Reutiliza `synapse-agentic` compaction (path-dep), sin reimplementar
- [ ] Test: sesión simulada larga → resumen compactado

## US-014: Buscar y guardar memoria en xavier
**Como** agente,
**quiero** hacer round-trip de búsqueda y guardado de sesiones contra xavier (HTTP/MCP client),
**para** tener memoria persistente sin embeber el motor.
**Feature:** feat-xavier-memory · **REQ:** REQ-014
**Aceptación:**
- [ ] Cliente HTTP xavier: `store` y `search` de memoria
- [ ] Round-trip test: guardar → buscar → encontrar (con xavier real o mock)
- [ ] Sin embedding de xavier: solo llamadas HTTP/MCP

## US-015: Compilar el loop a wasm32
**Como** desarrollador web,
**quiero** que `swal-core` (loop + tools trait) compile a wasm32 sin tokio/fs/process,
**para** reutilizar el mismo código en el navegador.
**Feature:** feat-wasm-core · **REQ:** REQ-015
**Aceptación:**
- [ ] `cargo build -p swal-core --target wasm32-unknown-unknown` sin errores
- [ ] Async vía wasm-bindgen-futures (sin tokio)
- [ ] El loop corre en test wasm (wasm-bindgen-test)

## US-016: Tools web: OPFS/WebContainers/isomorphic-git
**Como** usuario web,
**quiero** tools de archivos (OPFS), git (isomorphic-git) y shell (WebContainers) en el navegador,
**para** operar proyectos sin backend local (NO subprocess en web).
**Feature:** feat-tools-web · **REQ:** REQ-016
**Aceptación:**
- [ ] `crates/swal-tools-web` implementa la misma interfaz que native tools vía JS interop
- [ ] fs→OPFS, git→isomorphic-git, shell→WebContainers bridge
- [ ] No existe tool subprocess en el target web

## US-017: Persistir sesiones en IndexedDB
**Como** usuario web,
**quiero** que las sesiones se persistan en IndexedDB (rexie) con el mismo schema serde compartido,
**para** retomar conversaciones entre recargas del navegador.
**Feature:** feat-store-web · **REQ:** REQ-017
**Aceptación:**
- [ ] `Store` impl IndexedDB (rexie) en `crates/swal-store`
- [ ] Mismo schema serde compartido que el backend nativo
- [ ] Test wasm: round-trip create/read/delete en IndexedDB

## US-018: PWA Leptos con WebLLM en worker
**Como** usuario,
**quiero** una PWA Leptos (mismo codebase Rust → WASM) con el loop en un Comlink worker y WebLLM en-browser,
**para** ejecutar el agente 100% local en el navegador (o contra OpenAI-compatible remoto).
**Feature:** feat-pwa-leptos · **REQ:** REQ-018
**Aceptación:**
- [ ] PWA Leptos reutiliza tipos de `swal-core`
- [ ] Comlink worker corre loop + WebLLM fuera del hilo principal
- [ ] El loop corre en el navegador con WebLLM (o remote OpenAI-compatible)

## US-019: Sincronizar estado con CRDT a xavier
**Como** usuario multi-dispositivo,
**quiero** que el estado sincronice a xavier vía CRDT (native→EdgeMesh TCP/QUIC; web→WS/WebRTC relay),
**para** continuar mi sesión en cualquier dispositivo sin perder estado.
**Feature:** feat-sync-crdt · **REQ:** REQ-019
**Aceptación:**
- [ ] `crates/swal-sync/src/lib.rs` con SyncEngine detrás de transport trait
- [ ] Misma lógica CRDT/merge en native y web
- [ ] Test: dos nodos convergen tras ediciones concurrentes
