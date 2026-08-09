# [Ola 3.10] swal-pwa — Leptos PWA scaffold (app shell + stubs)

> Ola 3 — Web. Labels: `ola3`, `wave-3` (NO `jules` yet).
> Features: scaffolding for `feat-pwa-leptos`, `feat-pwa-comlink`, `feat-pwa-webllm`.

---

## Current State (MEASURABLE)
- No `swal-pwa` crate exists. Workspace members: 9 crates (swal-loop..swal-tools-web).
- swal-core (wasm-clean) + swal-tools-web (web tools) available from other Ola 3 issues.

## Desired State (DELTA)
- **`crates/swal-pwa/Cargo.toml`** (NEW): Leptos PWA crate (this issue owns manifest):
  - `leptos` (csr feature), `leptos_router`, `wasm-bindgen`, `serde`, `serde_json`
  - `swal-core` (path = "../swal-core"), `swal-tools-web` (path = "../swal-tools-web"), `swal-store` (path = "../swal-store"), `swal-sync` (path = "../swal-sync")
- **Root `Cargo.toml`**: ADD `"crates/swal-pwa"` to `members` — ⚠️ pre-approved ONLY for this addition (see Guard #5).
- **`crates/swal-pwa/src/lib.rs`** (NEW): `pub mod app; pub mod worker;` + doc.
- **`crates/swal-pwa/src/app.rs`** (NEW — real shell): Leptos `App` component: sidebar (sessions list) + chat view (messages + input). Uses `create_signal` for messages; calls `worker::run_task` stub. Minimal but real Leptos UI.
- **`crates/swal-pwa/src/worker.rs`** (NEW stub — real in 3.12): `pub fn run_task(_task: &str) -> String { String::new() }`
- `cargo check -p swal-pwa` passes (native compile of Leptos CSR may need wasm target — verify; if Leptos needs wasm32, document `rustup target add wasm32-unknown-unknown`).

## 🌐 Web Research Required
1. search: "leptos csr app component example 2026"
2. search: "leptos create_signal view closure"
3. search: "leptos wasm32 build check"

## Problem
The PWA dashboard (Leptos) is the browser face of the agent. This creates the crate + app shell + worker stub so 3.11 (worker+WebLLM) lands on a disjoint file.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `ls crates/swal-pwa/src/{lib,app,worker}.rs` — all exist
- [ ] `grep -c "leptos" crates/swal-pwa/Cargo.toml` >= 1
- [ ] `grep -c "swal-pwa" Cargo.toml` >= 1 (workspace member)
- [ ] `grep -c "fn App" crates/swal-pwa/src/app.rs` >= 1
- [ ] `cargo check -p swal-pwa` — 0 errors (or documented wasm-only build)

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `Cargo.toml` (root) | 9 members | Add swal-pwa member (PRE-APPROVED only) | MED |
| `crates/swal-pwa/Cargo.toml` | — | NEW: Leptos crate | MED |
| `crates/swal-pwa/src/lib.rs` | — | NEW: mods | LOW |
| `crates/swal-pwa/src/app.rs` | — | NEW: Leptos App shell | MED |
| `crates/swal-pwa/src/worker.rs` | — | NEW stub | LOW |

## DO NOT touch (Anti-Regression)
- Other crates (swal-loop/gateway/sched/agent/core/store/sync/tools-*) — read-only
- Root `Cargo.toml` `[profile.*]` sections — ONLY add the member line
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **Root Cargo.toml edit is pre-approved ONLY for adding the swal-pwa member** — nothing else.
2. Leptos version: use latest stable (0.7.x); if CSR-only APIs differ, adapt.
3. worker.rs is a stub — real Comlink+WebLLM is 3.11.
4. If Leptos requires wasm32 target and it's unavailable, document + `cargo check --target wasm32-unknown-unknown` best-effort.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty (5 files)
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
rustup target add wasm32-unknown-unknown 2>/dev/null
cargo check -p swal-pwa 2>&1 | tail -3
```

## Dependencies & Merge Order
- **Depends on:** Ola 1 complete; Ola 3.01/3.03 (swal-core wasm + tools-web available)
- **Parallel with:** Ola 3.02, 3.04-3.09 (impl issues) — disjoint files
- **Merge order within wave:** 1 of 12 (scaffold)
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| Leptos needs wasm target | Add target; document |
| Root Cargo.toml conflict | Add member line only; rebase if needed |
| Leptos API differs | Adapt to installed version |
