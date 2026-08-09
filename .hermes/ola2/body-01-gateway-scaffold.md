# [Ola 2.01] swal-gateway — crate scaffold (lib.rs, Cargo.toml, stubs)

> Ola 2 — Services. Labels: `ola2`, `wave-2` (NO `jules` yet).
> Feature: `feat-gateway` (8% of scope).

---

## Current State (MEASURABLE)
- `crates/swal-gateway/Cargo.toml`: empty `[dependencies]`.
- `crates/swal-gateway/src/lib.rs`: 1-line doc comment only ("Wave skeleton").
- No http.rs / ws.rs / mcp.rs / tests dir.

## Desired State (DELTA)
- **`crates/swal-gateway/Cargo.toml`**: add ALL deps (this issue owns the manifest — later gateway issues must NOT touch it):
  - `swal-loop` (path = "../swal-loop"), `swal-core` (path = "../swal-core")
  - `axum` (default + ws features), `tokio` (rt-multi-thread, macros, net, signal), `serde`, `serde_json`, `tower` (util), `tracing`
- **`crates/swal-gateway/src/lib.rs`** (NEW content): `pub mod http; pub mod ws; pub mod mcp;` + crate doc.
- **Stubs** (minimal, compilable — real impls in issues 02-04):
  - `crates/swal-gateway/src/http.rs`: `pub async fn serve(_loop: std::sync::Arc<swal_loop::loop_mod::AgentLoop>, _addr: std::net::SocketAddr) {}` (verify actual AgentLoop path from Ola 1 #07)
  - `crates/swal-gateway/src/ws.rs`: `pub async fn accept(_conn: &str) {}`
  - `crates/swal-gateway/src/mcp.rs`: `pub fn mcp_server() -> String { String::new() }`
- Workspace compiles.

## 🌐 Web Research Required
1. search: "axum 0.7 Router serve example 2026"
2. search: "axum websocket ws feature"
3. search: "rust workspace path dependency sibling crate"

## Problem
The gateway (HTTP/WS + MCP) is the remote face of the agent loop. Its manifest + module skeleton must compile before http/ws/mcp land on disjoint files.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `ls crates/swal-gateway/src/{http,ws,mcp}.rs` — all exist
- [ ] `grep -c "pub mod" crates/swal-gateway/src/lib.rs` >= 3
- [ ] `grep -c "axum" crates/swal-gateway/Cargo.toml` >= 1
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-gateway/Cargo.toml` | empty | Add axum/tokio/serde/swal-loop | MED |
| `crates/swal-gateway/src/lib.rs` | doc only | 3 `pub mod` | LOW |
| `crates/swal-gateway/src/http.rs` | — | NEW stub | LOW |
| `crates/swal-gateway/src/ws.rs` | — | NEW stub | LOW |
| `crates/swal-gateway/src/mcp.rs` | — | NEW stub | LOW |

## DO NOT touch (Anti-Regression)
- `crates/swal-loop/`, `crates/swal-core/`, `crates/swal-store/`, `crates/swal-agent/` — read-only
- Root `Cargo.toml` profiles; `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **Verify AgentLoop path**: `grep -rn "pub struct AgentLoop" crates/swal-loop/src/` — use the REAL path in the stub signature.
2. **axum version**: use latest stable (0.7.x); if API differs, adapt stub to compile.
3. Stubs compile with `#[allow(dead_code)]` where needed; zero warnings.
4. Do NOT implement real server logic here (issues 02-04).

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty (5 files)
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo check --workspace 2>&1 | tail -3
```

## Dependencies & Merge Order
- **Depends on:** Ola 1 complete (#07 AgentLoop merged)
- **Parallel with:** Ola 2.06 (sched scaffold), Ola 2.08 (loop services scaffold) — different crates
- **Merge order within wave:** 1 of 12
- **Expected effort:** Small (<1h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| axum API mismatch | Adapt stub to the installed axum version |
| AgentLoop path unknown | `grep -rn "struct AgentLoop" crates/swal-loop/src/` and use it |
| Warnings | `#[allow(dead_code)]` on stubs |
