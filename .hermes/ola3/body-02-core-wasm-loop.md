# [Ola 3.02] swal-core — wasm32 loop (real impl, wasm-bindgen-futures)

> Ola 3 — Web. Labels: `ola3`, `wave-3` (NO `jules` yet).
> Feature: `feat-wasm-core-loop` (2%), `feat-wasm-core-tools` (1%).

---

## Current State (MEASURABLE)
- `crates/swal-core/src/wasm.rs`: stub from Ola 3.01 (`WasmLoop::new()`).
- Native `AgentLoop` exists in swal-loop (Ola 1 #07) — but swal-core is the wasm-clean crate.

## Desired State (DELTA)
Replace stub in `crates/swal-core/src/wasm.rs` with a REAL wasm loop:
- `WasmLoop` struct: holds `Arc<dyn WasmProvider>`, `Vec<WasmTool>`, `max_steps: usize`
- `WasmProvider` trait (wasm-clean, no tokio): `async fn complete(&self, messages: &[WasmMessage]) -> Result<WasmResponse, String>` — use `wasm_bindgen_futures::future_to_promise` bridging where needed
- `WasmTool { name: String, handler: Box<dyn Fn(serde_json::Value) -> Result<serde_json::Value, String> + Send> }`
- `WasmLoop::run(&self, task: &str) -> Result<WasmOutput, String>`: synchronous loop (no tokio) — iterate tool_calls with async provider via `wasm_bindgen_futures`
- Serde types: `WasmMessage { role, content }`, `WasmResponse { content, tool_calls }`, `WasmOutput { content, steps }`
- Unit test (native run, `cargo test -p swal-core`): scripted provider returns tool_call then final → loop executes.

## 🌐 Web Research Required
1. search: "wasm-bindgen-futures future_to_promise async fn rust"
2. search: "wasm32 no tokio async loop pattern"
3. search: "serde_json wasm32 no_std"

## Problem
The browser needs the loop without tokio/fs/process. This is the wasm-clean loop core (feat-wasm-core-loop), reusing the same conversation pattern as native AgentLoop.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "struct WasmLoop" crates/swal-core/src/wasm.rs` >= 1
- [ ] `grep -c "future_to_promise\|wasm_bindgen_futures" crates/swal-core/src/wasm.rs` >= 1
- [ ] `grep -c "struct WasmProvider" crates/swal-core/src/wasm.rs` >= 1
- [ ] `cargo test -p swal-core wasm 2>&1 | grep "test result: ok"` — 1 match
- [ ] `cargo build -p swal-core --target wasm32-unknown-unknown` — success
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-core/src/wasm.rs` | stub (3.01) | Real WasmLoop + WasmProvider + tests | HIGH |

## DO NOT touch (Anti-Regression)
- `crates/swal-core/Cargo.toml` + `lib.rs` (3.01 owns), `src/tool.rs` (Ola 1)
- Other crates; `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **NO tokio** in this crate — wasm-clean is the whole point. Async via wasm-bindgen-futures only.
2. Tools are synchronous `Fn(Value) -> Result<Value>` handlers (browser has no subprocess).
3. Tests run on NATIVE target (`cargo test -p swal-core`) with scripted provider — no network.
4. Do NOT edit Cargo.toml (3.01 added wasm deps; if missing, comment on issue).

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-core wasm 2>&1 | tail -5
cargo build -p swal-core --target wasm32-unknown-unknown 2>&1 | tail -2
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** Ola 3.01 (scaffold + wasm deps)
- **Parallel with:** Ola 3.06/3.07/3.08 (tools-web impls), 3.10 (sync transport), 3.12 (worker) — disjoint files
- **Merge order within wave:** 8 of 12
- **Expected effort:** Large (4h+)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| wasm-bindgen-futures API differs | Adapt to installed version; keep native tests as source of truth |
| No tokio workaround needed | Use wasm-bindgen-futures::spawn_local or future_to_promise |
| Test fails | Fix logic; do NOT weaken assertions |
