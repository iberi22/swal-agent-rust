# [Ola 2.05] swal-gateway — E2E integration test (MCP/HTTP client completes task)

> Ola 2 — Services. Labels: `ola2`, `wave-2` (NO `jules` yet).
> Feature: `feat-gateway` (E2E proof).

---

## Current State (MEASURABLE)
- `crates/swal-gateway/src/`: http.rs (02), ws.rs (03), mcp.rs (04) implemented.
- `crates/swal-gateway/tests/` does NOT exist.

## Desired State (DELTA)
- **`crates/swal-gateway/tests/gateway_e2e.rs`** (NEW — integration test on PUBLIC API):
  1. Build `AgentLoop` with MockProvider (scripted: final response) + echo tool
  2. Start `http::serve(arc_loop, "127.0.0.1:0")` on ephemeral port (or call handler directly)
  3. HTTP client: `POST /health` → 200 `{"status":"ok"}`
  4. `POST /run {"task":"say hi"}` → 200 with `content` == mock final response
  5. If mcp routes merged: `POST /mcp/tools` → non-empty tool list
- The test proves the WAVE-2 done-criteria: "External MCP client completes a task via gateway".

## 🌐 Web Research Required
1. search: "rust integration test axum ephemeral port 127.0.0.1 0"
2. search: "reqwest rust http client test 2026"
3. search: "axum test with tower ServiceExt oneshot"

## Problem
Unit tests prove handlers; the E2E test proves the full path: client → HTTP/WS/MCP → AgentLoop → response, which is the Wave-2 done-criteria.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `ls crates/swal-gateway/tests/gateway_e2e.rs` — exists
- [ ] `grep -c "fn test_" crates/swal-gateway/tests/gateway_e2e.rs` >= 2
- [ ] `cargo test -p swal-gateway --test gateway_e2e 2>&1 | grep "test result: ok"` — 1 match
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-gateway/tests/gateway_e2e.rs` | — | NEW: E2E health + run + mcp tools | MED |

## DO NOT touch (Anti-Regression)
- `crates/swal-gateway/src/*` (all owned by 2.01-2.04), `Cargo.toml`
- `crates/swal-loop/` — read-only
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **Public API only**: import `swal_gateway::http::serve` etc. If not exported, comment on the owning issue.
2. **Ephemeral port**: bind `127.0.0.1:0` and read the actual port from the listener (or use `tower::ServiceExt::oneshot` on the Router — prefer this if available; no real sockets needed).
3. MockProvider scripted; no network/keys/sleeps.
4. If an HTTP client crate is needed and NOT in Cargo.toml → use `tower::ServiceExt` oneshot (no extra deps) — do NOT edit Cargo.toml.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty (1 new file)
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-gateway --test gateway_e2e 2>&1 | tail -6
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** Ola 2.02 (http), 2.04 (mcp routes)
- **Merge order within wave:** 4 of 12
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| serve() not exportable | Use `tower::ServiceExt::oneshot` on Router directly |
| Port binding flaky | oneshot approach avoids sockets entirely |
| Test fails | Fix logic; do NOT weaken assertions |
