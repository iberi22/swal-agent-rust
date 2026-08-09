# [Ola 1.02] swal-core — Tool trait + ToolRegistry (wasm32-clean)

> Ola 1 — Foundation. Labels: `ola1`, `wave-1` (NO `jules` yet).
> Feature: `feat-tool-registry` (8% of scope).

---

## Current State (MEASURABLE)
- `crates/swal-core/Cargo.toml`: empty `[dependencies]` (only package metadata).
- `crates/swal-core/src/lib.rs`: 3-line doc comment only. No `Tool` trait, no registry.
- Workspace compiles; swal-core has zero deps.

## Desired State (DELTA)
- **`crates/swal-core/Cargo.toml`**: add deps (use `workspace = true` for git deps if defined in root, else direct):
  - `serde` (derive), `serde_json`, `schemars`, `dashmap`, `async-trait`
- **`crates/swal-core/src/tool.rs`** (NEW):
  - `pub trait Tool: Send + Sync`: `fn name(&self) -> &str`, `fn description(&self) -> &str`, `fn input_schema(&self) -> schemars::schema::RootSchema`, `async fn execute(&self, args: Value) -> Result<Value, ToolError>`
  - `ToolError` enum (Serialization/Execution/NotFound)
  - `ToolRegistry` (DashMap<String, Arc<dyn Tool>>): `register`, `list` (sorted names), `execute(name, args)`
- **`crates/swal-core/src/lib.rs`**: `pub mod tool;` + doc comment.
- **Test** `crates/swal-core/tests/tool_test.rs` (NEW): register 2 tools, list returns both, execute returns result.
- ⚠️ **wasm32-clean**: NO `std::process`, NO tokio, NO fs imports anywhere in this crate.

## 🌐 Web Research Required
1. search: "schemars JsonSchema derive struct enum RootSchema"
2. search: "dashmap Arc dyn Trait registry pattern rust"
3. search: "async-trait trait object Send Sync rust 2026"

## Problem
The agent loop needs a uniform way to register/list/execute tools (terminal, file, git, MCP later). Without a platform-agnostic `Tool` trait, native and wasm targets would diverge.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "pub trait Tool" crates/swal-core/src/tool.rs` >= 1
- [ ] `grep -c "struct ToolRegistry" crates/swal-core/src/tool.rs` >= 1
- [ ] `grep -cE "std::process|tokio" crates/swal-core/src/*.rs` == 0 (wasm32-clean)
- [ ] `cargo test -p swal-core 2>&1 | grep "test result: ok"` — 1 match
- [ ] `cargo check -p swal-core` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-core/Cargo.toml` | empty deps | Add serde/schemars/dashmap/async-trait | LOW |
| `crates/swal-core/src/lib.rs` | doc only | `pub mod tool;` | LOW |
| `crates/swal-core/src/tool.rs` | — | NEW: Tool trait + ToolError + ToolRegistry | LOW |
| `crates/swal-core/tests/tool_test.rs` | — | NEW: register/list/execute test | LOW |

## DO NOT touch (Anti-Regression)
- Root `Cargo.toml` `[profile.*]` sections; root `[workspace.dependencies]` (issue 01 owns it)
- `crates/swal-store/`, `crates/swal-loop/`, `crates/swal-agent/` — other file islands
- `docs/`, `.gitcore/features.json`
- Do NOT modify any other crate's manifest

## Anti-Hallucination Guard ⚠️
1. **Verify crate exists before use**: run `cargo search schemars` or check crates.io for current versions (schemars ~0.8.x, dashmap ~6.x, async-trait ~0.1.x).
2. **No tokio/process/fs** in swal-core — this crate must compile to wasm32 later. If you need async, use `async-trait` only.
3. **DashMap is not std HashMap** — use `.insert()`, `.get()`, iterate with `.iter()` (returns RefMulti).
4. Tests must NOT need network or API keys.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty before PR
- [ ] PR contains >= 1 real source file
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-core 2>&1 | tail -5
cargo check -p swal-core 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** #01 (workspace git deps exist — harmless if not, this crate has no git deps)
- **Blocked by:** none
- **Parallel with:** #03 (swal-store — different crate, disjoint island)
- **Merge order within wave:** 2 of 12
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| schemars version mismatch | Use latest stable on crates.io; adjust `input_schema` return type accordingly |
| DashMap borrow errors | Clone values out of the map (`map.get(&k).map(|v| v.clone())`) |
| async-trait dyn issues | Add `Send + Sync` bounds on trait and `#[async_trait]` macro |
| Test fails | Fix test logic or implementation; do NOT weaken assertions |
