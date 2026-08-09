# [Ola 2.03] swal-gateway — WebSocket server (real impl)

> Ola 2 — Services. Labels: `ola2`, `wave-2` (NO `jules` yet).
> Feature: `feat-gateway` (WS part).

---

## Current State (MEASURABLE)
- `crates/swal-gateway/src/ws.rs`: stub from Ola 2.01 (`accept()` no-op).

## Desired State (DELTA)
Replace stub in `crates/swal-gateway/src/ws.rs` with a REAL WebSocket server:
- `pub async fn handle_ws(stream: WebSocketStream, agent: Arc<dyn AgentHandle>)` (or axum-native signature):
  - On client message `{"task": "..."}` → run task via AgentHandle → send back `{"content": "...", "steps": N}`
  - Echo/health handshake message supported
- Reuse `AgentHandle` trait from `crates/swal-gateway/src/http.rs` (import via `crate::http::AgentHandle`).
- Unit test: in-memory WS round-trip or handler-level test with MockProvider loop.

## 🌐 Web Research Required
1. search: "axum websocket example receive send 2026"
2. search: "axum ws WebSocketUpgrade handler"
3. search: "tokio-tungstenite or axum ws message types"

## Problem
Real-time clients (dashboard, chat adapters) need WS to stream tasks/results without polling REST.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "WebSocket\|ws" crates/swal-gateway/src/ws.rs` >= 1
- [ ] `grep -c "AgentHandle" crates/swal-gateway/src/ws.rs` >= 1
- [ ] `cargo check --workspace` — 0 errors
- [ ] `cargo test -p swal-gateway` — test ok (WS handler with mock)

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-gateway/src/ws.rs` | stub (2.01) | Real WS server + test | MED |

## DO NOT touch (Anti-Regression)
- `crates/swal-gateway/Cargo.toml` + `lib.rs` (2.01), `src/http.rs` (02 — import only), `src/mcp.rs` (04)
- `crates/swal-loop/` — read-only
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **Import AgentHandle from http.rs** — if the trait isn't `pub`, comment on issue 02 (do NOT edit http.rs).
2. WS API depends on axum version installed — adapt to it.
3. Test with MockProvider; no network/keys.
4. Handle client disconnect gracefully (no panic).

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-gateway 2>&1 | tail -5
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** Ola 2.01 (scaffold), 2.02 (AgentHandle trait)
- **Parallel with:** Ola 2.04 (mcp.rs)
- **Merge order within wave:** 2 of 12
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| axum ws API differs | Adapt to installed version |
| AgentHandle not pub | Comment on 02; define local wrapper in ws.rs instead (do NOT edit http.rs) |
| Disconnect panics | Match on `Ok/Err` and break cleanly |
