# [Ola 1.09] swal-agent — crate scaffold (main.rs, Cargo.toml, compilable stubs)

> Ola 1 — CLI. Labels: `ola1`, `wave-1` (NO `jules` yet).
> Features: scaffolding for `feat-cli-run` + `feat-native-tools`.

---

## Current State (MEASURABLE)
- `crates/swal-agent/Cargo.toml`: empty `[dependencies]`.
- `crates/swal-agent/src/main.rs`: empty `fn main() {}` (skeleton from Wave 0).

## Desired State (DELTA)
- **`crates/swal-agent/Cargo.toml`**: add ALL deps (this issue owns the manifest — later agent issues must NOT touch it):
  - `swal-core` (path = "../swal-core"), `swal-loop` (path = "../swal-loop"), `swal-store` (path = "../swal-store")
  - `gestalt_core` (git, workspace = true if defined), `synapse-agentic` (git, workspace = true if defined)
  - `serde`, `serde_json`, `clap` (derive), `tokio` (rt-multi-thread, macros), `anyhow`, `dirs`
- **`crates/swal-agent/src/main.rs`** (REAL, minimal but functional):
  - `mod cli; mod config; mod session; mod tools;` module declarations (stub files created below so it compiles)
  - `#[tokio::main] async fn main()` → calls `cli::run().await`, exits with error message on failure
- **Stub files** (minimal, compilable — REAL impls in issues 10-12):
  - `crates/swal-agent/src/cli.rs`: `pub async fn run() -> anyhow::Result<()> { Ok(()) }` + `#[allow(dead_code)]` helpers
  - `crates/swal-agent/src/config.rs`: `#[derive(Debug, Clone, Default)] pub struct Config;` (no fields yet)
  - `crates/swal-agent/src/session.rs`: `pub struct Session;`
  - `crates/swal-agent/src/tools.rs`: `pub fn register_defaults(_reg: &swal_core::tool::ToolRegistry) {}`
- Workspace compiles; `cargo run -p swal-agent` exits 0 silently.

## 🌐 Web Research Required
1. search: "clap derive 4.x Command arg subcommand 2026"
2. search: "tokio main async entrypoint rust"
3. search: "anyhow Result main error handling"

## Problem
swal-agent is the binary that wires everything (loop, store, tools, skills). Its manifest and module skeleton must exist and compile before issues 10-12 land on disjoint files.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `ls crates/swal-agent/src/{cli,config,session,tools}.rs` — all exist
- [ ] `grep -c "mod " crates/swal-agent/src/main.rs` >= 4
- [ ] `grep -c "swal-loop" crates/swal-agent/Cargo.toml` >= 1
- [ ] `cargo check --workspace` — 0 errors
- [ ] `cargo run -p swal-agent 2>&1` — exit code 0

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-agent/Cargo.toml` | empty deps | Add swal-core/loop/store path deps + clap/tokio/anyhow | MED |
| `crates/swal-agent/src/main.rs` | empty main | Module declarations + tokio main calling cli::run | LOW |
| `crates/swal-agent/src/cli.rs` | — | NEW stub (`run()` no-op) | LOW |
| `crates/swal-agent/src/config.rs` | — | NEW stub (empty Config) | LOW |
| `crates/swal-agent/src/session.rs` | — | NEW stub (empty Session) | LOW |
| `crates/swal-agent/src/tools.rs` | — | NEW stub (register_defaults no-op) | LOW |

## DO NOT touch (Anti-Regression)
- `crates/swal-loop/`, `crates/swal-core/`, `crates/swal-store/` — other islands (read-only)
- Root `Cargo.toml` profiles / workspace.dependencies
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **Stubs only** — real CLI/config/session/tools logic is issues 10-12. Keep stubs compilable, zero warnings (`#[allow(dead_code)]` where needed).
2. **Verify package names**: `swal-core`, `swal-loop`, `swal-store` are the path-dep names (check their Cargo.toml `name` fields).
3. **clap 4.x derive syntax** — verify against docs; `#[derive(Parser)]` on a `Cli` struct.
4. `tokio::main` requires `tokio` with `macros` + `rt-multi-thread` features.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty (6 files)
- [ ] PR contains >= 1 real source file
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo check --workspace 2>&1 | tail -3
cargo run -p swal-agent 2>&1; echo "exit=$?"
```

## Dependencies & Merge Order
- **Depends on:** #01 (git deps), #04 (swal-loop compiles), #02 (swal-core)
- **Blocked by:** none
- **Parallel with:** #08 (loop e2e tests — different crate)
- **Merge order within wave:** 7 of 12
- **Expected effort:** Small (<1h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| clap version API differs | Use clap 4 derive; if compile errors, adjust to documented 4.x API |
| path dep not found | Verify `crates/swal-*/Cargo.toml` names match exactly |
| Stub warnings | Add `#[allow(dead_code)]` |
