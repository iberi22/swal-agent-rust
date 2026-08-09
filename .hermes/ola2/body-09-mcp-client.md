# [Ola 2.09] swal-loop — MCP client (real impl, rmcp or fallback)

> Ola 2 — Services. Labels: `ola2`, `wave-2` (NO `jules` yet).
> Feature: `feat-mcp-client` (5% of scope).

---

## Current State (MEASURABLE)
- `crates/swal-loop/src/mcp_client.rs`: stub from Ola 2.08 (`McpClient::new()`, `list_tools() -> vec![]`).

## Desired State (DELTA)
Replace stub in `crates/swal-loop/src/mcp_client.rs` with a REAL MCP client:
- Per REUSE-MAP: Hermes role is MCP **client** — connect to external MCP servers, list tools, call tools.
- If an MCP client crate (`rmcp` or `gestalt_mcp` — gestalt workspace member) is ALREADY available via deps, use it. Check `crates/swal-loop/Cargo.toml` deps first; if NOT present, comment on the issue (do NOT edit Cargo.toml) and implement the FALLBACK:
  - `McpClient::connect(url)`: JSON-RPC over HTTP POST (`/mcp`): `initialize` → `tools/list` → `tools/call`
  - `list_tools() -> Result<Vec<McpTool>, McpError>`: `McpTool { name: String, description: String }`
  - `call_tool(name, args: Value) -> Result<Value, McpError>`
  - `McpError` (thiserror): `Connect(String)`, `JsonRpc(String)`, `ToolNotFound(String)`
- Unit tests: mock JSON-RPC server (or handler-level) — `tools/list` returns 2 tools; `tools/call` returns result. NO real network in tests.

## 🌐 Web Research Required
1. search: "MCP model context protocol JSON-RPC transport 2026"
2. search: "rmcp rust mcp client crate docs"
3. search: "mcp tools/list tools/call JSON-RPC schema"

## Problem
The agent must consume external MCP tools (Wave-2 done-criteria: "External MCP client completes a task via gateway" + xavier memory via MCP). The client is the loop's window to the MCP ecosystem.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "struct McpClient" crates/swal-loop/src/mcp_client.rs` >= 1
- [ ] `grep -c "fn list_tools" crates/swal-loop/src/mcp_client.rs` >= 1
- [ ] `grep -c "fn call_tool" crates/swal-loop/src/mcp_client.rs` >= 1
- [ ] `cargo test -p swal-loop mcp 2>&1 | grep "test result: ok"` — 1 match
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-loop/src/mcp_client.rs` | stub (2.08) | Real MCP client + tests | MED |

## DO NOT touch (Anti-Regression)
- `crates/swal-loop/Cargo.toml` + `lib.rs` (2.08 owns mod; 04 owns manifest) — if a dep is missing, COMMENT on issue
- `src/provider.rs`, `src/skills.rs`, `src/loop.rs` — read-only
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **No Cargo.toml edits** — fallback JSON-RPC over HTTP is CORRECT if no MCP crate is available; document which path you took.
2. **Tests never hit the network** — mock the HTTP layer (trait or local listener) or test serialization only.
3. JSON-RPC fields: `jsonrpc:"2.0"`, `id`, `method`, `params` — follow the MCP spec.
4. `reqwest` may not be in deps — if missing, use `tokio::net::TcpStream` + manual HTTP, or comment. Do NOT edit Cargo.toml.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-loop mcp 2>&1 | tail -5
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** Ola 2.08 (scaffold)
- **Parallel with:** Ola 2.10 (xavier.rs), 2.11 (compaction.rs) — disjoint files
- **Merge order within wave:** 8 of 12
- **Expected effort:** Large (4h+)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| No MCP crate in deps | JSON-RPC fallback; document |
| reqwest missing | TcpStream manual HTTP or comment on issue |
| JSON-RPC parse issues | serde_json Value-based parsing, defensive |
