# [Ola 2.02] swal-gateway — HTTP server (real impl)

> Ola 2 — Services. Labels: `ola2`, `wave-2` (NO `jules` yet).
> Feature: `feat-gateway` (HTTP part).

---

## Current State (MEASURABLE)
- `crates/swal-gateway/src/http.rs`: stub from Ola 2.01 (`serve()` no-op).

## Desired State (DELTA)
Replace stub in `crates/swal-gateway/src/http.rs` with a REAL axum HTTP server:
- `pub async fn serve(agent: Arc<dyn AgentHandle>, addr: SocketAddr) -> anyhow::Result<()>`:
  - Routes: `GET /health` → `{"status":"ok"}`; `POST /run` body `{"task": "..."}` → runs the loop, returns `{"content": "...", "steps": N}`
  - `AgentHandle` trait (in this file or lib.rs — do NOT touch lib.rs, define in http.rs): `async fn run_task(&self, task: &str) -> Result<serde_json::Value, String>`
  - Wire the real `AgentLoop` (Ola 1 #07) behind an `Arc<AgentLoop>` implementing `AgentHandle` (if AgentLoop's `run` is public, wrap it; if signature differs, adapt in this file only)
- Keep stubs of ws/mcp untouched.

## 🌐 Web Research Required
1. search: "axum routing POST JSON extract State 2026"
2. search: "axum Arc AppState handler"
3. search: "axum health check endpoint example"

## Problem
Clients need a REST surface to execute tasks remotely. This issue makes `POST /run` work end-to-end with the merged AgentLoop.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "async fn serve" crates/swal-gateway/src/http.rs` >= 1
- [ ] `grep -c "POST\|/run" crates/swal-gateway/src/http.rs` >= 1
- [ ] `grep -c "health" crates/swal-gateway/src/http.rs` >= 1
- [ ] `cargo check --workspace` — 0 errors
- [ ] `cargo test -p swal-gateway` (unit test on handler with MockProvider loop) — 1 ok

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-gateway/src/http.rs` | stub (2.01) | Real axum server + AgentHandle + test | MED |

## DO NOT touch (Anti-Regression)
- `crates/swal-gateway/Cargo.toml` + `lib.rs` (2.01), `src/ws.rs` (03), `src/mcp.rs` (04)
- `crates/swal-loop/` — read-only (use public API)
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **AgentLoop public API**: read merged `crates/swal-loop/src/loop.rs` — use `run(&self, task)` exactly as exposed; adapt via wrapper in THIS file only.
2. axum State pattern: `Arc<AppState>` via `with_state`.
3. Test uses MockProvider (no network, no API keys) — construct loop in test with scripted mock.
4. Do NOT edit Cargo.toml (axum already added in 2.01; if a feature is missing, comment on issue).

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-gateway 2>&1 | tail -5
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** Ola 2.01 (scaffold), Ola 1 complete (AgentLoop)
- **Parallel with:** Ola 2.03 (ws.rs), 2.04 (mcp.rs) — disjoint files
- **Merge order within wave:** 2 of 12
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| AgentLoop API differs | Wrap in this file; do NOT edit swal-loop |
| axum State borrow issues | Use `Arc<AppState>` + clone per handler |
| Test fails | Fix logic; do NOT weaken assertions |
