# [Ola 3.01] swal-core — wasm32 scaffold (lib.rs cfg, Cargo.toml, stubs)

> Ola 3 — Web. Labels: `ola3`, `wave-3` (NO `jules` yet).
> Feature: `feat-wasm-core-loop`, `feat-wasm-core-tools`.

---

## Current State (MEASURABLE)
- `crates/swal-core/src/lib.rs`: `pub mod tool;` (from Ola 1 #02), tool.rs with Tool trait + ToolRegistry.
- `crates/swal-core/Cargo.toml`: serde, schemars, dashmap, async-trait (Ola 1).

## Desired State (DELTA)
- **`crates/swal-core/Cargo.toml`**: add wasm-compatible deps (this issue owns manifest):
  - `wasm-bindgen`, `wasm-bindgen-futures` (only for wasm32 target — use `[target.'cfg(target_arch = "wasm32")'.dependencies]`)
  - `getrandom` with `js` feature (wasm32) if needed by any dep
- **`crates/swal-core/src/lib.rs`**: APPEND (do NOT remove) `#[cfg(target_arch = "wasm32")] pub mod wasm;` + keep existing mods.
- **`crates/swal-core/src/wasm.rs`** (NEW stub — real in Ola 3.02):
  ```rust
  pub struct WasmLoop;
  impl WasmLoop { pub fn new() -> Self { Self } }
  ```
- Verify: `cargo build -p swal-core --target wasm32-unknown-unknown` compiles (needs `rustup target add wasm32-unknown-unknown`).

## 🌐 Web Research Required
1. search: "wasm-bindgen-futures async rust 2026"
2. search: "cargo target cfg wasm32 dependencies"
3. search: "rustup target add wasm32-unknown-unknown"

## Problem
The same loop must run in the browser. This issue makes swal-core wasm32-compilable with the module skeleton for the wasm loop.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "wasm32" crates/swal-core/src/lib.rs` >= 1
- [ ] `ls crates/swal-core/src/wasm.rs` — exists
- [ ] `rustup target list --installed | grep wasm32` — target installed (or documented install)
- [ ] `cargo build -p swal-core --target wasm32-unknown-unknown` — success (stub level)
- [ ] `cargo check -p swal-core` (native) — 0 errors (existing tests still pass)

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-core/Cargo.toml` | Ola 1 deps | Add wasm32 target deps | MED |
| `crates/swal-core/src/lib.rs` | mod tool | Append `#[cfg(wasm32)] mod wasm;` | LOW |
| `crates/swal-core/src/wasm.rs` | — | NEW stub | LOW |

## DO NOT touch (Anti-Regression)
- `crates/swal-core/src/tool.rs` — read-only (Ola 1)
- Other crates; root `Cargo.toml` profiles; `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **APPEND only** to lib.rs — existing `pub mod tool;` must stay (Ola 1 tests depend on it).
2. wasm deps go in `[target.'cfg(target_arch = "wasm32")'.dependencies]` — NOT global.
3. Stub compiles for BOTH targets (use `#[cfg(target_arch = "wasm32")]` on the mod).
4. If wasm32 target missing: `rustup target add wasm32-unknown-unknown` — document if unavailable.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty (3 files)
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
rustup target add wasm32-unknown-unknown 2>/dev/null
cargo build -p swal-core --target wasm32-unknown-unknown 2>&1 | tail -3
cargo test -p swal-core 2>&1 | grep "test result: ok"
```

## Dependencies & Merge Order
- **Depends on:** Ola 1 complete (swal-core tool.rs exists)
- **Parallel with:** Ola 3.05 (tools-web scaffold), 3.09 (sync scaffold), 3.11 (pwa scaffold) — different crates
- **Merge order within wave:** 1 of 12
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| wasm32 target build fails on deps | Check each dep has wasm support; pin versions with wasm |
| lib.rs conflict | Append-only edit; do not reorder |
| Stub doesn't compile native | Guard with `#[cfg(target_arch = "wasm32")]` |
