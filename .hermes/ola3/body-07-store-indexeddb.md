# [Ola 3.07] swal-store — IndexedDB backend (rexie, real impl)

> Ola 3 — Web. Labels: `ola3`, `wave-3` (NO `jules` yet).
> Feature: `feat-store-indexeddb` (2%), `feat-store-schema-shared` (1%).

---

## Current State (MEASURABLE)
- `crates/swal-store/src/lib.rs`: `pub mod session;` (Ola 1 #03).
- `crates/swal-store/src/session.rs`: Store trait + SessionStore rusqlite (Ola 1).
- Shared serde types: `Session`, `Message` in session.rs.

## Desired State (DELTA)
- **`crates/swal-store/src/lib.rs`**: APPEND `#[cfg(target_arch = "wasm32")] pub mod indexeddb;` (keep existing).
- **`crates/swal-store/src/indexeddb.rs`** (NEW — real impl):
  - `IndexedDbStore` implementing the SAME `Store` trait (from session.rs) over IndexedDB via `rexie` crate (or web-sys IDB if rexie unavailable — prefer rexie per REUSE docs; if dep missing, comment, do NOT edit Cargo.toml)
  - Uses the SAME serde `Session`/`Message` types (feat-store-schema-shared proof)
  - `create_session`, `append_message`, `get_session`, `list_sessions`, `delete_session` — all async IDB ops
  - wasm-gated module; native compile passes with `#[cfg(target_arch = "wasm32")]` guard (module simply absent on native)
- Test: wasm-gated (`#[cfg(target_arch="wasm32")]` with wasm-bindgen-test, may be skipped in CI) + native compile check.

## 🌐 Web Research Required
1. search: "rexie indexeddb rust crate 2026"
2. search: "rust indexeddb store trait async"
3. search: "wasm-bindgen-test indexeddb"

## Problem
The browser needs session persistence over IndexedDB with the SAME schema as native SQLite (feat-store-schema-shared). This is feat-store-indexeddb.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "struct IndexedDbStore" crates/swal-store/src/indexeddb.rs` >= 1
- [ ] `grep -c "impl.*Store.*for.*IndexedDbStore" crates/swal-store/src/indexeddb.rs` >= 1
- [ ] `grep -c "cfg(target_arch = \"wasm32\")" crates/swal-store/src/lib.rs` >= 1
- [ ] `cargo check -p swal-store` (native) — 0 errors
- [ ] `cargo test -p swal-store` — existing session tests still pass (1 ok)

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-store/src/lib.rs` | mod session | Append cfg-gated mod | LOW |
| `crates/swal-store/src/indexeddb.rs` | — | NEW: IndexedDbStore (Store impl) | HIGH |

## DO NOT touch (Anti-Regression)
- `crates/swal-store/Cargo.toml` (Ola 1 #03 owns) — if rexie missing, COMMENT
- `crates/swal-store/src/session.rs` — read-only (Store trait + types + rusqlite)
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **Same Store trait, same serde types** — import from `crate::session::{Store, Session, Message}`. Do NOT redefine.
2. wasm-gated so native check passes; native tests unaffected.
3. No Cargo.toml edits — if rexie/web-sys missing, comment on issue.
4. IDB stores: object stores `sessions` (keyPath id) + `messages` (autoIncrement).

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty (2 files)
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo check -p swal-store 2>&1 | tail -2
cargo test -p swal-store 2>&1 | grep "test result: ok"
```

## Dependencies & Merge Order
- **Depends on:** Ola 1 complete (Store trait), Ola 3.03 (wasm deps pattern — actually independent crate)
- **Parallel with:** 3.02, 3.04, 3.05, 3.06, 3.10, 3.12 — disjoint
- **Merge order within wave:** 8 of 12
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| rexie not in deps | Comment on issue; use web-sys IDB if available, else document blocker |
| Trait method signature mismatch | Read merged session.rs; implement exactly |
| Native check fails | cfg(wasm32) gate the module |
