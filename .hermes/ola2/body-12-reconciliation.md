# [Ola 2.12] swal-agent — serve command wiring + Wave 2 reconciliation

> Ola 2 — Services (finale). Labels: `ola2`, `wave-2` (NO `jules` yet).
> Features: `feat-gateway` (serve wiring), `feat-cron-scheduler`, `feat-subagents`, `feat-mcp-client`, `feat-xavier-memory`, `feat-compaction`.

---

## Current State (MEASURABLE)
- `crates/swal-agent/src/cli.rs`: Ola 1 wired `run` (AgentLoop + MockProvider + tools + session).
- `crates/swal-gateway` (2.01-2.05), `crates/swal-sched` (2.06-2.07), loop services (2.08-2.11) merged.
- `.gitcore/features.json`: wave2 features at claimed targets (auto-reconciled as issues close).

## Desired State (DELTA)
- **`crates/swal-agent/src/cli.rs`** (minimal edit): add `Serve` subcommand:
  - `swal-agent serve --addr 127.0.0.1:8080`: builds the loop (same as `run`) → starts `swal_gateway::http::serve` + optional `swal_sched::ticker::Scheduler` with tasks from config
  - Wiring is thin: construct loop once, pass `Arc` to gateway + sched
- **`crates/swal-agent/Cargo.toml`**: add `swal-gateway` + `swal-sched` path deps — ⚠️ this manifest is owned by Ola 1 #09; if adding deps conflicts, comment on this issue instead of editing (or the orchestrator pre-approved: see Anti-Hallucination Guard #5)
- **`.gitcore/features.json`**: ⚠️ NO EDITAR — el cron `swal-agent-rust-features-persist`
  lo actualiza automáticamente con el % real. Este issue SOLO verifica que el cron
  refleje los merges (puede correr `python3 ~/.hermes/scripts/swal-agent-rust-features.py --force`
  para forzar el scan, pero no editar el JSON a mano).
- Verify `cargo check --workspace` + full test suite green.

## 🌐 Web Research Required
1. search: "axum serve SocketAddr bind cli"
2. search: "rust clap subcommand serve example"

## Problem
This closes Wave 2: the CLI exposes the gateway + scheduler so remote/scheduled operation works end-to-end, and features.json reflects the honest final %.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "Serve" crates/swal-agent/src/cli.rs` >= 1
- [ ] `grep -c "swal_gateway" crates/swal-agent/src/cli.rs` >= 1
- [ ] `cargo check --workspace` — 0 errors
- [ ] `cargo test --workspace 2>&1 | grep -c "test result: ok"` >= 4 (loop, store, gateway, sched)
- [ ] `python3 -c "import json; d=json.load(open('.gitcore/features.json')); print(all(f['progress_pct']>=100 for f in d['features'] if f['wave']=='wave2'))"` — True (all wave2 features 100%)

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-agent/src/cli.rs` | Ola 1 (run) | Add Serve subcommand wiring | MED |
| `crates/swal-agent/Cargo.toml` | Ola 1 deps | Add swal-gateway/swal-sched path deps | MED |
| `.gitcore/features.json` | wave2 targets | Final honest % (100 or actual) | LOW |

## DO NOT touch (Anti-Regression)
- `crates/swal-loop/`, `crates/swal-gateway/src/`, `crates/swal-sched/src/` — read-only
- `crates/swal-agent/src/{main,config,session,tools}.rs` — read-only (only cli.rs edited here)
- `docs/`, `README.md`, `HANDOFF.md`

## Anti-Hallucination Guard ⚠️
1. **features.json is orchestrator-owned** — this is the ONLY wave-2 issue allowed to touch it; set honest % (100 only if the E2E/unit evidence exists).
2. Loop construction must be shared (build once, Arc both gateway and sched).
3. `serve` must not break `run` — existing tests must still pass.
4. If gateway/sched public APIs differ from expectation, read the merged source and adapt; comment on the owning issue if something is missing.
5. ⚠️ Cargo.toml edit: pre-approved ONLY to add the two path deps; nothing else.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty (3 files)
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo check --workspace 2>&1 | tail -3
cargo test --workspace 2>&1 | grep "test result: ok" | wc -l
python3 -c "import json; d=json.load(open('.gitcore/features.json')); print(sum(f['progress_pct']*f['weight'] for f in d['features'])/100)"
```

## Dependencies & Merge Order
- **Depends on:** Ola 2.01-2.11 ALL merged; Ola 1 complete
- **Merge order within wave:** 12 of 12 (LAST — reconciliation)
- **Expected effort:** Large (4h+)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| gateway/sched API mismatch | Read merged source; adapt wiring |
| Existing run tests break | Fix wiring, keep run intact |
| features.json evidence missing | Set honest % (not inflated); comment |
