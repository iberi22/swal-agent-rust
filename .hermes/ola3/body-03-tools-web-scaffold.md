# [Ola 3.03] swal-tools-web — crate scaffold (lib.rs, Cargo.toml, stubs)

> Ola 3 — Web. Labels: `ola3`, `wave-3` (NO `jules` yet).
> Features: scaffolding for `feat-tools-opfs`, `feat-tools-git-web`, `feat-tools-shell-web`.

---

## Current State (MEASURABLE)
- `crates/swal-tools-web/Cargo.toml`: empty `[dependencies]`.
- `crates/swal-tools-web/src/lib.rs`: 1-line doc comment only.

## Desired State (DELTA)
- **`crates/swal-tools-web/Cargo.toml`**: add deps (this issue owns manifest):
  - `wasm-bindgen`, `wasm-bindgen-futures`, `serde`, `serde_json`, `js-sys`, `web-sys` (File, Blob, Window, Worker features)
- **`crates/swal-tools-web/src/lib.rs`**: `pub mod opfs; pub mod git; pub mod shell;` + doc.
- **Stubs** (compilable — real in 3.06/3.07/3.08):
  - `opfs.rs`: `pub fn read_file(_path: &str) -> Result<String, String> { Err("not implemented".into()) }` + `pub fn write_file(_path: &str, _content: &str) -> Result<(), String> { Ok(()) }`
  - `git.rs`: `pub fn clone_repo(_url: &str, _dir: &str) -> Result<(), String> { Err("not implemented".into()) }` + `pub fn commit_all(_msg: &str) -> Result<(), String> { Ok(()) }`
  - `shell.rs`: `pub fn run_cmd(_cmd: &str) -> Result<String, String> { Err("not implemented".into()) }`
- Workspace compiles (native target; wasm target best-effort).

## 🌐 Web Research Required
1. search: "web-sys crate features File Blob Worker 2026"
2. search: "wasm-bindgen js-sys interop rust"
3. search: "rust wasm32 crate target dependencies"

## Problem
The web tools (OPFS/git/shell) need their crate skeleton. Stubs compile so implementation issues land on disjoint files in parallel.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `ls crates/swal-tools-web/src/{opfs,git,shell}.rs` — all exist
- [ ] `grep -c "pub mod" crates/swal-tools-web/src/lib.rs` >= 3
- [ ] `cargo check -p swal-tools-web` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-tools-web/Cargo.toml` | empty | Add wasm-bindgen/web-sys/serde | MED |
| `crates/swal-tools-web/src/lib.rs` | doc only | 3 `pub mod` | LOW |
| `crates/swal-tools-web/src/opfs.rs` | — | NEW stub | LOW |
| `crates/swal-tools-web/src/git.rs` | — | NEW stub | LOW |
| `crates/swal-tools-web/src/shell.rs` | — | NEW stub | LOW |

## DO NOT touch (Anti-Regression)
- Other crates; root `Cargo.toml` profiles; `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **Stubs compile on native** — web-sys APIs may not compile on native; keep stubs to pure Rust (no web-sys calls in stubs).
2. lib.rs declares modules; files exist as stubs so `cargo check` passes.
3. No `unsafe` in stubs.
4. Manifest: wasm-bindgen versions must match workspace (check root workspace.dependencies if present).

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty (5 files)
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo check -p swal-tools-web 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** Ola 1 complete (workspace deps)
- **Parallel with:** Ola 3.01 (core wasm), 3.09 (sync), 3.11 (pwa) — different crates
- **Merge order within wave:** 1 of 12
- **Expected effort:** Small (<1h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| web-sys doesn't compile native | Stubs avoid web-sys calls; put real calls behind #[cfg(target_arch="wasm32")] in impl issues |
| Version mismatch | Align with workspace deps; comment if conflict |
| Warnings | `#[allow(dead_code)]` on stub fns |
