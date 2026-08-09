# [Ola 2.04] swal-gateway — MCP server (real impl)

> Ola 2 — Services. Labels: `ola2`, `wave-2` (NO `jules` yet).
> Feature: `feat-gateway` (MCP part) + `feat-mcp-client` (exposes loop as MCP server).

---

## Current State (MEASURABLE)
- `crates/swal-gateway/src/mcp.rs`: stub from Ola 2.01 (`mcp_server()` returning String).

## Desired State (DELTA)
Replace stub in `crates/swal-gateway/src/mcp.rs` with a REAL MCP server surface:
- Per REUSE-MAP: Hermes role is MCP **client** in the loop; the gateway exposes an MCP **server** so external MCP clients can drive the loop.
- If `gestalt_mcp` (member of gestalt workspace) or `rmcp` provides server utilities, use them via git dep IF ALREADY in gateway Cargo.toml (2.01 added only swal-loop/axum/tokio — if an MCP crate is needed, comment on the issue; do NOT edit Cargo.toml).
- Minimal fallback (no new deps): expose MCP-style JSON endpoints over the HTTP server (implemented in http.rs): `POST /mcp/tools` → tool list; `POST /mcp/call` → execute tool via AgentHandle. Implement the glue in mcp.rs and a route registration fn `pub fn routes() -> axum::Router` that http.rs can merge (document the integration point).
- Unit test: `mcp_call` helper executes a mock tool.

## 🌐 Web Research Required
1. search: "model context protocol server rust rmcp 2026"
2. search: "gestalt_mcp rust crate github iberi22"
3. search: "MCP protocol JSON-RPC tools list call"

## Problem
External MCP clients must be able to drive the agent loop (Wave-2 done-criteria: "External MCP client completes a task via gateway").

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "mcp" crates/swal-gateway/src/mcp.rs` >= 1
- [ ] `grep -c "fn routes\|fn mcp_" crates/swal-gateway/src/mcp.rs` >= 1
- [ ] `cargo check --workspace` — 0 errors
- [ ] `cargo test -p swal-gateway` — test ok

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-gateway/src/mcp.rs` | stub (2.01) | Real MCP surface (server glue + routes) | HIGH |

## DO NOT touch (Anti-Regression)
- `crates/swal-gateway/Cargo.toml` + `lib.rs` (2.01), `src/http.rs` (02), `src/ws.rs` (03)
- `crates/swal-loop/` — read-only
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **No new Cargo.toml deps** unless already present — fallback path (JSON endpoints) is CORRECT and documented if an MCP crate is unavailable.
2. If you use `gestalt_mcp`, verify it exists in gestalt workspace members first (`git clone --depth 1 https://github.com/iberi22/gestalt.git /tmp/g && grep -A15 members /tmp/g/Cargo.toml`).
3. `routes()` must be mergeable into http.rs's Router — return `axum::Router`.
4. Test uses mock; no network/keys.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-gateway 2>&1 | tail -5
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** Ola 2.01 (scaffold), 2.02 (http.rs routes integration)
- **Parallel with:** Ola 2.03 (ws.rs)
- **Merge order within wave:** 3 of 12
- **Expected effort:** Large (4h+)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| No MCP crate available | Fallback JSON endpoints (documented) — do NOT edit Cargo.toml |
| Router merge conflict | Provide `routes()` returning Router; http.rs merges with `.merge()` |
| Test fails | Fix logic; do NOT weaken assertions |
