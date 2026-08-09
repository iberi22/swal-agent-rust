# [Ola 1.04] swal-loop — crate scaffold (lib.rs, Cargo.toml, compilable stubs)

> Ola 1 — Core. Labels: `ola1`, `wave-1` (NO `jules` yet).
> Features: scaffolding for `feat-agent-loop`, `feat-llm-providers`, `feat-skills-loader`.

---

## Current State (MEASURABLE)
- `crates/swal-loop/Cargo.toml`: empty `[dependencies]`.
- `crates/swal-loop/src/lib.rs`: doc comment only.
- No `provider.rs`, `skills.rs`, `loop.rs`, no tests dir.

## Desired State (DELTA)
- **`crates/swal-loop/Cargo.toml`**: add ALL deps the crate will need (this issue owns the manifest — later loop issues must NOT touch it):
  - `swal-core` (path = "../swal-core")
  - `gestalt_core = { git = ".../gestalt.git", package = "gestalt_core" }` (workspace = true if defined)
  - `synapse-agentic` (git, workspace = true if defined)
  - `serde`, `serde_json`, `async-trait`, `tokio` (rt-multi-thread, macros, time), `thiserror`
- **`crates/swal-loop/src/lib.rs`** (NEW content): `pub mod provider; pub mod skills; pub mod r#loop;` + crate doc.
- **`crates/swal-loop/src/provider.rs`** (NEW, MINIMAL STUB — real impl in issue 05):
  ```rust
  #[async_trait::async_trait]
  pub trait Provider: Send + Sync {
      async fn complete(&self, prompt: &str) -> Result<String, String>;
  }
  pub struct MockProvider;
  #[async_trait::async_trait]
  impl Provider for MockProvider {
      async fn complete(&self, _prompt: &str) -> Result<String, String> { Ok(String::new()) }
  }
  ```
- **`crates/swal-loop/src/skills.rs`** (NEW, MINIMAL STUB — real impl in issue 06):
  ```rust
  pub fn load_skills(_dir: &str) -> Vec<String> { Vec::new() }
  ```
- **`crates/swal-loop/src/loop.rs`** (NEW, MINIMAL STUB — real impl in issue 07):
  ```rust
  pub struct AgentLoop;
  impl AgentLoop { pub fn new() -> Self { Self } }
  ```
- Workspace compiles with these stubs.

## 🌐 Web Research Required
1. search: "cargo path dependency sibling crate workspace"
2. search: "async_trait 0.1 rust 2026"
3. search: "tokio features rt-multi-thread macros time"

## Problem
swal-loop is the core crate of Ola 1 (loop, providers, skills). Its manifest + module skeleton must exist and compile before the three implementation issues can land in parallel on disjoint files.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `ls crates/swal-loop/src/provider.rs crates/swal-loop/src/skills.rs crates/swal-loop/src/loop.rs` — all exist
- [ ] `grep -c "pub mod" crates/swal-loop/src/lib.rs` >= 3
- [ ] `grep -c "swal-core" crates/swal-loop/Cargo.toml` >= 1
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-loop/Cargo.toml` | empty deps | Add swal-core path dep + git deps + serde/tokio | MED |
| `crates/swal-loop/src/lib.rs` | doc only | 3 `pub mod` + doc | LOW |
| `crates/swal-loop/src/provider.rs` | — | NEW stub (trait + MockProvider skeleton) | LOW |
| `crates/swal-loop/src/skills.rs` | — | NEW stub (`load_skills` returning vec![]) | LOW |
| `crates/swal-loop/src/loop.rs` | — | NEW stub (`AgentLoop` struct) | LOW |

## DO NOT touch (Anti-Regression)
- `crates/swal-loop/src/provider.rs|skills.rs|loop.rs` beyond the stubs above — issues 05/06/07 own the REAL implementations (replace stub content)
- `crates/swal-core/`, `crates/swal-store/`, `crates/swal-agent/` — other islands
- Root `Cargo.toml` profiles
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **Stubs only** — do NOT implement real provider/skills/loop logic here; that's issues 05-07. Keep stubs compilable and minimal.
2. **Verify `swal-core` package name** is `swal-core` (it is — check its Cargo.toml).
3. **Do NOT add a second `gestalt` git dep** if the root workspace already defines it — use `workspace = true` if present.
4. Stub functions must compile with zero warnings (`#[allow(dead_code)]` if needed).

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty (5 files)
- [ ] PR contains >= 1 real source file
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo check --workspace 2>&1 | tail -3
cargo test -p swal-loop 2>&1 | tail -3
```

## Dependencies & Merge Order
- **Depends on:** #01 (git deps available), #02 (swal-core Tool trait — path dep must compile)
- **Blocked by:** none
- **Parallel with:** none (owns lib.rs of swal-loop — subsequent loop issues need this merged)
- **Merge order within wave:** 3 of 12
- **Expected effort:** Small (<1h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| gestalt git dep fails to resolve | Check `[workspace.dependencies]` from issue 01 is merged; `cargo update` |
| swal-core path dep broken | Verify `crates/swal-core` compiles standalone first |
| Stub warnings | Add `#[allow(dead_code)]` on stub items |
