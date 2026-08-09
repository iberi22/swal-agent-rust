# [Ola 3.08] swal-sync — crate scaffold + CRDT engine (real core)

> Ola 3 — Web. Labels: `ola3`, `wave-3` (NO `jules` yet).
> Feature: `feat-sync-crdt` (2%).

---

## Current State (MEASURABLE)
- `crates/swal-sync/Cargo.toml`: empty `[dependencies]`.
- `crates/swal-sync/src/lib.rs`: 1-line doc comment only.

## Desired State (DELTA)
- **`crates/swal-sync/Cargo.toml`**: add deps (this issue owns manifest):
  - `serde`, `serde_json`, `thiserror`
- **`crates/swal-sync/src/lib.rs`**: `pub mod engine; pub mod transport;` + doc.
- **`crates/swal-sync/src/engine.rs`** (REAL — the core of this issue):
  - `SyncEngine` struct: holds `Vec<SyncOp>` log + `merkle: HashMap<String, u64>` (doc/op hashes)
  - `SyncOp { id: String, doc_id: String, op_type: String, payload: serde_json::Value, ts: u64 }` (serde)
  - `apply_local(op) -> SyncOp` (append to log, update merkle)
  - `merge_remote(ops: Vec<SyncOp>) -> MergeResult { applied: usize, conflicts: usize }`: CRDT merge — idempotent by op id (already-seen ops skipped), conflicts counted when same id different payload
  - `state_hash() -> String`: deterministic hash of the op log (for transport sync handshake)
  - Unit tests: (1) same ops applied twice → idempotent (applied 0 second time); (2) two engines converge to same state_hash after merging each other's ops; (3) conflict detected on same-id different-payload.
- **`crates/swal-sync/src/transport.rs`**: NEW stub (real in 3.09).

## 🌐 Web Research Required
1. search: "CRDT op-based merge idempotent rust 2026"
2. search: "state-based CRDT convergence proof"
3. search: "merkle hash state sync handshake"

## Problem
Multi-device sessions need convergent sync (feat-sync-crdt). The engine is wasm-clean (pure Rust, no I/O) — the transport trait (3.09) plugs EdgeMesh/WS behind it.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "struct SyncEngine" crates/swal-sync/src/engine.rs` >= 1
- [ ] `grep -c "fn merge_remote" crates/swal-sync/src/engine.rs` >= 1
- [ ] `grep -c "fn state_hash" crates/swal-sync/src/engine.rs` >= 1
- [ ] `cargo test -p swal-sync 2>&1 | grep "test result: ok"` — 1 match
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-sync/Cargo.toml` | empty | Add serde/thiserror | LOW |
| `crates/swal-sync/src/lib.rs` | doc only | 2 `pub mod` | LOW |
| `crates/swal-sync/src/engine.rs` | — | NEW: SyncEngine CRDT core + tests | MED |
| `crates/swal-sync/src/transport.rs` | — | NEW stub (real 3.09) | LOW |

## DO NOT touch (Anti-Regression)
- Other crates; root `Cargo.toml` profiles; `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **Pure Rust, wasm-clean** — no tokio/fs/network in engine.rs (transport handles I/O).
2. CRDT merge is idempotent by op id — that's the convergence guarantee.
3. `state_hash` deterministic (sort ops by id before hashing).
4. transport.rs is a compilable stub ONLY.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty (4 files)
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-sync 2>&1 | tail -5
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** Ola 1 complete (workspace compiles)
- **Parallel with:** Ola 3.01 (core wasm), 3.03 (tools-web), 3.11 (pwa) — different crates
- **Merge order within wave:** 1 of 12 (scaffold + core)
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| Convergence test fails | Sort ops deterministically; ensure idempotency by id |
| Hash differs across runs | Hash sorted op ids + payloads (no HashMap iteration order) |
| Test fails | Fix logic; do NOT weaken assertions |
