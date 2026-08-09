# [Ola 3.05] swal-tools-web — git tools via isomorphic-git (real impl)

> Ola 3 — Web. Labels: `ola3`, `wave-3` (NO `jules` yet).
> Feature: `feat-tools-git-web` (1%).

---

## Current State (MEASURABLE)
- `crates/swal-tools-web/src/git.rs`: stub from Ola 3.03 (`clone_repo`/`commit_all` returning Err/Ok).

## Desired State (DELTA)
Replace stub in `crates/swal-tools-web/src/git.rs` with REAL git tools via isomorphic-git (JS interop):
- Per REUSE-MAP: web git = isomorphic-git (JS), NOT git2 (native-only). Bridge via `js_sys`/`web_sys` calling the isomorphic-git JS API (window.__isomorphicGit or npm-loaded global).
- `pub async fn clone_repo(url: &str, dir: &str) -> Result<(), String>`: call `isomorphicGit.clone({url, dir, fs})` via js_sys::Function/Reflect
- `pub async fn commit_all(msg: &str) -> Result<(), String>`: `isomorphicGit.add` + `isomorphicGit.commit`
- `pub async fn status() -> Result<Vec<String>, String>`: `isomorphicGit.statusMatrix`
- `pub async fn push(remote: &str) -> Result<(), String>`: `isomorphicGit.push`
- ⚠️ wasm-gated (`#[cfg(target_arch = "wasm32")]`); native fallback returns `Err("isomorphic-git unavailable on native")`.
- Unit test (native): fallback Err graceful; wasm tests cfg-gated.

## 🌐 Web Research Required
1. search: "isomorphic-git clone fs browser js 2026"
2. search: "js_sys Function call_with_this Reflect rust"
3. search: "isomorphic-git statusMatrix push"

## Problem
The browser agent needs git operations without a native git lib. isomorphic-git runs in JS; this bridges it to Rust.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "async fn clone_repo" crates/swal-tools-web/src/git.rs` >= 1
- [ ] `grep -c "isomorphic" crates/swal-tools-web/src/git.rs` >= 1
- [ ] `grep -c "js_sys\|web_sys\|Reflect" crates/swal-tools-web/src/git.rs` >= 1
- [ ] `cargo check -p swal-tools-web` — 0 errors
- [ ] `cargo test -p swal-tools-web git 2>&1 | grep "test result: ok"` — 1 match (fallback)

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-tools-web/src/git.rs` | stub (3.03) | Real isomorphic-git bridge + fallback test | HIGH |

## DO NOT touch (Anti-Regression)
- `crates/swal-tools-web/Cargo.toml` + `lib.rs` (3.03), `src/opfs.rs` (3.04), `src/shell.rs` (3.06)
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **isomorphic-git is JS** — you are bridging, not implementing git. Use js_sys/web_sys interop; do NOT write a git implementation.
2. wasm-gate real calls; native fallback Err so cargo check passes.
3. No new deps unless in 3.03 manifest; if `js-sys` missing, comment.
4. Tests: native fallback only.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo check -p swal-tools-web 2>&1 | tail -2
cargo test -p swal-tools-web git 2>&1 | tail -4
```

## Dependencies & Merge Order
- **Depends on:** Ola 3.03 (scaffold)
- **Parallel with:** 3.04 (opfs.rs), 3.06 (shell.rs), 3.02, 3.10, 3.12 — disjoint
- **Merge order within wave:** 8 of 12
- **Expected effort:** Large (4h+)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| js_sys call API differs | Use Reflect::get/set/call_with_args; verify against js-sys docs |
| isomorphic-git global missing | Document expected global name; use `window.__isomorphicGit` convention |
| Native check fails | cfg(wasm32) gate |
