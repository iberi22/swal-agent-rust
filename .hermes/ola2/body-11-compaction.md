# [Ola 2.11] swal-loop — Compaction (synapse-agentic reuse)

> Ola 2 — Services. Labels: `ola2`, `wave-2` (NO `jules` yet).
> Feature: `feat-compaction` (4% of scope).

---

## Current State (MEASURABLE)
- `crates/swal-loop/src/compaction.rs`: stub from Ola 2.08 (`compact()` returning empty vec).

## Desired State (DELTA)
Replace stub in `crates/swal-loop/src/compaction.rs` with REAL compaction:
- Per REUSE-MAP: reuse synapse-agentic compaction module (path-dep already in loop Cargo.toml from Ola 1 #04).
- `pub fn compact(messages: &[crate::provider::Message], max_tokens: usize) -> Vec<crate::provider::Message>`:
  - If synapse-agentic exposes a compaction fn: wrap it (verify API in the merged dep source; adapt in THIS file)
  - Fallback (if not directly reusable): simple context trimming — keep system + last N messages, prepend a summary marker message `{"role":"system","content":"[compacted: kept last N of M messages]"}` — documented as v1
- `pub fn should_compact(messages: &[Message], max_tokens: usize) -> bool` — heuristic: total char length > threshold
- Unit tests: (1) short list → not compacted; (2) long list → compacted, summary marker present, size reduced; (3) should_compact boundary.

## 🌐 Web Research Required
1. search: "synapse-agentic compaction rust module"
2. search: "rust context trimming summary messages LLM"
3. search: "token estimation char heuristic"

## Problem
Long sessions degrade quality and cost tokens (Wave-2 done-criteria: "Compaction triggers on long session"). Reuse synapse-agentic per REUSE-MAP, with a documented v1 fallback.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "fn compact" crates/swal-loop/src/compaction.rs` >= 1
- [ ] `grep -c "fn should_compact" crates/swal-loop/src/compaction.rs` >= 1
- [ ] `grep -c "compacted" crates/swal-loop/src/compaction.rs` >= 1 (marker)
- [ ] `cargo test -p swal-loop compaction 2>&1 | grep "test result: ok"` — 1 match
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-loop/src/compaction.rs` | stub (2.08) | Real compaction + should_compact + tests | MED |

## DO NOT touch (Anti-Regression)
- `crates/swal-loop/Cargo.toml` + `lib.rs` (2.08 owns mod), `src/mcp_client.rs` (09), `src/xavier.rs` (10)
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **Verify synapse compaction API** in merged dep source (`~/.cargo/git/checkouts/gestalt-*/` or `grep -rn "compaction" ~/.cargo/...`) BEFORE writing — wrap it if compatible, else documented fallback.
2. Fallback keeps FIRST message (system) + LAST N — never drop the system prompt.
3. Summary marker must be a `Message` with role "system".
4. No new deps; no network.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-loop compaction 2>&1 | tail -5
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** Ola 2.08 (scaffold), Ola 1 (provider::Message)
- **Parallel with:** Ola 2.09, 2.10 — disjoint files
- **Merge order within wave:** 8 of 12
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| synapse compaction API incompatible | Documented v1 fallback (trim + marker) |
| Message type differs | Read merged provider.rs; adapt |
| Test fails | Fix logic; do NOT weaken assertions |
