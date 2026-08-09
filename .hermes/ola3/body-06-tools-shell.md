# [Ola 3.06] swal-tools-web — shell via WebContainers (real impl)

> Ola 3 — Web. Labels: `ola3`, `wave-3` (NO `jules` yet).
> Feature: `feat-tools-shell-web` (1%).

---

## Current State (MEASURABLE)
- `crates/swal-tools-web/src/shell.rs`: stub from Ola 3.03 (`run_cmd` returning Err).

## Desired State (DELTA)
Replace stub in `crates/swal-tools-web/src/shell.rs` with REAL WebContainers shell:
- `pub async fn run_cmd(cmd: &str) -> Result<String, String>`: bridge to WebContainers API (`WebContainer.boot()`, `container.spawn("bash", ["-c", cmd])`, collect stdout) via js_sys/web_sys interop
- `pub async fn run_cmd_in(workdir: &str, cmd: &str) -> Result<String, String>`: spawn with cwd
- `pub fn is_available() -> bool`: checks `window.WebContainer` exists
- ⚠️ **NO subprocess** in web — WebContainers is the sandboxed shell. wasm-gated; native fallback `Err("WebContainers unavailable on native")`.
- Unit test (native): fallback Err + `is_available() == false` on native.

## 🌐 Web Research Required
1. search: "WebContainers API spawn bash rust bridge 2026"
2. search: "webcontainers boot spawn stdout js"
3. search: "js_sys Promise await async interop rust"

## Problem
The browser agent needs a shell — WebContainers provides it in-browser (no subprocess). This bridges Rust to it.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "async fn run_cmd" crates/swal-tools-web/src/shell.rs` >= 1
- [ ] `grep -c "WebContainer" crates/swal-tools-web/src/shell.rs` >= 1
- [ ] `grep -c "fn is_available" crates/swal-tools-web/src/shell.rs` >= 1
- [ ] `cargo check -p swal-tools-web` — 0 errors
- [ ] `cargo test -p swal-tools-web shell 2>&1 | grep "test result: ok"` — 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-tools-web/src/shell.rs` | stub (3.03) | Real WebContainers bridge + fallback test | HIGH |

## DO NOT touch (Anti-Regression)
- `crates/swal-tools-web/Cargo.toml` + `lib.rs` (3.03), `src/opfs.rs` (3.04), `src/git.rs` (3.05)
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **WebContainers is a JS API** — bridge it, don't reimplement. `WebContainer.boot()` returns a Promise; await via wasm-bindgen-futures.
2. **NO subprocess tool** in web target (explicit non-goal) — document this in code.
3. wasm-gate; native fallback Err.
4. No new deps unless in 3.03 manifest.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo check -p swal-tools-web 2>&1 | tail -2
cargo test -p swal-tools-web shell 2>&1 | tail -4
```

## Dependencies & Merge Order
- **Depends on:** Ola 3.03 (scaffold)
- **Parallel with:** 3.04, 3.05, 3.02, 3.10, 3.12 — disjoint
- **Merge order within wave:** 8 of 12
- **Expected effort:** Large (4h+)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| Promise interop fails | js_sys::Promise + wasm_bindgen_futures::JsFuture |
| WebContainer global absent | is_available() false; document |
| Native check fails | cfg(wasm32) gate |
