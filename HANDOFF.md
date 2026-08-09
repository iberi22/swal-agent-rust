# HANDOFF — Continuar en otra sesión

> Estado: 2026-08-09 (madrugada). Proyecto: swal-agent-rust — clon de Hermes en Rust.
> Repo: https://github.com/iberi22/swal-agent-rust (público).

## Qué es el proyecto
Rust-native clone of the Hermes agent harness (ultra-efficient, 15-40 MB vs ~345 MB/instancia Python).
One codebase → native CLI/TUI + WASM PWA (reemplaza swal-agent-runner). Reusa gestalt,
synapse-agentic, xavier (NO replicar). Kimi k3 diseñó la arquitectura (minimalista).

## Estado actual (DONE)
- [x] Wave 0: docs canónicos + esqueleto 9 crates (commits ab7d89d, 994e96a, 225aa20, 6668464)
- [x] **Catálogo 100% features**: `.gitcore/features.json` (19 features, pesos 55/30/15 → 100%)
- [x] **SRS con historias de usuario**: `docs/SRS/USER-STORIES.md` (US-001..019) + `docs/SRS/REQUIREMENTS.md` (REQ-001..019)
- [x] **12 issues Ola 1 creados** (#1-#12, labels ola1,wave-1) — bodies en `.hermes/ola1/body-*.md`
- [x] **Harness file islands verificado** — grupos de despacho definidos
- [x] **Dispatch Grupo A a Jules**: #1 foundation, #2 core, #3 store (labels jules aplicados)
- [x] **Cron persistencia**: `swal-agent-rust-features-persist` (0d384e5087f3, cada 30 min,
      no_agent, script `~/.hermes/scripts/swal-agent-rust-features.py` → features.json + Xavier)
- [x] Labels ola1/wave-1/jules creados en el repo

## Issues Ola 1 (12) — grupos de despacho
| Grupo | Issues | Contenido | Merge order |
|-------|--------|-----------|-------------|
| A (DISPATCHED) | #1 | foundation: git deps + CI | 1 |
| A (DISPATCHED) | #2 | swal-core: Tool trait + ToolRegistry (wasm32-clean) | 2 |
| A (DISPATCHED) | #3 | swal-store: Store trait + SessionStore rusqlite | 2 |
| B (wait A) | #4 | swal-loop scaffold (lib.rs, Cargo.toml, stubs) | 3 |
| C (wait B) | #5 | provider.rs: Provider + MockProvider real | 4 |
| C (wait B) | #6 | skills.rs: 2-layer cache loader real | 4 |
| D (wait C) | #7 | loop.rs: AgentLoop core (LLM→tools→feedback) | 5 |
| D (wait C) | #9 | swal-agent scaffold (main.rs, Cargo.toml, stubs) | 7 |
| E (wait D) | #8 | loop e2e tests (public API) | 6 |
| E (wait D) | #10 | cli.rs + config.rs real (run + config) | 8 |
| E (wait D) | #11 | session.rs real (persistencia) | 8 |
| F (wait E) | #12 | tools.rs + cli wiring final (run E2E con MockProvider) | 9 |

Regla: aplicar label `jules` SOLO al grupo cuyo grupo previo esté mergeado.
Verificar antes con harness: los "conflictos" #4↔#5/#6/#7 y #9↔#10/#11/#12 son scaffold→impl (secuencial, correcto).

## Próximo paso (cuando #1-#3 mergeen)
```bash
cd ~/proyectosSWAL/swal-agent-rust
# 1. Verificar merges
gh pr list --state open --repo iberi22/swal-agent-rust
# 2. Dispatch Grupo B (issue 4):
gh issue edit 4 --add-label jules
# 3. Luego C (5,6), D (7,9), E (8,10,11), F (12) — siempre tras merge del grupo previo
```

## Cron de persistencia
- Job: `swal-agent-rust-features-persist` (0d384e5087f3, cada 30 min, no_agent, deliver=local)
- Script: `~/.hermes/scripts/swal-agent-rust-features.py`
  - Escanea `.gitcore/features.json` (19 features) → recalcula % real (7 checks ponderados)
  - Persiste local (features.json) + Xavier (POST /v1/memories, token de xavier/.env)
  - SILENT si no hay cambio (watchdog pattern); reporta solo cuando el % global cambia
- Nota: los issues de código NO tocan features.json — el % se actualiza por el scan
  (el scanner mide evidencia real en disco: paths implemented_in, tests, recencia, caveats)

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
