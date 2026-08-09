# [Ola 1.03] swal-store — Store trait + SessionStore (rusqlite)

> Ola 1 — Foundation. Labels: `ola1`, `wave-1` (NO `jules` yet).
> Feature: `feat-session-store` (8% of scope).

---

## Current State (MEASURABLE)
- `crates/swal-store/Cargo.toml`: empty `[dependencies]`.
- `crates/swal-store/src/lib.rs`: doc comment only. No Store trait, no schema, no rusqlite.

## Desired State (DELTA)
- **`crates/swal-store/Cargo.toml`**: add `rusqlite` (features=["bundled"]), `serde` (derive), `serde_json`, `chrono` (serde feature).
- **`crates/swal-store/src/session.rs`** (NEW):
  - Shared serde types: `Session { id: String, created_at: i64, updated_at: i64, summary: String }`, `Message { id: i64, session_id: String, role: String, content: String, ts: i64 }`
  - `Store` trait: `create_session`, `append_message`, `get_session`, `list_sessions`, `delete_session` (async or sync — your choice, document it)
  - `SessionStore` (rusqlite): opens `data/swal-agent.db` (create dirs), `PRAGMA journal_mode=WAL`, creates tables:
    - `sessions(id TEXT PK, created_at INTEGER, updated_at INTEGER, summary TEXT)`
    - `messages(id INTEGER PK AUTOINCREMENT, session_id TEXT, role TEXT, content TEXT, ts INTEGER)`
- **`crates/swal-store/src/lib.rs`**: `pub mod session;`
- **Test** `crates/swal-store/tests/session_test.rs` (NEW): create → append 2 messages → read back → delete; verify row counts.

## 🌐 Web Research Required
1. search: "rusqlite Connection execute params insert select 2026"
2. search: "rusqlite bundled feature compile nixos"
3. search: "PRAGMA journal_mode WAL rusqlite"

## Problem
Sessions must persist across runs (native SQLite; later IndexedDB on web). A `Store` trait abstracts the backend so the loop and CLI are backend-agnostic.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "pub trait Store" crates/swal-store/src/session.rs` >= 1
- [ ] `grep -c "pub struct SessionStore" crates/swal-store/src/session.rs` >= 1
- [ ] `grep -c "journal_mode=WAL" crates/swal-store/src/session.rs` >= 1
- [ ] `cargo test -p swal-store 2>&1 | grep "test result: ok"` — 1 match
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-store/Cargo.toml` | empty deps | Add rusqlite(bundled)/serde/chrono | LOW |
| `crates/swal-store/src/lib.rs` | doc only | `pub mod session;` | LOW |
| `crates/swal-store/src/session.rs` | — | NEW: Store trait + SessionStore + schema | MED |
| `crates/swal-store/tests/session_test.rs` | — | NEW: CRUD round-trip test | LOW |

## DO NOT touch (Anti-Regression)
- Root `Cargo.toml` profiles/workspace.dependencies (issue 01 owns)
- `crates/swal-core/`, `crates/swal-loop/`, `crates/swal-agent/` — other file islands
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **rusqlite bundled**: use `rusqlite = { version = "X", features = ["bundled"] }` so SQLite compiles without system lib (NixOS-safe).
2. **Verify rusqlite API** on docs.rs before writing — method names matter (`conn.execute`, `conn.query_row`, `params!`).
3. **Timestamp**: use `chrono::Utc::now().timestamp()` (i64 seconds).
4. Tests must not need network or API keys; clean up the test DB (`tempfile` or a `:memory:` connection per test — prefer `Connection::open_in_memory` for tests).
5. `data/` dir must be created with `std::fs::create_dir_all` before opening the DB.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] PR contains >= 1 real source file
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-store 2>&1 | tail -5
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** #01 (optional — no git deps needed here)
- **Blocked by:** none
- **Parallel with:** #02 (swal-core — different crate, disjoint island)
- **Merge order within wave:** 2 of 12
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| bundled sqlite compile fails | Ensure `cc` available; on NixOS `nix-shell -p openssl.dev pkg-config` |
| Schema mismatch in test | Drop/recreate test DB in setup; assert on fresh connection |
| Borrow/move errors in trait | Implement `Store` for `&SessionStore` or `Arc<SessionStore>` as documented |
| Test fails | Fix test or implementation; do NOT weaken assertions |
