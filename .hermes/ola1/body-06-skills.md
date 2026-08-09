# [Ola 1.06] swal-loop — Skills loader (2-layer cache, real impl)

> Ola 1 — Core. Labels: `ola1`, `wave-1` (NO `jules` yet).
> Feature: `feat-skills-loader` (6% of scope).

---

## Current State (MEASURABLE)
- `crates/swal-loop/src/skills.rs`: stub from issue 04 — `pub fn load_skills(_dir: &str) -> Vec<String> { Vec::new() }`.

## Desired State (DELTA)
Replace stub content in `crates/swal-loop/src/skills.rs` with a REAL 2-layer cache skills loader:
- `Skill { name: String, path: String, description: String, content: String }` (serde)
- `SkillLoader` struct:
  - Layer 1: in-memory snapshot (`DashMap<String, Skill>` or `HashMap` behind `RwLock`) — loaded once from disk at `new(dir)` / `reload()`
  - Layer 2: LRU cache for hot skills (use a simple VecDeque-based LRU or `lru` crate — if `lru` crate needed, report it: it is NOT in Cargo.toml and you may NOT edit it; implement a minimal LRU by hand)
  - `load_skills(dir) -> Result<Vec<Skill>, SkillError>`: walk `dir` recursively for `SKILL.md` files, parse YAML frontmatter `name` + `description` (frontmatter between `---` lines; NO yaml crate — parse with string ops) and body as content
  - `get(name) -> Option<Skill>`: LRU-hit fast path, fallback to snapshot
- `SkillError` (thiserror): `NotFound(String)`, `Io(String)`.
- Unit tests in `crates/swal-loop/src/skills.rs` `#[cfg(test)]`:
  - test 1: writes temp dir with 2 SKILL.md files → `load_skills` returns 2 with parsed frontmatter
  - test 2: second `load_skills` on same dir → cache hit path (snapshot reused — assert by calling twice and checking no re-walk, e.g. via a counter or by deleting the dir between calls and still getting results)

## 🌐 Web Research Required
1. search: "SKILL.md frontmatter format yaml name description"
2. search: "rust LRU cache implementation VecDeque"
3. search: "walkdir recursive SKILL.md files"

## Problem
The agent must load reusable skills (SKILL.md files) from the filesystem with caching so repeated turns don't re-read disk. Hermes uses a 2-layer cache — this mirrors it in ~100 LOC.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "struct SkillLoader" crates/swal-loop/src/skills.rs` >= 1
- [ ] `grep -c "struct Skill " crates/swal-loop/src/skills.rs` >= 1
- [ ] `grep -c "fn load_skills" crates/swal-loop/src/skills.rs` >= 1
- [ ] `cargo test -p swal-loop skills 2>&1 | grep "test result: ok"` — 1 match
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-loop/src/skills.rs` | stub (04) | Real SkillLoader 2-layer cache + frontmatter parse + tests | MED |

## DO NOT touch (Anti-Regression)
- `crates/swal-loop/Cargo.toml` + `lib.rs` (issue 04), `src/provider.rs` (05), `src/loop.rs` (07)
- `crates/swal-core/`, `crates/swal-store/`, `crates/swal-agent/`
- `docs/`, `.gitcore/features.json`
- **If you need a crate not in Cargo.toml (yaml, lru, walkdir) — implement by hand in this file. Do NOT edit Cargo.toml.**

## Anti-Hallucination Guard ⚠️
1. **No new deps**: this issue may NOT touch `Cargo.toml`. Frontmatter parse = string ops; LRU = hand-rolled; dir walk = `std::fs::read_dir` recursion.
2. **Frontmatter format**: file starts with `---\n`, ends with `\n---\n`; `name:` and `description:` lines are YAML scalars — extract with line splitting, not a YAML parser.
3. Tests use `std::env::temp_dir()` + unique subdir + `std::fs` cleanup; no network.
4. Cache must be `Send + Sync` (the loop runs in async contexts).

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] PR contains >= 1 real source file
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-loop skills 2>&1 | tail -5
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** #04 (scaffold + stub)
- **Blocked by:** none
- **Parallel with:** #05 (provider.rs), #07 (loop.rs) — disjoint files
- **Merge order within wave:** 4 of 12
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| Frontmatter parse edge cases | Support `name:` / `description:` at line start; tolerate missing description (empty string) |
| LRU borrow issues | Use `RwLock<VecDeque<(String, Skill)>>` or `Mutex`; keep it simple |
| Cache test flaky | Use unique temp dir per test; cleanup with Drop or explicit remove |
| Test fails | Fix test or implementation; do NOT weaken assertions |
