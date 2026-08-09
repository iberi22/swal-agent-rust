# [Ola 3.09] swal-sync — transport trait (EdgeMesh/WS real impl)

> Ola 3 — Web. Labels: `ola3`, `wave-3` (NO `jules` yet).
> Feature: `feat-sync-transport` (1%).

---

## Current State (MEASURABLE)
- `crates/swal-sync/src/transport.rs`: stub from Ola 3.08.

## Desired State (DELTA)
Replace stub in `crates/swal-sync/src/transport.rs` with REAL transport trait + impls:
- `Transport` trait (wasm-clean): `async fn send(&self, ops: Vec<SyncOp>) -> Result<(), String>`, `async fn receive(&self) -> Result<Vec<SyncOp>, String>`
- `EdgeMeshTransport { endpoint: String }`: native — TCP/QUIC to EdgeMesh relay (if `tokio`/`quinn` available in deps; verify — if not, comment, do NOT edit Cargo.toml). Fallback: HTTP POST/GET JSON to `{endpoint}/sync` using whatever HTTP client is available (or document as stub with clear TODO if no deps).
- `WsRelayTransport { url: String }`: web — WS/WebRTC to relay (wasm-gated; web-sys WebSocket via js_sys interop).
- `fn state_hash()` reuse from engine for handshake.
- Unit tests (native): EdgeMeshTransport serialization round-trip (mock endpoint via local test server or pure serde test); WsRelayTransport compile check only.

## 🌐 Web Research Required
1. search: "rust transport trait async send receive 2026"
2. search: "quinn QUIC rust client 2026"
3. search: "web-sys WebSocket wasm rust"

## Problem
Sync must work over EdgeMesh (native) and WS/WebRTC (web). The transport trait decouples the CRDT engine from the wire.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "trait Transport" crates/swal-sync/src/transport.rs` >= 1
- [ ] `grep -c "struct EdgeMeshTransport" crates/swal-sync/src/transport.rs` >= 1
- [ ] `grep -c "WsRelayTransport\|WebSocket" crates/swal-sync/src/transport.rs` >= 1
- [ ] `cargo check -p swal-sync` — 0 errors
- [ ] `cargo test -p swal-sync transport 2>&1 | grep "test result: ok"` — 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-sync/src/transport.rs` | stub (3.08) | Real Transport trait + EdgeMesh + WS impls + tests | HIGH |

## DO NOT touch (Anti-Regression)
- `crates/swal-sync/Cargo.toml` + `lib.rs` (3.08 owns), `src/engine.rs` (3.08)
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **No Cargo.toml edits** — if quinn/tokio/web-sys missing, use available deps or document fallback (serde round-trip test only).
2. Engine (`SyncOp`, `state_hash`) imported from `crate::engine` — reuse, don't redefine.
3. wasm-gate web impls; native tests use EdgeMeshTransport serde round-trip without network.
4. Transport is thin — no CRDT logic here (engine owns it).

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-sync transport 2>&1 | tail -5
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** Ola 3.08 (scaffold + engine)
- **Parallel with:** 3.02, 3.04, 3.05, 3.06, 3.07, 3.12 — disjoint
- **Merge order within wave:** 8 of 12
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| No transport deps available | Serde round-trip test + documented fallback; comment |
| WebSocket wasm compile issues | cfg(wasm32) gate; native test skips |
| Test fails | Fix logic; do NOT weaken assertions |
