# [Ola 1.01] Foundation — git deps + CI workflow

> Ola 1 — Foundation. Labels: `ola1`, `wave-1` (NO `jules` yet).
> Epic: https://github.com/iberi22/swal-agent-rust (100% feature scope).

---

## Current State (MEASURABLE)
- Root `Cargo.toml`: workspace-only, 9 empty crate skeletons, no `[workspace.dependencies]`, compiles in 0.18s.
- `gestalt` repo (public, `https://github.com/iberi22/gestalt.git`, HEAD `a5d9608`) is a workspace containing `gestalt_core`, `gestalt_state`... (`gestalt-state`), `synapse-agentic`, `gestalt-router`, etc.
- `synapse-agentic` is a MEMBER of the gestalt workspace — do NOT add a separate git dep.
- No `.github/` directory, no CI workflow.
- `.gitcore/features.json` exists: `feat-workspace-foundation` at 30%.

## Desired State (DELTA)
- **Root `Cargo.toml`**: add `[workspace.dependencies]`:
  - `gestalt_core = { git = "https://github.com/iberi22/gestalt.git", package = "gestalt_core" }`
  - `synapse-agentic = { git = "https://github.com/iberi22/gestalt.git", package = "synapse-agentic" }`
  - `gestalt-state = { git = "https://github.com/iberi22/gestalt.git", package = "gestalt-state" }`
  (verify exact member crate names by cloning gestalt and reading its workspace `members`)
- **`.github/workflows/ci.yml`** (NEW): on push/PR to `main` — `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace`. `CARGO_TARGET_DIR` not needed (default).

## 🌐 Web Research Required
1. search: "cargo git dependency workspace package rename 2026"
2. search: "github actions rust workflow cargo fmt clippy test 2026"
3. search: "cargo workspace.dependencies inherit git"

## Problem
The workspace has no shared dependency graph and no CI. Every Ola-1 crate needs `gestalt_core`/`synapse-agentic` via git deps, and every PR needs automated fmt/clippy/test verification.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "gestalt_core" Cargo.toml` >= 1 (in `[workspace.dependencies]`)
- [ ] `grep -c "synapse-agentic" Cargo.toml` >= 1
- [ ] `ls .github/workflows/ci.yml` — exists
- [ ] `grep -c "cargo clippy" .github/workflows/ci.yml` >= 1
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `Cargo.toml` | workspace + profiles only | Add `[workspace.dependencies]` with 3 git deps | MED |
| `.github/workflows/ci.yml` | — | NEW: fmt+clippy+test on push/PR | LOW |

## DO NOT touch (Anti-Regression)
- `docs/`, `README.md`, `HANDOFF.md`, `.gitcore/features.json` — orchestrator-owned
- Any `crates/*/` file or manifest (their Cargo.toml deps come in later issues)
- Root `[profile.*]` sections (lto=thin, strip, dev-fast must stay)

## Anti-Hallucination Guard ⚠️
1. **Verify crate names BEFORE writing deps**: `git clone --depth 1 https://github.com/iberi22/gestalt.git /tmp/g` then read `/tmp/g/Cargo.toml` `members` — use the EXACT package names found there.
2. **synapse-agentic is inside gestalt** — never add `https://github.com/iberi22/synapse-agentic.git` (it may exist but is NOT the canonical source).
3. Do NOT add `[workspace.dependencies]` entries for crates not verified to exist.
4. CI file must be valid YAML — no tabs.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git status --porcelain` shows Cargo.toml modified + ci.yml created BEFORE PR
- [ ] PR contains >= 1 real file (ci.yml)
- [ ] If blocked: comment the blocker on the issue, do NOT open an empty PR

## Verification
```bash
cargo check --workspace 2>&1 | tail -3
git diff --stat HEAD
```

## Dependencies & Merge Order
- **Depends on:** none
- **Blocked by:** none
- **Parallel with:** none (foundation — everything else needs these deps)
- **Merge order within wave:** 1 of 12
- **Expected effort:** Small (<1h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| git dep resolution fails | `cargo update`; verify package name exists in gestalt workspace members |
| CI YAML invalid | Validate locally with `python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/ci.yml'))"` |
| cargo check slow on first git fetch | Expected — deps compile once, ~2-5 min |
