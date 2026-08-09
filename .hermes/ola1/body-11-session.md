# [Ola 1.11] swal-agent — Session persistence wiring (swal-store)

> Ola 1 — CLI. Labels: `ola1`, `wave-1` (NO `jules` yet).
> Feature: `feat-cli-run` (session persistence part) + `feat-session-store` integration.

---

## Current State (MEASURABLE)
- `crates/swal-agent/src/session.rs`: stub from issue 09 — `pub struct Session;`.
- `crates/swal-store` provides `Store` trait + `SessionStore` (rusqlite) from issue 03.

## Desired State (DELTA)
- **`crates/swal-agent/src/session.rs`** (REAL):
  - `SessionHandle { store: Arc<swal_store::session::SessionStore>, session_id: String }`
  - `SessionHandle::open(config: &Config) -> anyhow::Result<Self>`: opens `data/swal-agent.db` via SessionStore, creates a session row (UUID id via `uuid` crate? — uuid NOT in deps; generate id with timestamp + counter, or report if `uuid` needed — do NOT edit Cargo.toml; use `std::time::SystemTime` based id)
  - `async fn append(&self, role: &str, content: &str) -> anyhow::Result<()>`: delegates to Store trait
  - `async fn list_messages(&self) -> anyhow::Result<Vec<swal_store::session::Message>>`
  - `Session::from_handle(handle) -> Session` wrapper so the stub type is replaced cleanly
- **Unit tests** in `crates/swal-agent/src/session.rs` `#[cfg(test)]`:
  - open → append 2 → list → assert 2 messages (use `Connection::open_in_memory` if SessionStore supports it, else temp dir)

## 🌐 Web Research Required
1. search: "rust timestamp unique id without uuid"
2. search: "rusqlite in memory connection test"
3. search: "Arc SessionStore share between threads"

## Problem
Sessions must persist: each `swal-agent run` creates a session row and appends messages. This wires swal-store into the agent so the loop can persist (full wiring in issue 12).

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "struct SessionHandle" crates/swal-agent/src/session.rs` >= 1
- [ ] `grep -c "swal_store" crates/swal-agent/src/session.rs` >= 1
- [ ] `grep -c "fn append" crates/swal-agent/src/session.rs` >= 1
- [ ] `cargo test -p swal-agent session 2>&1 | grep "test result: ok"` — 1 match
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-agent/src/session.rs` | stub (09) | Real SessionHandle over swal-store + tests | MED |

## DO NOT touch (Anti-Regression)
- `crates/swal-agent/Cargo.toml` + `main.rs` (09), `src/cli.rs` (10), `src/tools.rs` (12)
- `crates/swal-store/` — read-only (issue 03 owns it; use its public API)
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **Use swal-store public API**: read `crates/swal-store/src/session.rs` (merged #03) — `Store` trait methods, `SessionStore::new(path)`. If the API doesn't fit, comment on the issue — do NOT edit swal-store.
2. **No new deps**: session id = `format!("{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos())`.
3. Tests: temp dir via `std::env::temp_dir() + unique` or in-memory DB if supported; no network.
4. `Session` type from the stub may be replaced — keep the public name `Session` exported for issue 12's wiring.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] PR contains >= 1 real source file
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-agent session 2>&1 | tail -5
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** #09 (scaffold), #03 (swal-store real)
- **Blocked by:** none
- **Parallel with:** #10 (cli.rs), #12 (tools.rs) — disjoint files
- **Merge order within wave:** 8 of 12
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| swal-store API differs from expectation | Read merged session.rs and adapt calls |
| DB path permissions | Use config session_dir; create_dir_all |
| Test DB conflicts | Unique temp dir per test; drop DB between tests |
| Test fails | Fix logic; do NOT weaken assertions |
