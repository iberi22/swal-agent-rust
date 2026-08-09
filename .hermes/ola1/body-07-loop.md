# [Ola 1.07] swal-loop — AgentLoop core (LLM → tools → feedback)

> Ola 1 — Core. Labels: `ola1`, `wave-1` (NO `jules` yet).
> Feature: `feat-agent-loop` (12% of scope — the biggest feature of Wave 1).

---

## Current State (MEASURABLE)
- `crates/swal-loop/src/loop.rs`: stub from issue 04 — `pub struct AgentLoop; impl AgentLoop { pub fn new() -> Self }`.
- Available (merged or in parallel): `Provider`/`MockProvider`/`ToolCall` from #05 (provider.rs), `SkillLoader` from #06 (skills.rs), `ToolRegistry` from #02 (swal-core).

## Desired State (DELTA)
Replace stub content in `crates/swal-loop/src/loop.rs` with the REAL AgentLoop:
- `AgentLoop` struct: holds `Arc<dyn Provider>`, `ToolRegistry`, `SkillLoader`, `max_steps: usize` (default 10), `session_id: Option<String>`.
- `AgentLoop::new(provider, tools, skills) -> Self` and `with_max_steps(...)` builder.
- Core method: `async fn run(&self, task: &str) -> Result<AgentOutput, LoopError>`:
  1. Build initial messages: system prompt (mentions skills) + user task
  2. Loop up to `max_steps`:
     - `provider.complete(&messages)` → `ProviderResponse`
     - If `tool_calls` non-empty: for each `ToolCall` → `tools.execute(name, args)` → append tool result message (role "tool")
     - Else: response is final → return `AgentOutput { content, steps, tool_calls_executed }`
  3. If max_steps exceeded → `LoopError::MaxSteps`
- `AgentOutput { content: String, steps: usize, tool_calls_executed: usize }` (serde)
- `LoopError` (thiserror): `MaxSteps`, `Provider(String)`, `Tool(String)`, `NoFinalResponse`.
- Unit tests in `crates/swal-loop/src/loop.rs` `#[cfg(test)]`:
  - Round-trip: MockProvider scripted [tool_call(echo), final] → loop executes tool via a test Tool in registry → asserts `tool_calls_executed == 1` and final content.

## 🌐 Web Research Required
1. search: "rust agent loop LLM tool calling pattern 2026"
2. search: "async loop tokio spawn_blocking tool execution"
3. search: "thiserror enum async errors"

## Problem
The conversational loop (prompt → LLM → tool_calls → execute → feedback) is the heart of the agent. This issue makes `swal-agent run "task"` possible end-to-end with MockProvider.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "struct AgentLoop" crates/swal-loop/src/loop.rs` >= 1
- [ ] `grep -c "async fn run" crates/swal-loop/src/loop.rs` >= 1
- [ ] `grep -c "enum LoopError" crates/swal-loop/src/loop.rs` >= 1
- [ ] `cargo test -p swal-loop 2>&1 | grep "test result: ok"` — 1 match
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-loop/src/loop.rs` | stub (04) | Real AgentLoop (run loop, tool execution, output, tests) | HIGH |

## DO NOT touch (Anti-Regression)
- `crates/swal-loop/Cargo.toml` + `lib.rs` (04), `src/provider.rs` (05), `src/skills.rs` (06)
- `crates/swal-core/`, `crates/swal-store/`, `crates/swal-agent/`
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **Use types from #05**: `Provider`, `Message`, `ToolCall`, `ProviderResponse` — import via `crate::provider::*`. If a field is missing, that's a real gap: comment on the issue instead of inventing.
2. **ToolRegistry API from swal-core**: `tools.execute(name, args) -> Result<Value, ToolError>` — verify signature in `crates/swal-core/src/tool.rs` before calling.
3. **MockProvider must drive the test** — no network, no API keys. Script: first response contains a ToolCall, second is final.
4. **Loop is platform-agnostic**: no tokio::fs/process here (that's native tools, later). File I/O goes through tools only.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] PR contains >= 1 real source file
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-loop 2>&1 | tail -8
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** #04 (scaffold), #05 (provider real), #06 (skills — import may be optional: if loop doesn't call skills yet, note it; prefer calling `skills.load_skills()` once at startup)
- **Blocked by:** none
- **Parallel with:** none (loop.rs is the integration point of the crate — merge AFTER 05 and 06 land)
- **Merge order within wave:** 5 of 12
- **Expected effort:** Large (4h+)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| Provider trait signature mismatch | Read `provider.rs` (merged #05) and adapt — do NOT edit provider.rs |
| ToolRegistry API differs | Read `crates/swal-core/src/tool.rs` and adapt the call |
| Borrow issues with Arc | Store `Arc<dyn Provider>` and clone per run; keep registry behind `Arc` |
| Test fails | Fix test or implementation; do NOT weaken assertions |
