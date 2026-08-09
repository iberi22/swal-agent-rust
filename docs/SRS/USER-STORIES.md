# User Stories — swal-agent-rust

> Canonical user stories for 100% feature scope (37 features: 13 Wave1 + 12 Wave2 + 12 Wave3).
> Format: `US-NNN` ↔ `REQ-NNN` ↔ `feat-*` (see `.gitcore/features.json`).
> Verified 2026-08-09. Each story has verifiable acceptance criteria.

## US-001: Git deps (gestalt_core, synapse-agentic)
**Como** parte de desarrollador del ecosistema SWAL,
**quiero** Como desarrollador del ecosistema SWAL, quiero gestalt_core y synapse-agentic como git deps del workspace, para componer sin duplicar código.

**Feature:** feat-git-deps · **REQ:** REQ-001 · **Wave:** W1
**Aceptación:**
- [ ] El workspace declara [workspace.dependencies] con gestalt_core, synapse-agentic, gestalt-state vía git = https://github.com/iberi22/gestalt.git.

## US-002: CI pipeline (fmt+clippy+test)
**Como** parte de mantenedor,
**quiero** Como mantenedor, quiero CI que corra fmt+clippy+test en cada push/PR, para detectar regresiones automáticamente.

**Feature:** feat-ci-pipeline · **REQ:** REQ-002 · **Wave:** W1
**Aceptación:**
- [ ] GitHub Actions verifica cargo fmt --all --check, clippy -D warnings y test --workspace.

## US-003: Tool trait (wasm32-clean)
**Como** parte de agente,
**quiero** Como agente, quiero un trait Tool con schema JSON de entrada y execute async, para invocar cualquier tool uniformemente (también en wasm).

**Feature:** feat-tool-trait · **REQ:** REQ-003 · **Wave:** W1
**Aceptación:**
- [ ] Tool trait en swal-core sin imports tokio/fs/process (wasm32-clean).

## US-004: ToolRegistry (DashMap)
**Como** parte de loop,
**quiero** Como loop, quiero un ToolRegistry DashMap para registrar/listar/ejecutar tools, para despachar tool_calls sin acoplar implementaciones.

**Feature:** feat-tool-registry · **REQ:** REQ-004 · **Wave:** W1
**Aceptación:**
- [ ] ToolRegistry con register/list/execute en swal-core.

## US-005: Store trait (session CRUD)
**Como** parte de usuario,
**quiero** Como usuario, quiero un trait Store de sesiones (CRUD), para que native (SQLite) y web (IndexedDB) compartan interfaz.

**Feature:** feat-store-trait · **REQ:** REQ-005 · **Wave:** W1
**Aceptación:**
- [ ] Store trait: create_session, append_message, get_session, list_sessions, delete_session.

## US-006: SessionStore SQLite (rusqlite WAL)
**Como** parte de usuario,
**quiero** Como usuario, quiero persistir sesiones en SQLite con WAL, para retomar conversaciones entre ejecuciones.

**Feature:** feat-session-sqlite · **REQ:** REQ-006 · **Wave:** W1
**Aceptación:**
- [ ] SessionStore rusqlite bundled, schema sessions+messages, data/swal-agent.db.

## US-007: LLM Provider trait (async)
**Como** parte de loop,
**quiero** Como loop, quiero un trait Provider async (messages → response con tool_calls), para conmutar LLMs sin tocar el loop.

**Feature:** feat-provider-trait · **REQ:** REQ-007 · **Wave:** W1
**Aceptación:**
- [ ] Provider trait + ProviderResponse{content, tool_calls} + ProviderError.

## US-008: MockProvider (deterministic)
**Como** parte de desarrollador,
**quiero** Como desarrollador, quiero un MockProvider determinista y scripteable, para testear el loop sin red ni API keys.

**Feature:** feat-mock-provider · **REQ:** REQ-008 · **Wave:** W1
**Aceptación:**
- [ ] MockProvider con respuestas enlatadas (incluye tool_calls) y unit tests.

## US-009: Skills file loader (frontmatter)
**Como** parte de agente,
**quiero** Como agente, quiero cargar SKILL.md desde filesystem parseando frontmatter, para reutilizar procedimientos canónicos.

**Feature:** feat-skills-loader · **REQ:** REQ-009 · **Wave:** W1
**Aceptación:**
- [ ] SkillLoader parsea name/description del frontmatter y cuerpo como content.

## US-010: Skills 2-layer cache (LRU+snapshot)
**Como** parte de agente,
**quiero** Como agente, quiero cache de 2 capas (snapshot + LRU) para skills, para no releer disco en cada turno.

**Feature:** feat-skills-cache · **REQ:** REQ-010 · **Wave:** W1
**Aceptación:**
- [ ] Snapshot en memoria + LRU; test: segunda carga no re-walkea.

## US-011: AgentLoop (LLM→tools→feedback)
**Como** parte de usuario,
**quiero** Como usuario, quiero que run() ejecute el ciclo LLM→tools→feedback hasta respuesta final, para completar tareas complejas.

**Feature:** feat-agent-loop · **REQ:** REQ-011 · **Wave:** W1
**Aceptación:**
- [ ] AgentLoop con max_steps, ejecución de tool_calls, AgentOutput{content, steps, tool_calls_executed}.

## US-012: CLI run + config (env/file)
**Como** parte de usuario,
**quiero** Como usuario, quiero swal-agent run "task" con config CLI>env>defaults, para operar el agente reproduciblemente.

**Feature:** feat-cli-run · **REQ:** REQ-012 · **Wave:** W1
**Aceptación:**
- [ ] clap Run subcommand + Config{model, provider, session_dir, max_steps} desde JSON/env.

## US-013: Native tools wiring + session persist
**Como** parte de agente,
**quiero** Como agente, quiero las tools shell/read/write/git de gestalt registradas y sesión persistida, para operar el sistema sin reimplementar.

**Feature:** feat-native-tools · **REQ:** REQ-013 · **Wave:** W1
**Aceptación:**
- [ ] Adapter de gestalt tools al Tool trait + SessionHandle append en cada run.

## US-014: Gateway HTTP server (axum)
**Como** parte de operador,
**quiero** Como operador, quiero un servidor HTTP (GET /health, POST /run), para ejecutar tareas remotamente vía REST.

**Feature:** feat-gateway-http · **REQ:** REQ-014 · **Wave:** W2
**Aceptación:**
- [ ] axum server con AgentHandle (run_task) sobre Arc<AgentLoop>.

## US-015: Gateway WebSocket server
**Como** parte de operador,
**quiero** Como operador, quiero un servidor WebSocket que reciba tareas y devuelva resultados, para clientes en tiempo real.

**Feature:** feat-gateway-ws · **REQ:** REQ-015 · **Wave:** W2
**Aceptación:**
- [ ] WS handler: mensaje {task} → run_task → respuesta {content, steps}.

## US-016: Gateway MCP server (routes)
**Como** parte de cliente MCP externo,
**quiero** Como cliente MCP externo, quiero endpoints MCP en el gateway, para conducir el loop desde herramientas MCP.

**Feature:** feat-gateway-mcp-server · **REQ:** REQ-016 · **Wave:** W2
**Aceptación:**
- [ ] routes() MCP: tools/list + tools/call mergeables en el Router HTTP.

## US-017: MCP client (JSON-RPC/rmcp)
**Como** parte de agente,
**quiero** Como agente, quiero ser cliente MCP (connect/initialize/list_tools/call_tool), para consumir servidores MCP externos.

**Feature:** feat-mcp-client · **REQ:** REQ-017 · **Wave:** W2
**Aceptación:**
- [ ] McpClient JSON-RPC (o rmcp si está disponible) + McpError.

## US-018: MCP tools in loop (external)
**Como** parte de agente,
**quiero** Como agente, quiero invocar tools de servidores MCP externos desde el loop, para ampliar capacidades sin código nuevo.

**Feature:** feat-mcp-tools · **REQ:** REQ-018 · **Wave:** W2
**Aceptación:**
- [ ] Tools MCP externas integrables vía McpClient en AgentLoop.

## US-019: Cron ticker (tokio timers)
**Como** parte de operador,
**quiero** Como operador, quiero un scheduler tokio que dispare tareas por intervalo, para ejecución periódica sin cron del sistema.

**Feature:** feat-cron-ticker · **REQ:** REQ-019 · **Wave:** W2
**Aceptación:**
- [ ] Scheduler + tokio::time (interval/advance) + tests.

## US-020: Scheduled tasks (RunTask)
**Como** parte de operador,
**quiero** Como operador, quiero tareas programadas con prompt propio vía trait RunTask, para desacoplar scheduler del loop.

**Feature:** feat-cron-tasks · **REQ:** REQ-020 · **Wave:** W2
**Aceptación:**
- [ ] ScheduledTask{name, interval, prompt} + RunTask trait + mock en tests.

## US-021: Subagent spawner (isolated)
**Como** parte de orquestador,
**quiero** Como orquestador, quiero lanzar subagentes aislados con límite de concurrencia, para paralelizar sin saturar.

**Feature:** feat-subagent-spawn · **REQ:** REQ-021 · **Wave:** W2
**Aceptación:**
- [ ] SubagentSpawner con tokio::spawn + Semaphore(max_concurrent).

## US-022: Subagent isolation (worktree)
**Como** parte de orquestador,
**quiero** Como orquestador, quiero aislamiento real (gestalt worktrees native, WebWorkers web), para que cada subagente tenga contexto separado.

**Feature:** feat-subagent-isolation · **REQ:** REQ-022 · **Wave:** W2
**Aceptación:**
- [ ] Aislamiento documentado: v1 spawn aislado; worktrees como follow-up native.

## US-023: Compaction (synapse reuse)
**Como** parte de usuario,
**quiero** Como usuario, quiero compactar contextos largos reutilizando synapse-agentic, para no degradar sesiones extensas.

**Feature:** feat-compaction · **REQ:** REQ-023 · **Wave:** W2
**Aceptación:**
- [ ] compact(messages, max_tokens) con marker + should_compact heurístico.

## US-024: Xavier memory store (HTTP client)
**Como** parte de agente,
**quiero** Como agente, quiero guardar memoria en xavier vía HTTP (X-Xavier-Token), para persistir sin embeber el motor.

**Feature:** feat-xavier-store · **REQ:** REQ-024 · **Wave:** W2
**Aceptación:**
- [ ] XavierClient.store(path, content) → POST /v1/memories + XavierTransport trait.

## US-025: Xavier memory search (round-trip)
**Como** parte de agente,
**quiero** Como agente, quiero buscar memoria en xavier (round-trip store→search), para recuperar contexto relevante.

**Feature:** feat-xavier-search · **REQ:** REQ-025 · **Wave:** W2
**Aceptación:**
- [ ] XavierClient.search(query, limit) → hits; test con MockTransport.

## US-026: swal-core loop on wasm32
**Como** parte de desarrollador web,
**quiero** Como desarrollador web, quiero el loop compilando a wasm32 sin tokio/fs/process, para reutilizar el mismo código en el navegador.

**Feature:** feat-wasm-core-loop · **REQ:** REQ-026 · **Wave:** W3
**Aceptación:**
- [ ] cargo build --target wasm32-unknown-unknown + async vía wasm-bindgen-futures.

## US-027: Tool trait on wasm32
**Como** parte de desarrollador web,
**quiero** Como desarrollador web, quiero el Tool trait verificado en wasm32, para que tools y loop compartan abstracción.

**Feature:** feat-wasm-core-tools · **REQ:** REQ-027 · **Wave:** W3
**Aceptación:**
- [ ] Tool trait compila para wasm32 (sin std::process/tokio).

## US-028: Web fs tools (OPFS)
**Como** parte de usuario web,
**quiero** Como usuario web, quiero tools de archivos sobre OPFS, para operar ficheros en el navegador.

**Feature:** feat-tools-opfs · **REQ:** REQ-028 · **Wave:** W3
**Aceptación:**
- [ ] swal-tools-web fs→OPFS vía JS interop.

## US-029: Web git tools (isomorphic-git)
**Como** parte de usuario web,
**quiero** Como usuario web, quiero git en el navegador vía isomorphic-git, para operar repos sin backend.

**Feature:** feat-tools-git-web · **REQ:** REQ-029 · **Wave:** W3
**Aceptación:**
- [ ] swal-tools-web git→isomorphic-git (JS interop).

## US-030: Web shell (WebContainers)
**Como** parte de usuario web,
**quiero** Como usuario web, quiero shell vía WebContainers (sin subprocess), para ejecutar comandos en el navegador.

**Feature:** feat-tools-shell-web · **REQ:** REQ-030 · **Wave:** W3
**Aceptación:**
- [ ] swal-tools-web shell→WebContainers bridge; prohibido subprocess.

## US-031: IndexedDB store backend (rexie)
**Como** parte de usuario web,
**quiero** Como usuario web, quiero sesiones en IndexedDB (rexie), para retomar conversaciones entre recargas.

**Feature:** feat-store-indexeddb · **REQ:** REQ-031 · **Wave:** W3
**Aceptación:**
- [ ] Store impl IndexedDB (rexie) con mismo schema serde.

## US-032: Shared serde schema (native+web)
**Como** parte de desarrollador,
**quiero** Como desarrollador, quiero UN schema serde compartido nativo+web, para que los backends sean intercambiables.

**Feature:** feat-store-schema-shared · **REQ:** REQ-032 · **Wave:** W3
**Aceptación:**
- [ ] Session/Message serde types usados por rusqlite e IndexedDB.

## US-033: Leptos PWA dashboard
**Como** parte de usuario,
**quiero** Como usuario, quiero una PWA Leptos reutilizando tipos de swal-core, para operar el agente desde el navegador.

**Feature:** feat-pwa-leptos · **REQ:** REQ-033 · **Wave:** W3
**Aceptación:**
- [ ] PWA Leptos (Rust→WASM) con tipos compartidos.

## US-034: Comlink worker (loop off main)
**Como** parte de usuario,
**quiero** Como usuario, quiero el loop en un Comlink worker (fuera del hilo principal), para UI fluida.

**Feature:** feat-pwa-comlink · **REQ:** REQ-034 · **Wave:** W3
**Aceptación:**
- [ ] Comlink worker corre loop + WebLLM off-main-thread.

## US-035: WebLLM in-browser inference
**Como** parte de usuario,
**quiero** Como usuario, quiero inferencia en-browser con WebLLM (o remote OpenAI-compatible), para privacidad total.

**Feature:** feat-pwa-webllm · **REQ:** REQ-035 · **Wave:** W3
**Aceptación:**
- [ ] WebLLM (MLC-LLM→WASM+WebGPU) API estilo OpenAI.

## US-036: CRDT sync engine
**Como** parte de usuario multi-dispositivo,
**quiero** Como usuario multi-dispositivo, quiero sync CRDT convergente, para continuar sesiones en cualquier dispositivo.

**Feature:** feat-sync-crdt · **REQ:** REQ-036 · **Wave:** W3
**Aceptación:**
- [ ] SyncEngine con misma lógica CRDT/merge en native y web.

## US-037: Sync transport (EdgeMesh/WS)
**Como** parte de usuario multi-dispositivo,
**quiero** Como usuario multi-dispositivo, quiero transporte EdgeMesh (native TCP/QUIC) y WS/WebRTC (web), para sincronizar con xavier.

**Feature:** feat-sync-transport · **REQ:** REQ-037 · **Wave:** W3
**Aceptación:**
- [ ] Transport trait: native→EdgeMesh, web→WS/WebRTC relay.
