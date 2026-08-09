# [Ola 2.10] swal-loop — Xavier memory client (store/search round-trip)

> Ola 2 — Services. Labels: `ola2`, `wave-2` (NO `jules` yet).
> Feature: `feat-xavier-memory` (4% of scope).

---

## Current State (MEASURABLE)
- `crates/swal-loop/src/xavier.rs`: stub from Ola 2.08 (`XavierClient::new(base)`, `search() -> vec![]`).

## Desired State (DELTA)
Replace stub in `crates/swal-loop/src/xavier.rs` with a REAL xavier HTTP client:
- `XavierClient { base_url: String, token: Option<String> }` — `new(base_url)`, `with_token(token)`
- `async fn store(&self, path: &str, content: &str) -> Result<(), XavierError>`:
  - `POST {base}/v1/memories` with `X-Xavier-Token` header (if token set), body `{"path": path, "content": content}`
- `async fn search(&self, query: &str, limit: Option<usize>) -> Result<Vec<XavierHit>, XavierError>`:
  - `POST {base}/v1/memories/search` body `{"query": query, "limit": N}` → parse hits (`XavierHit { path, content, score }`)
- `XavierError` (thiserror): `Http(String)`, `Unauthorized`, `Parse(String)`
- Round-trip test: mock HTTP layer (trait `XavierTransport` with `store`/`search`, real client uses HTTP, test uses in-memory mock) — asserts store→search returns the stored content.

## 🌐 Web Research Required
1. search: "xavier swal memory API v1 memories search token"
2. search: "rust http client POST header token async 2026"
3. search: "trait abstraction http client test mock rust"

## Problem
The agent needs persistent memory (Wave-2 done-criteria: "Memory: session search + store round-trip via xavier HTTP/MCP"). Xavier is consumed as a client — never embedded.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "struct XavierClient" crates/swal-loop/src/xavier.rs` >= 1
- [ ] `grep -c "async fn store" crates/swal-loop/src/xavier.rs` >= 1
- [ ] `grep -c "async fn search" crates/swal-loop/src/xavier.rs` >= 1
- [ ] `cargo test -p swal-loop xavier 2>&1 | grep "test result: ok"` — 1 match
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-loop/src/xavier.rs` | stub (2.08) | Real client (store/search) + transport trait + tests | MED |

## DO NOT touch (Anti-Regression)
- `crates/swal-loop/Cargo.toml` + `lib.rs` (2.08 owns mod), `src/mcp_client.rs` (09), `src/compaction.rs` (11)
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **No Cargo.toml edits** — HTTP via tokio TcpStream manual or whatever is available; if `reqwest`/`ureq` missing, implement a minimal HTTP POST with `tokio::net::TcpStream` (document) or comment.
2. **Transport trait** for testability: `#[async_trait] pub trait XavierTransport` with `store`/`search`; real `HttpTransport`, test `MockTransport`.
3. Tests NEVER hit network or real xavier.
4. Token header: `X-Xavier-Token: <token>` (SWAL convention).
5. 401 → `Unauthorized` error variant.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-loop xavier 2>&1 | tail -5
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** Ola 2.08 (scaffold)
- **Parallel with:** Ola 2.09 (mcp_client.rs), 2.11 (compaction.rs) — disjoint files
- **Merge order within wave:** 8 of 12
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| HTTP client dep missing | Minimal TcpStream HTTP or comment; do NOT edit Cargo.toml |
| Token header convention | Use `X-Xavier-Token` |
| Test fails | Fix mock; do NOT weaken assertions |
