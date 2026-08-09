# [Ola 1.12] swal-agent — Native tools wiring + full loop integration

> Ola 1 — CLI (finale). Labels: `ola1`, `wave-1` (NO `jules` yet).
> Features: `feat-native-tools` (4%) + completes `feat-cli-run` (6%).

---

## Current State (MEASURABLE)
- `crates/swal-agent/src/tools.rs`: stub from issue 09 — `register_defaults` no-op.
- Available (merged): `swal-core` ToolRegistry (#02), `swal-loop` AgentLoop (#07) + Provider/MockProvider (#05), `swal-store` (#03), CLI+config (#10), session wiring (#11).

## Desired State (DELTA)
- **`crates/swal-agent/src/tools.rs`** (REAL):
  - `pub fn register_defaults(reg: &swal_core::tool::ToolRegistry)`:
    - If `gestalt_core` exposes a tools module (ToolRegistry reuse per REUSE-MAP), wire its shell/read/write/git tools here via adapter (gestalt's Tool trait may differ — adapt with a wrapper implementing `swal_core::Tool`).
    - If gestalt's tools API is not directly reusable (verify in the merged dep!), implement 2 minimal native tools inline as fallback: `read_file(path)` and `echo(text)` — document which path was taken.
  - `pub fn make_registry() -> swal_core::tool::ToolRegistry` — builds and returns populated registry.
- **`crates/swal-agent/src/cli.rs`** (FINAL wiring, minimal edit): in `run()` after config resolution:
  1. `let reg = tools::make_registry();`
  2. `let provider = Arc::new(MockProvider::new(vec![final_response]));` — MockProvider from swal-loop (mock mode until real providers in Wave 2)
  3. `let loop_ = AgentLoop::new(provider, reg, SkillLoader::new("skills"));`
  4. `let out = loop_.run(&task).await?;` print `out.content`
  5. Session: `session::SessionHandle::open(&config)` + append task + append final content (best-effort, warn on error)
- **Test** `crates/swal-agent/tests/run_e2e.rs` (NEW): `cargo run` binary via `std::process::Command` or direct `cli::run()` — assert `swal-agent run "hello"` exits 0 and prints output (MockProvider final response).

## 🌐 Web Research Required
1. search: "gestalt_core rust tools module ToolRegistry github iberi22"
2. search: "rust adapter pattern trait wrapper"
3. search: "cargo test binary integration std::process::Command"

## Problem
This is the Ola-1 finale: `swal-agent run "task"` executes the full loop (LLM→tools→feedback) with native tools and persists the session — the Wave-1 done-criteria from docs/PLAN.md.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "fn make_registry" crates/swal-agent/src/tools.rs` >= 1
- [ ] `grep -c "AgentLoop::new" crates/swal-agent/src/cli.rs` >= 1
- [ ] `cargo run -p swal-agent -- run "hello" 2>&1 | grep -iE "hello|done|ok"` — prints final output
- [ ] `cargo test -p swal-agent --test run_e2e 2>&1 | grep "test result: ok"` — 1 match
- [ ] `cargo check --workspace` — 0 errors
- [ ] Session persisted: `ls data/swal-agent.db` after a run (or `data/sessions/` per config)

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-agent/src/tools.rs` | stub (09) | Real registry: gestalt tools via adapter (or minimal native fallback) | MED |
| `crates/swal-agent/src/cli.rs` | real (10) | Final loop wiring (registry + provider + loop + session) | MED |
| `crates/swal-agent/tests/run_e2e.rs` | — | NEW: binary E2E test | LOW |

## DO NOT touch (Anti-Regression)
- `crates/swal-agent/Cargo.toml` + `main.rs` (09), `src/config.rs` (10), `src/session.rs` (11)
- `crates/swal-loop/`, `crates/swal-core/`, `crates/swal-store/` — read-only
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **Verify gestalt's tools API FIRST** (`grep -rn "ToolRegistry\|pub trait Tool" ~/.cargo/git/checkouts/gestalt-*` or in the merged dep source) — if not directly reusable, the inline fallback (read_file/echo) is CORRECT and documented; do NOT invent a fake gestalt adapter.
2. **MockProvider only** — real LLM providers are Wave 2 (synapse wiring). This issue must work offline.
3. `SkillLoader::new("skills")` — path may not exist; loader must handle missing dir gracefully (empty).
4. Session persistence is best-effort — a failure to append must NOT crash the run.
5. Do NOT edit Cargo.toml for `uuid`/`tempfile` — use std.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty (3 files)
- [ ] PR contains >= 1 real source file
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo run -p swal-agent -- run "hello" 2>&1 | tail -5
cargo test -p swal-agent --test run_e2e 2>&1 | tail -5
cargo check --workspace 2>&1 | tail -2
ls data/swal-agent.db 2>/dev/null || echo "no db yet (check config session_dir)"
```

## Dependencies & Merge Order
- **Depends on:** #09 (scaffold), #10 (cli+config), #11 (session), #05 (MockProvider), #07 (AgentLoop), #02 (ToolRegistry)
- **Blocked by:** #10, #11 (need them merged for cli.rs final wiring)
- **Parallel with:** none (this is the integration point)
- **Merge order within wave:** 9 of 12
- **Expected effort:** Large (4h+)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| gestalt tools API incompatible | Use documented inline fallback (read_file/echo); note in PR description |
| MockProvider script mismatch | Read provider.rs (merged #05); script a simple final response |
| SkillLoader dir missing | Ensure loader returns empty gracefully |
| E2E binary test flaky | Test `cli::run()` directly (async) instead of spawning binary |
| Test fails | Fix logic; do NOT weaken assertions |
