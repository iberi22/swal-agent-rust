# [Ola 2.07] swal-sched — Subagent spawn (isolated loop)

> Ola 2 — Services. Labels: `ola2`, `wave-2` (NO `jules` yet).
> Feature: `feat-subagents` (5% of scope).

---

## Current State (MEASURABLE)
- `crates/swal-sched/src/subagent.rs`: stub from Ola 2.06.

## Desired State (DELTA)
Replace stub in `crates/swal-sched/src/subagent.rs` with REAL subagent spawning:
- `SubagentSpawner` struct: holds `Arc<dyn RunTask>` (same trait as ticker) + spawn config (max_concurrent: usize)
- `async fn spawn(&self, task: &str) -> Result<SubagentHandle, String>`:
  - Spawns an ISOLATED task execution (per REUSE-MAP: native isolation via gestalt-router worktrees is Wave 2+; v1 = separate `tokio::spawn` with its own prompt context, documented)
  - `SubagentHandle { id: String, join: JoinHandle<Result<(), String>> }` with `await_completion()`
- `Semaphore`-bounded concurrency (tokio::sync::Semaphore) — prevents runaway parallel tasks
- Unit tests: spawn 2 subagents concurrently, both complete; semaphore limit respected (3 tasks, max 2 → at most 2 concurrent)

## 🌐 Web Research Required
1. search: "tokio spawn join handle await result"
2. search: "tokio Semaphore concurrency limit"
3. search: "rust subagent pattern isolated task context"

## Problem
The agent must spawn isolated subagent runs (Wave-2 done-criteria: "Subagent runs in isolated worktree (gestalt-router)" — v1 delivers isolated task contexts; worktree isolation documented as follow-up).

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "struct SubagentSpawner" crates/swal-sched/src/subagent.rs` >= 1
- [ ] `grep -c "Semaphore" crates/swal-sched/src/subagent.rs` >= 1
- [ ] `grep -c "async fn spawn" crates/swal-sched/src/subagent.rs` >= 1
- [ ] `cargo test -p swal-sched 2>&1 | grep "test result: ok"` — 1 match
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-sched/src/subagent.rs` | stub (2.06) | Real spawner + semaphore + tests | MED |

## DO NOT touch (Anti-Regression)
- `crates/swal-sched/Cargo.toml` + `lib.rs` (2.06), `src/ticker.rs` (06)
- `crates/swal-loop/` — read-only (RunTask impl mock in tests)
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **No new deps**: tokio sync/macros already in sched Cargo.toml (2.06).
2. Subagent v1 = isolated `tokio::spawn` context; document that gestalt-router worktree isolation is the native follow-up (do NOT try to add worktree logic here).
3. Tests: mock RunTask that sleeps ~30ms; assert completion + semaphore concurrency via atomic counter.
4. No network/keys.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-sched 2>&1 | tail -5
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** Ola 2.06 (scaffold + RunTask trait)
- **Parallel with:** Ola 2.08+ (loop services — different crate)
- **Merge order within wave:** 6 of 12
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| Semaphore borrow issues | Clone `Arc<Semaphore>` per spawn |
| JoinHandle type complexity | Keep `SubagentHandle` simple: id + JoinHandle |
| Test flaky | Use tokio::time::pause or short sleeps with generous asserts |
