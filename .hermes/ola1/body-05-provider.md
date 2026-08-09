# [Ola 1.05] swal-loop — Provider trait + MockProvider (real impl)

> Ola 1 — Core. Labels: `ola1`, `wave-1` (NO `jules` yet).
> Feature: `feat-llm-providers` (6% of scope).

---

## Current State (MEASURABLE)
- `crates/swal-loop/src/provider.rs`: stub from issue 04 — `Provider` trait with `complete(&self, prompt) -> Result<String,String>` and `MockProvider` returning `Ok(String::new())`.

## Desired State (DELTA)
Replace stub content in `crates/swal-loop/src/provider.rs` with a REAL implementation:
- `Provider` trait (async, Send+Sync): `complete(&self, messages: &[Message]) -> Result<ProviderResponse, ProviderError>` — the loop will pass a message list (system+user+tool results), NOT a single prompt string. Define `Message { role: String, content: String }` and `ProviderResponse { content: String, tool_calls: Vec<ToolCall> }`, `ToolCall { id: String, name: String, args: serde_json::Value }` (all serde-serializable).
- `MockProvider` (deterministic, NO network): configurable script — given a Vec of canned responses, returns them in order. Includes a response that emits a `ToolCall` so the loop can be tested end-to-end. E.g. `MockProvider::new(vec![resp_with_tool_call, resp_final])`.
- `ProviderError` (thiserror): `RequestFailed(String)`, `ParseFailed(String)`.
- **Keep the module compile-clean**: `cargo check --workspace` passes; unit tests in `crates/swal-loop/src/provider.rs` `#[cfg(test)]` module.

## 🌐 Web Research Required
1. search: "rust async trait error handling thiserror 2026"
2. search: "serde_json Value tool call arguments llm"
3. search: "rust unit tests async tokio test"

## Problem
The AgentLoop needs a real Provider abstraction to call LLMs (MockProvider for tests, synapse-agentic adapters later). The stub from issue 04 is not enough to drive a tool-calling loop.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "pub struct MockProvider" crates/swal-loop/src/provider.rs` >= 1
- [ ] `grep -c "pub struct ToolCall" crates/swal-loop/src/provider.rs` >= 1
- [ ] `grep -c "enum ProviderError" crates/swal-loop/src/provider.rs` >= 1
- [ ] `cargo test -p swal-loop provider 2>&1 | grep "test result: ok"` — 1 match
- [ ] `cargo check --workspace` — 0 errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-loop/src/provider.rs` | stub (04) | Real Provider/MockProvider/ToolCall/ProviderError + unit tests | MED |

## DO NOT touch (Anti-Regression)
- `crates/swal-loop/Cargo.toml` + `lib.rs` (owned by issue 04)
- `crates/swal-loop/src/skills.rs` (issue 06), `src/loop.rs` (issue 07)
- `crates/swal-core/`, `crates/swal-store/`, `crates/swal-agent/`
- `docs/`, `.gitcore/features.json`

## Anti-Hallucination Guard ⚠️
1. **Serde everything**: `Message`, `ToolCall`, `ProviderResponse` must derive `Serialize/Deserialize` (loop persists them).
2. **MockProvider is deterministic** — same input → same output; no randomness, no network, no env vars.
3. **Do NOT implement real LLM HTTP calls here** — synapse-agentic providers are a later wave (Wave 2, feat-llm-providers wiring). This issue is trait + mock only.
4. Tests must not need network or API keys.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty
- [ ] PR contains >= 1 real source file
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo test -p swal-loop provider 2>&1 | tail -5
cargo check --workspace 2>&1 | tail -2
```

## Dependencies & Merge Order
- **Depends on:** #04 (scaffold + stub)
- **Blocked by:** none
- **Parallel with:** #06 (skills.rs), #07 (loop.rs) — disjoint files
- **Merge order within wave:** 4 of 12
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| thiserror not in deps | It IS in loop Cargo.toml (issue 04 added it); if missing, report — do NOT edit Cargo.toml |
| async test issues | Use `#[tokio::test]` (tokio macros feature added in 04) |
| serde derive missing | serde/serde_json are in loop Cargo.toml from 04; verify with `grep serde crates/swal-loop/Cargo.toml` |
| Test fails | Fix test or implementation; do NOT weaken assertions |
