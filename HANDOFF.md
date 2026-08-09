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
| B | #4 scaffold loop | 🟢 EN CURSO (label jules) |
| C | #5 provider, #6 skills | ⏳ espera B |
| D | #7 AgentLoop, #9 scaffold agent | ⏳ espera C |
| E | #8 e2e, #10 cli+config, #11 session | ⏳ espera D |
| F | #12 tools wiring final | ⏳ espera E |

## AUTO-AVANCE (el cron lo hace solo)
El script `swal-agent-rust-features.py` ahora:
1. **Auto-reconcilia**: issue CLOSED (PR merged) → sube claimed del feature a su target
   (mapa ISSUE_FEATURES: #7→agent-loop 60%, #8→+40%, #10→cli-run 60%, #11→+40%, #12→native-tools 100%)
2. **Auto-despacha**: grupo anterior todo closed → aplica label jules a los issues OPEN
   del siguiente grupo (grupo C: #5+#6 simultáneo; D: #7+#9; E: #8+#10+#11; F: #12)
3. **Escanea % real** con 7 checks (paths, tests, recencia, caveats) → clamped a claimed
4. Persiste local + Xavier; SILENT si nada cambió

## Próximo paso (manual opcional)
El cron avanza solo. Si quieres acelerar: verificar PRs abiertos y merges manualmente:
```bash
cd ~/proyectosSWAL/swal-agent-rust
gh pr list --state open --repo iberi22/swal-agent-rust
gh issue list --label jules --repo iberi22/swal-agent-rust
# El script despacha el grupo siguiente automáticamente al mergear el actual
```

## Cron de persistencia
- Job: `swal-agent-rust-features-persist` (0d384e5087f3, cada 30 min, no_agent, deliver=local)
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
