# [Ola 1.01] foundation — deps, swal-core Tool trait, swal-store sessions, CI

> Ola 1 — Foundation. Labels: `ola1`, `wave-1` (NO `jules` yet).

---

## Current State (MEASURABLE)
- Workspace: `Cargo.toml` root with 9 empty crate skeletons (compiles in 0.18s), no dependencies.
- `crates/swal-core/src/lib.rs` — 3-line doc comment only (no `Tool` trait).
- `crates/swal-store/src/lib.rs` — empty; no `Store` trait, no session schema, no rusqlite.
- No `.github/` directory, no CI workflow.

## Desired State (DELTA)
- **Root `Cargo.toml`**: add git dependencies usable by all crates:
  - `gestalt_core = { git = "https://github.com/iberi22/gestalt.git", package = "gestalt_core" }`
  - `synapse-agentic = { git = "https://github.com/iberi22/synapse-agentic.git" }`
  (both public repos — do NOT vendor or copy code from them).
- **`crates/swal-core`**: add `Tool` trait (name, description, input JSON-Schema via `schemars`, async `execute` returning `ToolResult`), plus `ToolRegistry` (DashMap-based, register/list/execute). Platform-agnostic: NO tokio/fs/process imports.
- **`crates/swal-store`**: add `Store` trait (session CRUD) + `SessionStore` impl on rusqlite (SQLite file at `data/swal-agent.db`, WAL). Schema: `sessions(id TEXT PK, created_at INTEGER, updated_at INTEGER, summary TEXT)` and `messages(id INTEGER PK AUTOINCREMENT, session_id TEXT, role TEXT, content TEXT, ts INTEGER)`. Shared serde types.
- **`.github/workflows/ci.yml`**: on push/PR to main — `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace`.
- **New test**: `crates/swal-store/tests/session_test.rs` — create/append/read/delete session round-trip.

## 🌐 Web Research Required
1. search: "rusqlite Connection methods insert querying 2026"
2. search: "schemars JsonSchema derive struct enum"
3. search: "cargo git dependency workspace package rename"
4. search: "github actions rust workflow cargo fmt clippy test"

## 🔬 Agent Session Prompt
"Before implementing:
1. Read `docs/ARCHITECTURE.md` and `docs/REUSE-MAP.md` in this repo — the role of swal-core and swal-store is defined there.
2. Read `crates/swal-core/Cargo.toml` and `crates/swal-store/Cargo.toml` skeletons.
3. Check latest rusqlite/schemars versions on crates.io before pinning.
4. Document findings, then implement."

## Existing Code Patterns
- Empty crate skeleton pattern: each crate has its own `Cargo.toml` using `version.workspace = true` etc.
- Root `Cargo.toml` already defines `[profile.release]` with `lto = "thin"` — do not change profiles.
- Commit style: `type(scope): summary`.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `cargo check --workspace` — 0 errors
- [ ] `cargo test -p swal-store 2>&1 | grep "test result: ok"` — 1 match
- [ ] `grep -c "pub trait Tool" crates/swal-core/src/tool.rs` >= 1
- [ ] `grep -c "pub struct SessionStore" crates/swal-store/src/session.rs` >= 1
- [ ] `ls .github/workflows/ci.yml` — exists
- [ ] `git show HEAD --name-only | grep -cE "crates/(swal-core|swal-store)/src"` >= 2 (real source, not just manifests)

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `Cargo.toml` | workspace only | Add `[workspace.dependencies]` git deps (gestalt_core, synapse-agentic) | MED |
| `crates/swal-core/Cargo.toml` | no deps | Add serde, serde_json, schemars, dashmap, async-trait | LOW |
| `crates/swal-core/src/lib.rs` | doc only | Export `tool` module | LOW |
| `crates/swal-core/src/tool.rs` | — | NEW: `Tool` trait + `ToolRegistry` | LOW |
| `crates/swal-store/Cargo.toml` | no deps | Add rusqlite (bundled feature), serde, chrono | LOW |
| `crates/swal-store/src/lib.rs` | doc only | Export `session` module | LOW |
| `crates/swal-store/src/session.rs` | — | NEW: `Store` trait + `SessionStore` (rusqlite, WAL) | MED |
| `crates/swal-store/tests/session_test.rs` | — | NEW: CRUD round-trip test | LOW |
| `.github/workflows/ci.yml` | — | NEW: fmt+clippy+test workflow | LOW |

## DO NOT touch (Anti-Regression)
- `docs/ARCHITECTURE.md`, `docs/REUSE-MAP.md`, `docs/PLAN.md`, `README.md` — canonical, owned by orchestrator
- `crates/swal-loop/`, `crates/swal-agent/`, `crates/swal-gateway/`, `crates/swal-sched/` — other file islands
- The three OTHER empty web crates (swal-sync, swal-tools-native, swal-tools-web) — Wave 3
- Root `[profile.*]` sections in `Cargo.toml`

## Anti-Hallucination Guard ⚠️
1. **READ before write**: read each file completely before modifying (files are tiny skeletons).
2. **Git deps, NOT vendored code**: use `git = "https://github.com/iberi22/..."` — copying gestalt/synapse-agentic code into this repo is a FAILURE.
3. **No invented APIs**: verify rusqlite method names against docs.rs before using them.
4. **rusqlite bundled**: use `rusqlite = { version = "...", features = ["bundled"] }` so SQLite compiles without system lib.
5. **No tokio in swal-core**: this crate must compile to wasm32 later — no std::process, no tokio runtime.
6. **Tests must not need network or API keys.**
7. Empty PRs are forbidden — verify `git diff --stat HEAD` before push.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git status --porcelain` shows the new/modified files BEFORE opening the PR
- [ ] `git diff --stat HEAD` lists the files (NOT empty)
- [ ] The PR MUST contain >= 1 source file: verify with `git ls-files` before push
- [ ] If the work could not be completed: DO NOT open a PR — comment the blocker on the issue

## Verification
```bash
cargo check --workspace
cargo test -p swal-store 2>&1 | grep "test result: ok"
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
```

## Dependencies & Merge Order
- **Depends on:** none
- **Blocked by:** none
- **Parallel with:** none (this is the foundation)
- **Merge order within wave:** 1 of 3
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| `cargo check` fails on git deps | `cargo update` then retry; if gestalt_core doesn't compile standalone, pin a commit via `rev = "<sha>"` |
| rusqlite bundled build fails | Ensure `cc`/`cmake` available; on NixOS use `nix-shell -p openssl.dev pkg-config` |
| Test fails | Fix test logic or implementation |
| PR conflicts with parallel work | Rebase on main, re-run verification |
