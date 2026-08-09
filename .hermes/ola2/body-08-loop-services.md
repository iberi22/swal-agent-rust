# [Ola 2.08] swal-loop — services scaffold (MCP client, xavier, compaction stubs)

> Ola 2 — Services. Labels: `ola2`, `wave-2` (NO `jules` yet).
> Features: scaffolding for `feat-mcp-client`, `feat-xavier-memory`, `feat-compaction`.

---

## Current State (MEASURABLE)
- `crates/swal-loop/src/lib.rs`: `pub mod provider; pub mod skills; pub mod r#loop;` (from Ola 1).
- `crates/swal-loop/Cargo.toml`: swal-core, gestalt_core, synapse-agentic, serde, tokio (from Ola 1 #04).

## Desired State (DELTA)
- **`crates/swal-loop/src/lib.rs`**: ADD `pub mod mcp_client; pub mod xavier; pub mod compaction;` (append to existing mods — do NOT remove existing).
- **`crates/swal-loop/src/mcp_client.rs`** (NEW stub — real in 2.09):
  ```rust
  pub struct McpClient;
  impl McpClient { pub fn new() -> Self { Self } }
  pub async fn list_tools(&self) -> Vec<String> { Vec::new() }
  ```
- **`crates/swal-loop/src/xavier.rs`** (NEW stub — real in 2.10):
  ```rust
  pub struct XavierClient;
  impl XavierClient { pub fn new(_base: &str) -> Self { Self } }
  pub async fn search(&self, _q: &str) -> Vec<String> { Vec::new() }
  ```
- **`crates/swal-loop/src/compaction.rs`** (NEW stub — real in 2.11):
  ```rust
  pub fn compact(_messages: &[crate::provider::Message]) -> Vec<crate::provider::Message> { Vec::new() }
  ```
- Workspace compiles; existing tests still pass.

## 🌐 Web Research Required
1. search: "rust lib.rs module declaration append"
2. search: "rmcp mcp client rust crate 2026"

## Problem
Wave 2 adds MCP client, xavier memory and compaction to swal-loop. Their module skeleton must compile before 2.09-2.11 land on disjoint files.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "mcp_client" crates/swal-loop/src/lib.rs` >= 1
- [ ] `grep -c "xavier" crates/swal-loop/src/lib.rs` >= 1
- [ ] `grep -c "compaction" crates/swal-loop/src/lib.rs` >= 1
- [ ] `ls crates/swal-loop/src/{mcp_client,xavier,compaction}.rs` — all exist
- [ ] `cargo test -p swal-loop 2>&1 | grep "test result: ok"` — >= 1 match (existing tests still pass)
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-loop/src/lib.rs` | 3 mods (Ola 1) | Append 3 new mods | LOW |
| `crates/swal-loop/src/mcp_client.rs` | — | NEW stub | LOW |
| `crates/swal-loop/src/xavier.rs` | — | NEW stub | LOW |
| `crates/swal-loop/src/compaction.rs` | — | NEW stub | LOW |

## DO NOT touch (Anti-Regression)
- `crates/swal-loop/Cargo.toml` (Ola 1 #04 owns), `src/provider.rs`, `src/skills.rs`, `src/loop.rs` — read-only
- `crates/swal-gateway/`, `crates/swal-sched/`, `crates/swal-agent/`
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **APPEND mods** — do not remove/reorder existing `pub mod` lines (Ola 1 tests depend on them).
2. Stubs reference `crate::provider::Message` — verify that type name in merged provider.rs first.
3. Stubs compile with zero warnings (`#[allow(dead_code)]` where needed).
4. No new Cargo.toml deps.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty (4 files)
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-loop 2>&1 | tail -4
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** Ola 1 complete (provider::Message exists)
- **Parallel with:** Ola 2.01 (gateway), 2.06 (sched) — different crates
- **Merge order within wave:** 7 of 12
- **Expected effort:** Small (<1h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| provider::Message name differs | Read merged provider.rs; adapt stub to real type |
| mod conflict | Ensure append-only edit of lib.rs |
| Test fails | Stubs must not break existing tests — check imports |
