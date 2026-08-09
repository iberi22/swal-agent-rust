# [Ola 1.10] swal-agent — CLI run command + config (real impl)

> Ola 1 — CLI. Labels: `ola1`, `wave-1` (NO `jules` yet).
> Feature: `feat-cli-run` (6% of scope).

---

## Current State (MEASURABLE)
- `crates/swal-agent/src/cli.rs`: stub from issue 09 — `pub async fn run() -> anyhow::Result<()> { Ok(()) }`.
- `crates/swal-agent/src/config.rs`: stub — empty `Config` struct.

## Desired State (DELTA)
- **`crates/swal-agent/src/config.rs`** (REAL):
  - `Config { model: String, provider: String, session_dir: PathBuf, max_steps: usize }` with `Default` (model="mock", provider="mock", session_dir="data/sessions", max_steps=10)
  - `Config::from_file(path) -> anyhow::Result<Config>`: reads TOML/JSON — prefer `serde_json` (JSON is in deps; TOML is NOT — use JSON or env only)
  - `Config::from_env() -> Config`: reads `SWAL_MODEL`, `SWAL_PROVIDER`, `SWAL_SESSION_DIR`, `SWAL_MAX_STEPS` env vars, falls back to defaults
  - `Config::load(args_config: Option<PathBuf>)`: CLI path > env > defaults
- **`crates/swal-agent/src/cli.rs`** (REAL):
  - clap `Cli` struct: `#[command(name="swal-agent", version)]`, subcommand `Run { task: String, #[arg(long)] config: Option<PathBuf> }` (start with only `run` — more subcommands later)
  - `pub async fn run() -> anyhow::Result<()>`: parse args → `Config::load` → print resolved config (debug) → call `session::start_session(&config).await?` (stub from issue 09 — session real in issue 11; if session module not yet real, call tools::register_defaults + a placeholder loop run)
  - Keep it minimal: `swal-agent run "task"` must parse, load config, and reach the loop wiring point (issue 12 wires the full loop — this issue proves CLI+config+parse pipeline with a printed confirmation).

## 🌐 Web Research Required
1. search: "clap 4 subcommand struct derive example"
2. search: "rust config from env vars default fallback"
3. search: "serde_json from_file PathBuf"

## Problem
Users need `swal-agent run "task"` with configurable model/provider/session dir. This issue makes the CLI + config layer real so issue 12 can wire the loop.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "struct Config" crates/swal-agent/src/config.rs` >= 1
- [ ] `grep -c "fn from_env" crates/swal-agent/src/config.rs` >= 1
- [ ] `grep -c "Run" crates/swal-agent/src/cli.rs` >= 1
- [ ] `SWAL_MODEL=test-model cargo run -p swal-agent -- run "hi" 2>&1 | grep -i "test-model"` — prints resolved model
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-agent/src/config.rs` | stub (09) | Real Config + from_file/from_env/load | MED |
| `crates/swal-agent/src/cli.rs` | stub (09) | clap Run subcommand + config resolution + wiring call | MED |

## DO NOT touch (Anti-Regression)
- `crates/swal-agent/Cargo.toml` + `main.rs` (issue 09), `src/session.rs` (11), `src/tools.rs` (12)
- `crates/swal-loop/`, `crates/swal-core/`, `crates/swal-store/` — read-only
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **No TOML**: `serde_json` is the only serialization dep — JSON config file (`.json` extension) or env vars.
2. **session::start_session may be a stub** if issue 11 hasn't merged — call it via `crate::session::...` anyway (compiles against stub), or guard with a comment. Do NOT edit session.rs.
3. clap 4 syntax: `#[derive(Parser)]`, `#[command(version, about)]`, `#[arg(long)]`.
4. `cargo run -p swal-agent -- run "hi"` must work from repo root.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty (2 files)
- [ ] PR contains >= 1 real source file
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
SWAL_MODEL=test-model cargo run -p swal-agent -- run "hi" 2>&1 | tail -5
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** #09 (scaffold + stubs)
- **Blocked by:** none
- **Parallel with:** #11 (session.rs), #12 (tools.rs) — disjoint files
- **Merge order within wave:** 8 of 12
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| clap parse errors | Verify clap 4 derive docs; adjust attribute syntax |
| session module not real yet | Call stub API; the full loop wiring happens in #12 |
| Config load order wrong | Implement CLI path > env > defaults explicitly and test each |
| Test fails | Fix logic; do NOT weaken assertions |
