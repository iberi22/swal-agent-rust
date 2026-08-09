# [Ola 1.08] swal-loop — E2E integration test (MockProvider + tool round-trip)

> Ola 1 — Core. Labels: `ola1`, `wave-1` (NO `jules` yet).
> Feature: `feat-agent-loop` (test completeness).

---

## Current State (MEASURABLE)
- `crates/swal-loop/src/loop.rs`: AgentLoop implemented in #07 (run loop + tool execution).
- `crates/swal-loop/tests/` directory does NOT exist.

## Desired State (DELTA)
- **`crates/swal-loop/tests/loop_e2e.rs`** (NEW — integration test, NOT unit test):
  1. Build a `ToolRegistry` with a real echo tool (register a minimal Tool impl inline: `name="echo"`, returns args back)
  2. Script `MockProvider`: response 1 → ToolCall(echo, {"text":"hi"}), response 2 → final "done"
  3. `AgentLoop::new(...).run("say hi")` → assert `content == "done"`, `tool_calls_executed == 1`, `steps == 2`
  4. Second test: max_steps=1 with a provider that always returns tool_calls → assert `LoopError::MaxSteps`
- The file uses the PUBLIC API only (`swal_loop::{AgentLoop, ...}`), proving the crate works as a library.

## 🌐 Web Research Required
1. search: "rust integration tests tests directory extern crate"
2. search: "cargo test integration test public api"

## Problem
Unit tests inside loop.rs prove internals; an integration test in `tests/` proves the PUBLIC API works end-to-end (registry + provider + loop), which is what `swal-agent` will consume.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `ls crates/swal-loop/tests/loop_e2e.rs` — exists
- [ ] `grep -c "fn test_" crates/swal-loop/tests/loop_e2e.rs` >= 2
- [ ] `cargo test -p swal-loop --test loop_e2e 2>&1 | grep "test result: ok"` — 1 match
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-loop/tests/loop_e2e.rs` | — | NEW: E2E public-API test (echo tool round-trip + max_steps) | LOW |

## DO NOT touch (Anti-Regression)
- `crates/swal-loop/src/*` (all owned by 04-07), `Cargo.toml`, `lib.rs`
- `crates/swal-core/`, `crates/swal-store/`, `crates/swal-agent/`
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **Public API only**: import `swal_loop::...` — if an item isn't exported from lib.rs, report it (lib.rs is issue 04's, but a missing export is a real gap to surface via issue comment).
2. **MockProvider scripted** — no network, no sleeps, deterministic.
3. Echo tool: implement `Tool` trait inline in the test file (swal-core dep is available via swal-loop's public re-export or direct path dep — check what swal-loop re-exports; if not re-exported, add `swal-core` as a dev-dependency? NO — report if the public API is insufficient).
4. `assert!` with meaningful messages.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty (1 new file)
- [ ] PR contains >= 1 real source file
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-loop --test loop_e2e 2>&1 | tail -6
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** #07 (AgentLoop real)
- **Blocked by:** none
- **Parallel with:** #09+ (swal-agent issues — different crate)
- **Merge order within wave:** 6 of 12
- **Expected effort:** Small (<1h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| swal-core not accessible from test | Use `swal_loop` public re-exports only; if missing, comment on the issue |
| MockProvider script API differs | Read provider.rs (merged #05) and adapt — do NOT edit provider.rs |
| Test fails | Fix test logic; do NOT weaken assertions |
