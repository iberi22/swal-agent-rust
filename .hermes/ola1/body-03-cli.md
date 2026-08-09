# [Ola 1.03] swal-agent — CLI `run` command, config, session persistence

> Ola 1 — Core. Labels: `ola1`, `wave-1` (NO `jules` yet).

---

## Current State (MEASURABLE)
- `crates/swal-agent/src/main.rs` — prints "swal-agent skeleton — Wave 0" only.
- `crates/swal-agent/Cargo.toml` — no dependencies.
- Foundation (#1) provides: `swal-store` `SessionStore` (rusqlite), `swal-core` ToolRegistry.
- Loop (#2) provides: `swal-loop::AgentLoop`, `ProviderFactory`, skills loader.

## Desired State (DELTA)
- **`crates/swal-agent/src/config.rs`**: `Config` struct loaded from env + optional `swal-agent.toml`: `llm.provider` (openrouter|mock), `llm.model`, `llm.api_key_env` (default `OPENROUTER_API_KEY`), `skills_dir` (default `skills/`), `data_dir` (default `data/`). Clap-derive optional, keep minimal.
- **`crates/swal-agent/src/run.rs`**: `run_command(task: &str, config: &Config)` — open `SessionStore` (data_dir/swal-agent.db), create session, build `AgentLoop` (ProviderFactory + skills loader + ToolRegistry with gestalt tools: shell, read, write, git), run loop, persist session + messages, print final answer to stdout, print tool trace to stderr (for TUI later).
- **`crates/swal-agent/src/main.rs`**: clap CLI: `swal-agent run "<task>"` → run_command; `--version`; error handling (exit code 1 on loop failure, message to stderr).
- **New test**: `crates/swal-agent/tests/cli_test.rs` — `swal-agent run "say hi"` with `SWAL_LLM_PROVIDER=mock` exits 0, prints a non-empty answer; session row exists in DB.

## 🌐 Web Research Required
1. search: "clap derive subcommand example 2026"
2. search: "rusqlite query and print row values"
3. search: "rust env var config pattern from_env"
4. search: "exit codes CLI conventions rust"

## 🔬 Agent Session Prompt
"Before implementing:
1. Read `docs/ARCHITECTURE.md` — swal-agent is the bin that wires everything.
2. Read `crates/swal-store/src/session.rs` (from #1) — exact `SessionStore` API.
3. Read `crates/swal-loop/src/lib.rs` (from #2) — exact `AgentLoop::run` signature and `ProviderFactory`.
4. Check how gestalt tools are constructed (`create_gestalt_tools` in gestalt_core) — wire them into the ToolRegistry if API permits; otherwise register the loop's own shell/read/write/git tools behind the same `swal-core::Tool` trait.
5. Document findings, then implement."

## Existing Code Patterns
- Workspace manifests use `version.workspace = true`.
- Tests use MockProvider (`SWAL_LLM_PROVIDER=mock`) — never real API keys.
- stdout = answer, stderr = diagnostics.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `cargo check -p swal-agent` — 0 errors
- [ ] `SWAL_LLM_PROVIDER=mock cargo run -p swal-agent -- run "say hi"` — exits 0, non-empty stdout
- [ ] `cargo test -p swal-agent 2>&1 | grep "test result: ok"` — 1 match
- [ ] `grep -c "fn run_command" crates/swal-agent/src/run.rs` >= 1
- [ ] `grep -c "struct Config" crates/swal-agent/src/config.rs` >= 1
- [ ] Session persisted: `sqlite3 data/swal-agent.db "select count(*) from sessions"` >= 1 after a run
- [ ] `git show HEAD --name-only | grep -cE "crates/swal-agent/src/(main|run|config)\.rs"` >= 3

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-agent/Cargo.toml` | no deps | Add clap (derive), tokio, swal-loop (path), swal-store (path), swal-core (path), serde, anyhow | MED |
| `crates/swal-agent/src/main.rs` | 3-line print | clap CLI + run dispatch + error handling | MED |
| `crates/swal-agent/src/config.rs` | — | NEW: `Config` from env + toml | MED |
| `crates/swal-agent/src/run.rs` | — | NEW: `run_command` wiring loop + store | HIGH |
| `crates/swal-agent/tests/cli_test.rs` | — | NEW: mock end-to-end CLI test | LOW |

## DO NOT touch (Anti-Regression)
- `docs/*.md`, `README.md` — canonical
- `crates/swal-loop/`, `crates/swal-core/`, `crates/swal-store/` — other islands (merge 1, 2 first)
- `crates/swal-gateway/`, `crates/swal-sched/` — Wave 2
- Web crates (swal-sync, swal-tools-*) — Wave 3
- Root `Cargo.toml` workspace section

## Anti-Hallucination Guard ⚠️
1. **READ before write**: read swal-store and swal-loop public APIs before wiring.
2. **Mock end-to-end**: the CLI must work fully with `SWAL_LLM_PROVIDER=mock` — if it needs a real API key to run, it's a FAILURE.
3. **No panics on user error**: bad args/config → clean error message + exit code 1.
4. **Persistence is real**: session rows must actually be written (verify with sqlite3 query, not just code reading).
5. **stdout/stderr discipline**: answer to stdout only; trace/logs to stderr.
6. Empty PRs are forbidden — verify `git diff --stat HEAD` before push.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git status --porcelain` shows the new/modified files BEFORE opening the PR
- [ ] `git diff --stat HEAD` lists the files (NOT empty)
- [ ] The PR MUST contain >= 1 source file: verify with `git ls-files` before push
- [ ] If the work could not be completed: DO NOT open a PR — comment the blocker on the issue

## Verification
```bash
SWAL_LLM_PROVIDER=mock cargo run -p swal-agent -- run "say hi"
cargo test -p swal-agent 2>&1 | grep "test result: ok"
sqlite3 data/swal-agent.db "select count(*) from sessions"
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
```

## Dependencies & Merge Order
- **Depends on:** #1 (swal-store, swal-core), #2 (swal-loop)
- **Blocked by:** #1, #2
- **Parallel with:** none
- **Merge order within wave:** 3 of 3
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| gestalt tools API doesn't fit | Fall back: implement minimal shell/read/write/git tools behind `swal-core::Tool` in this crate (documented in the PR) |
| sqlite3 CLI not available for verification | Use `cargo test` session assertions instead; note it in the PR |
| clap version conflicts | Pin clap = "4" |
| PR conflicts | Rebase on main, re-run verification |
