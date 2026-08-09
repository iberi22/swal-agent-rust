# [Ola 3.04] swal-tools-web — OPFS filesystem tools (real impl)

> Ola 3 — Web. Labels: `ola3`, `wave-3` (NO `jules` yet).
> Feature: `feat-tools-opfs` (1%).

---

## Current State (MEASURABLE)
- `crates/swal-tools-web/src/opfs.rs`: stub from Ola 3.03 (`read_file`/`write_file` returning Err/Ok).

## Desired State (DELTA)
Replace stub in `crates/swal-tools-web/src/opfs.rs` with REAL OPFS-backed fs tools:
- `pub async fn read_file(path: &str) -> Result<String, String>`: OPFS via `navigator.storage.getDirectory()` (web-sys / js-sys interop) — open file handle, read text
- `pub async fn write_file(path: &str, content: &str) -> Result<(), String>`: create/write file handle
- `pub async fn list_dir(path: &str) -> Result<Vec<String>, String>`: iterate directory handles
- `pub async fn delete_file(path: &str) -> Result<(), String>`
- ⚠️ Code is `#[cfg(target_arch = "wasm32")]`-gated where web APIs are used; native fallback returns `Err("OPFS unavailable on native")` so `cargo check` passes both targets.
- Unit test (native): fallback returns Err gracefully; wasm test marked `#[cfg(target_arch="wasm32")]` (may be skipped in CI).

## 🌐 Web Research Required
1. search: "OPFS FileSystemDirectoryHandle web-sys rust"
2. search: "navigator.storage.getDirectory wasm-bindgen"
3. search: "FileSystemFileHandle createWritable OPFS"

## Problem
The browser agent needs file tools over the Origin Private File System (no backend). This is feat-tools-opfs.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "async fn read_file" crates/swal-tools-web/src/opfs.rs` >= 1
- [ ] `grep -c "async fn write_file" crates/swal-tools-web/src/opfs.rs` >= 1
- [ ] `grep -c "getDirectory\|OPFS" crates/swal-tools-web/src/opfs.rs` >= 1
- [ ] `cargo check -p swal-tools-web` — 0 errors (native)
- [ ] `cargo test -p swal-tools-web opfs 2>&1 | grep "test result: ok"` — 1 match (native fallback test)

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-tools-web/src/opfs.rs` | stub (3.03) | Real OPFS impl (wasm-gated) + fallback test | HIGH |

## DO NOT touch (Anti-Regression)
- `crates/swal-tools-web/Cargo.toml` + `lib.rs` (3.03 owns), `src/git.rs` (3.07), `src/shell.rs` (3.08)
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **wasm-gate web APIs** — `#[cfg(target_arch = "wasm32")]` on the real impl; native fallback Err. cargo check MUST pass on native.
2. If web-sys lacks OPFS bindings (may be unstable), implement via `js_sys::Reflect`/raw `web_sys::window()` interop or document + minimal stub — do NOT invent APIs.
3. No new Cargo.toml deps unless already added in 3.03.
4. Tests: native fallback only; wasm tests cfg-gated.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo check -p swal-tools-web 2>&1 | tail -2
cargo test -p swal-tools-web opfs 2>&1 | tail -4
```

## Dependencies & Merge Order
- **Depends on:** Ola 3.03 (scaffold)
- **Parallel with:** Ola 3.07 (git.rs), 3.08 (shell.rs), 3.02, 3.10, 3.12 — disjoint files
- **Merge order within wave:** 8 of 12
- **Expected effort:** Large (4h+)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| web-sys lacks OPFS API | js_sys Reflect interop or documented minimal; do NOT fake |
| Native check fails | Gate web code with cfg(wasm32); fallback Err |
| Test fails | Fix logic; do NOT weaken assertions |
