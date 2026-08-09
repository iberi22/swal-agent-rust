# HANDOFF — Continuar en otra sesión

> Estado: 2026-08-08 (noche). Proyecto: swal-agent-rust — clon de Hermes en Rust.
> Repo: https://github.com/iberi22/swal-agent-rust (público, push hecho).

## Qué es el proyecto
Rust-native clone of the Hermes agent harness (ultra-efficient, 15-40 MB vs ~345 MB/instancia Python).
One codebase → native CLI/TUI + WASM PWA (reemplaza swal-agent-runner). Reusa gestalt,
synapse-agentic, xavier (NO replicar). Kimi k3 diseñó la arquitectura (minimalista).
Investigación web incorporada: AutoAgents/Rig benchmarks, rmcp, wasm-bindgen+WebLLM, tokio+rayon.

## Estado actual (DONE)
- [x] Wave 0: docs canónicos + esqueleto 9 crates. Commits: ab7d89d, 994e96a, 225aa20
- [x] Repo GitHub creado y pusheado: https://github.com/iberi22/swal-agent-rust
- [x] Docs: README.md, docs/ARCHITECTURE.md, docs/REUSE-MAP.md, docs/PLAN.md
- [x] 3 bodies de issues Wave 1 listos: `.hermes/ola1/body-01-foundation.md`, `body-02-loop.md`, `body-03-cli.md`
- [x] File islands verificadas: 0 conflictos (paralelo seguro)
- [x] Decisión guardada en Xavier: `swal-agent-rust/decisions/001-architecture`
- [ ] AGENTS.md del repo: NO creado (guard bloqueó por falta de respuesta; no reintentar sin aprobación explícita)

## PRÓXIMO PASO (cuando se reanude): despachar Wave 1 a Jules
```bash
cd ~/proyectosSWAL/swal-agent-rust
# 1. Crear los 3 issues (usar nombres REALES de archivo):
gh issue create --title "$(head -1 .hermes/ola1/body-01-foundation.md | sed 's/^# //')" \
  --body-file .hermes/ola1/body-01-foundation.md --label ola1,wave-1
gh issue create --title "$(head -1 .hermes/ola1/body-02-loop.md | sed 's/^# //')" \
  --body-file .hermes/ola1/body-02-loop.md --label ola1,wave-1
gh issue create --title "$(head -1 .hermes/ola1/body-03-cli.md | sed 's/^# //')" \
  --body-file .hermes/ola1/body-03-cli.md --label ola1,wave-1

# 2. VERIFICAR (regla de platino: releer bodies antes de dispatch)
gh issue list --label ola1
# Releer cada body: gh issue view <NUM> --json body | head -30

# 3. SOLO ENTONCES dispatch:
gh issue edit <NUM1> --add-label jules
gh issue edit <NUM2> --add-label jules
gh issue edit <NUM3> --add-label jules

# 4. Monitoreo (24h): PRs con archivos
gh pr list --state open --repo iberi22/swal-agent-rust
# Anti-empty-PR: gh pr view <PR> --json files --jq '.files | length'
```

## Issues Wave 1 (diseño ya hecho)
| Issue | Archivo body | Contenido | Merge order |
|-------|-------------|-----------|-------------|
| #1 foundation | body-01-foundation.md | git deps (gestalt_core, synapse-agentic) + swal-core Tool trait + swal-store SessionStore rusqlite + CI | 1 |
| #2 loop | body-02-loop.md | AgentLoop (LLM→tools→feedback, MockProvider, skills loader) | 2 |
| #3 cli | body-03-cli.md | swal-agent run + config + persistencia sesión | 3 |

Dependencias: 2→1, 3→1,2. Islands: Cargo.toml+swal-core+swal-store+.github / swal-loop / swal-agent.

## Después de Wave 1 → Wave 2 (30%)
Gateway HTTP/WS+MCP (swal-gateway, rmcp), cron+subagentes (swal-sched), xavier HTTP/MCP client,
synapse-agentic Hive/providers/compaction. Done: MCP cliente remoto, cron dispara, subagente aislado, compaction.

## Después → Wave 3 (15%) — reemplaza swal-agent-runner
swal-core wasm32-clean, swal-tools-web (OPFS/WebContainers/isomorphic-git), swal-store IndexedDB,
Leptos PWA + Comlink worker + WebLLM, swal-sync CRDT → EdgeMesh/xavier.

## Referencias útiles
- Arquitectura: docs/ARCHITECTURE.md (Tokio I/O + Rayon CPU, pools separados, rayon::spawn+oneshot)
- Reuso: docs/REUSE-MAP.md (qué tomar de gestalt/synapse-agentic/xavier, qué NO)
- Plan: docs/PLAN.md (done-criteria de cada wave)
- Skills de orquestación: ~/.hermes/skills/gitcore-jules-issues (template canónico), swal-wave-execution
- Provider memoria Xavier de Hermes ROTO (tool_call_id) → usar curl HTTP con token de ~/proyectosSWAL/xavier/.env
