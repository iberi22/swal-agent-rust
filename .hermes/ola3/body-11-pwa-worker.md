# [Ola 3.11] swal-pwa — Comlink worker + WebLLM (real impl)

> Ola 3 — Web. Labels: `ola3`, `wave-3` (NO `jules` yet).
> Feature: `feat-pwa-comlink` (1%), `feat-pwa-webllm` (1%).

---

## Current State (MEASURABLE)
- `crates/swal-pwa/src/worker.rs`: stub from Ola 3.10 (`run_task` returning empty String).
- `crates/swal-pwa/src/app.rs`: Leptos App shell calling `worker::run_task`.

## Desired State (DELTA)
Replace stub in `crates/swal-pwa/src/worker.rs` with REAL Comlink worker + WebLLM:
- `run_task(task: &str) -> String`: 
  1. Sets up a Comlink-exposed worker API (if comlink JS lib available via js-sys interop; else document the pattern) — the loop runs OFF the main thread
  2. Instantiates `WasmLoop` (swal-core wasm.rs, Ola 3.02) with a WebLLM provider:
     - WebLLM (MLC-LLM → WASM + WebGPU): `webllm.CreateMLCEngine({model: "..."})` via js_sys — OpenAI-style API
     - Fallback: OpenAI-compatible remote endpoint (configurable URL) — same `WasmProvider` trait
  3. Returns final output content
- `WasmProvider` impl for WebLLM: `complete(messages)` → call engine.chat.completions.create (js_sys interop), parse response
- Worker glue: `#[cfg(target_arch = "wasm32")]` + wasm-bindgen export (`#[wasm_bindgen] pub fn run_task_js(task: String) -> String`)
- Test: provider response parsing unit test (mock WebLLM response JSON → parsed correctly); native compile check.

## 🌐 Web Research Required
1. search: "WebLLM MLC-LLM CreateMLCEngine js api 2026"
2. search: "comlink worker postMessage rust wasm"
3. search: "wasm_bindgen export function string js"

## Problem
The PWA must run the loop off the main thread (feat-pwa-comlink) with in-browser inference (feat-pwa-webllm) or a remote OpenAI-compatible fallback. This is the worker + provider.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "run_task" crates/swal-pwa/src/worker.rs` >= 1
- [ ] `grep -c "WebLLM\|MLCEngine\|openai" crates/swal-pwa/src/worker.rs` >= 1
- [ ] `grep -c "wasm_bindgen" crates/swal-pwa/src/worker.rs` >= 1
- [ ] `cargo check -p swal-pwa` — 0 errors (native)
- [ ] `cargo test -p swal-pwa 2>&1 | grep "test result: ok"` — 1 match (parsing test)

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-pwa/src/worker.rs` | stub (3.10) | Real Comlink worker + WebLLM provider + test | HIGH |

## DO NOT touch (Anti-Regression)
- `crates/swal-pwa/Cargo.toml` + `lib.rs` + `src/app.rs` (3.10 owns — import worker only)
- Other crates; `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **WebLLM is a JS API** — bridge via js_sys/web_sys; don't reimplement inference.
2. wasm-gate real calls; native fallback so cargo check passes.
3. Test = response JSON parsing with a mock — NO real WebLLM in tests (needs GPU/browser).
4. No new Cargo.toml deps unless already in 3.10 manifest; comment if missing.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo check -p swal-pwa 2>&1 | tail -2
cargo test -p swal-pwa 2>&1 | tail -4
```

## Dependencies & Merge Order
- **Depends on:** Ola 3.10 (scaffold), Ola 3.02 (WasmLoop)
- **Parallel with:** 3.04, 3.05, 3.06, 3.07, 3.09 — disjoint
- **Merge order within wave:** 8 of 12
- **Expected effort:** Large (4h+)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| WebLLM API differs | Adapt to installed JS API; document |
| comlink interop issues | js_sys Function/Reflect; document pattern |
| Test fails | Fix parsing logic; do NOT weaken assertions |
