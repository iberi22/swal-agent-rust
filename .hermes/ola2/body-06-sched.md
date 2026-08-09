# [Ola 2.06] swal-sched — crate scaffold + cron ticker (real)

> Ola 2 — Services. Labels: `ola2`, `wave-2` (NO `jules` yet).
> Feature: `feat-cron-scheduler` (4% of scope).

---

## Current State (MEASURABLE)
- `crates/swal-sched/Cargo.toml`: empty `[dependencies]`.
- `crates/swal-sched/src/lib.rs`: 1-line doc comment only.

## Desired State (DELTA)
- **`crates/swal-sched/Cargo.toml`**: add ALL deps (this issue owns the manifest):
  - `swal-loop` (path = "../swal-loop"), `serde`, `serde_json`, `tokio` (rt-multi-thread, macros, time, sync), `anyhow`, `tracing`
- **`crates/swal-sched/src/lib.rs`** (NEW content): `pub mod ticker; pub mod subagent;` + doc.
- **`crates/swal-sched/src/ticker.rs`** (REAL — the core of this issue):
  - `Scheduler` struct: holds `Vec<ScheduledTask { name: String, cron_expr: String, prompt: String }>` + `Arc<dyn RunTask>`
  - `RunTask` trait: `async fn run(&self, task: &str) -> Result<(), String>` (implemented by AgentLoop wrapper in swal-agent, or a test mock)
  - `Scheduler::new()`, `add_task(name, interval_secs, prompt)`, `async fn run_forever(&self)`: loop over tasks with `tokio::time::sleep(interval)` firing each prompt via `RunTask`
  - Cron parsing: INTERVAL-BASED (seconds) is fine for v1 — do NOT add a cron-parser crate unless already in deps; document that cron syntax comes later
- Unit tests: task fires after ~50ms interval (tokio timers), count executions.

## 🌐 Web Research Required
1. search: "tokio time interval sleep periodic task 2026"
2. search: "rust scheduler periodic tasks pattern"
3. search: "tokio select timeout multiple tasks"

## Problem
The agent must run scheduled tasks (cron ticks). This issue delivers the timer core of swal-sched (Wave-2 done-criteria: "Cron fires a job on schedule (tokio timers)").

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "struct Scheduler" crates/swal-sched/src/ticker.rs` >= 1
- [ ] `grep -c "async fn run_forever" crates/swal-sched/src/ticker.rs` >= 1
- [ ] `grep -c "tokio::time" crates/swal-sched/src/ticker.rs` >= 1
- [ ] `cargo test -p swal-sched 2>&1 | grep "test result: ok"` — 1 match
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-sched/Cargo.toml` | empty | Add tokio/swal-loop/serde | LOW |
| `crates/swal-sched/src/lib.rs` | doc only | `pub mod ticker; pub mod subagent;` | LOW |
| `crates/swal-sched/src/ticker.rs` | — | NEW: Scheduler + RunTask + timer loop + tests | MED |
| `crates/swal-sched/src/subagent.rs` | — | NEW stub (real in 2.07) | LOW |

## DO NOT touch (Anti-Regression)
- `crates/swal-loop/`, `crates/swal-gateway/`, `crates/swal-agent/` — read-only
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **Interval-based v1** — no cron-parser dep unless already present; document.
2. Tests use `tokio::time::pause()` or short intervals (50ms); no sleeps > 2s.
3. `subagent.rs` is a compilable stub ONLY (real in 2.07).
4. `RunTask` trait keeps the scheduler decoupled from AgentLoop (testable with mock).

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty (4 files)
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-sched 2>&1 | tail -5
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** Ola 1 complete (swal-loop AgentLoop exists)
- **Parallel with:** Ola 2.01 (gateway scaffold), 2.08 (loop services scaffold) — different crates
- **Merge order within wave:** 5 of 12
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| tokio timer test flaky | Use `tokio::time::pause()` + `advance()` |
| RunTask trait too generic | Keep `Arc<dyn RunTask>`; mock in tests |
| Test fails | Fix logic; do NOT weaken assertions |
