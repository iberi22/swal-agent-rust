# HANDOFF — Continuar en otra sesión

> Estado: 2026-08-09 (10:40). Proyecto: swal-agent-rust — clon de Hermes en Rust.
> Repo: https://github.com/iberi22/swal-agent-rust (público).

## Qué es el proyecto
Rust-native clone of the Hermes agent harness (ultra-efficient, 15-40 MB vs ~345 MB/instancia Python).
One codebase → native CLI/TUI + WASM PWA (reemplaza swal-agent-runner). Reusa gestalt,
synapse-agentic, xavier (NO replicar). Kimi k3 diseñó la arquitectura (minimalista).

## Estado actual (DONE)
- [x] Wave 0: docs canónicos + esqueleto 9 crates
- [x] **Catálogo 100% features**: `.gitcore/features.json` (19 features, pesos 55/30/15 → 100%)
- [x] **SRS**: `docs/SRS/USER-STORIES.md` (US-001..019) + `docs/SRS/REQUIREMENTS.md` (REQ-001..019)
- [x] **12 issues Ola 1 creados** (#1-#12, labels ola1,wave-1) — bodies en `.hermes/ola1/body-*.md`
- [x] **Grupo A COMPLETADO**: PRs #13 (foundation), #14 (store), #15 (core) MERGED ✅
      → features foundation/tool-registry/session-store al 100% (reconciliado con evidencia real)
- [x] **Grupo B DESPACHADO**: issue #4 (swal-loop scaffold) con label jules
- [x] **Cron persistencia + AUTO-AVANCE**: `swal-agent-rust-features-persist` (0d384e5087f3, cada 30 min,
      no_agent, script `~/.hermes/scripts/swal-agent-rust-features.py`)
      → escanea % real + auto-reconcilia issues cerrados + auto-despacha siguiente grupo

## Issues Ola 1 (12) — estado de grupos
| Grupo | Issues | Estado |
|-------|--------|--------|
| A | #1, #2, #3 | ✅ COMPLETO (PRs #13-15 merged) |
| B | #4 scaffold loop | ✅ COMPLETO (PR #29 merged) |
| C | #5 provider, #6 skills | ✅ COMPLETO (PRs #30, #31 merged) |
| D | #7 AgentLoop, #9 scaffold agent | 🟢 EN CURSO (sesiones 2883384449109234112, 705251803403282729) |
| E | #8 e2e, #10 cli+config, #11 session | ⏳ espera D |
| F | #12 tools wiring final | ⏳ espera E |

## Ola 2 (30%) — issues #17-#28 CREADOS (labels ola2,wave-2, bodies en .hermes/ola2/)
| Grupo | Issues | Contenido |
|-------|--------|-----------|
| G | #17, #22, #24 | scaffolds: gateway, sched, loop-services |
| H | #18, #19, #23 | gateway http+ws (disjuntos) + subagents |
| I | #20, #25, #26, #27 | gateway mcp + mcp-client + xavier + compaction |
| J | #21 | gateway e2e (integra mcp routes) |
| K | #28 | reconciliation (features.json wave2) |

Ola 2 se despacha SOLO cuando Ola 1 F (#12) esté closed — el cron lo hace automáticamente.

## CATÁLOGO 37 FEATURES (12-13 por wave)
- Wave 1: 13 features (git-deps, ci-pipeline, tool-trait, tool-registry, store-trait,
  session-sqlite, provider-trait, mock-provider, skills-loader, skills-cache,
  agent-loop, cli-run, native-tools) — 55%
- Wave 2: 12 features (gateway-http, gateway-ws, gateway-mcp-server, mcp-client,
  mcp-tools, cron-ticker, cron-tasks, subagent-spawn, subagent-isolation,
  compaction, xavier-store, xavier-search) — 30%
- Wave 3: 12 features (wasm-core-loop, wasm-core-tools, tools-opfs, tools-git-web,
  tools-shell-web, store-indexeddb, store-schema-shared, pwa-leptos, pwa-comlink,
  pwa-webllm, sync-crdt, sync-transport) — 15%
- Total 37 = 100%. SRS: US-001..US-037, REQ-001..REQ-037.
- % actual: 21% (6 features W1 al 100% por PRs #13-15)

## AUTO-AVANCE (el cron lo hace solo)
El script `swal-agent-rust-features.py` ahora:
1. **Auto-reconcilia**: issue CLOSED (PR merged) → sube claimed del feature a su target
   (mapa ISSUE_FEATURES cubre Ola 1 #1-12 Y Ola 2 #17-28; targets acumulativos MAX por feature)
2. **Auto-despacha**: grupo anterior todo closed → aplica label jules a los issues OPEN
   del siguiente grupo (Ola 1: C→D→E→F; luego Ola 2: G→H→I→J→K)
3. **Escanea % real** con 7 checks (paths, tests, recencia, caveats) → clamped a claimed
4. Persiste local + Xavier; SILENT si nada cambió

## Cron de persistencia
- Job: `swal-agent-rust-features-persist` (0d384e5087f3, **cada 20 min**, no_agent, deliver=local)
- Script: `~/.hermes/scripts/swal-agent-rust-features.py`
- Depende de `gh` CLI (auto-reconcilia/despacha); sin gh → solo escanea % y persiste.
- Nota: los issues de código NO tocan features.json — el % se actualiza por el scan
  (evidencia real en disco) + reconciliación por issues cerrados.

## Después de Ola 1 → Ola 2 (30%)
Gateway HTTP/WS+MCP (swal-gateway, rmcp), cron+subagentes (swal-sched), xavier HTTP/MCP client,
synapse-agentic Hive/providers/compaction. 12 issues nuevos (body-XX en `.hermes/ola2/`).

## Después → Ola 3 (15%) — reemplaza swal-agent-runner
swal-core wasm32-clean, swal-tools-web (OPFS/WebContainers/isomorphic-git), swal-store IndexedDB,
Leptos PWA + Comlink worker + WebLLM, swal-sync CRDT → EdgeMesh/xavier. 12 issues nuevos.

## Referencias útiles
- Arquitectura: docs/ARCHITECTURE.md · Reuso: docs/REUSE-MAP.md · Plan: docs/PLAN.md
- Features 100%: .gitcore/features.json · Stories: docs/SRS/USER-STORIES.md · Reqs: docs/SRS/REQUIREMENTS.md
- Skills: jules-async-orchestration, swal-wave-execution, gitcore-auto-verify, swal-project-monitoring
- Provider memoria Xavier de Hermes ROTO (tool_call_id) → usar curl HTTP con token de ~/proyectosSWAL/xavier/.env
