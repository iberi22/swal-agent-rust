# [Ola 1.02] swal-loop — conversational agent loop (LLM → tools → feedback)

> Ola 1 — Core. Labels: `ola1`, `wave-1` (NO `jules` yet).

---

## Current State (MEASURABLE)
- `crates/swal-loop/src/lib.rs` — 3-line doc comment only. No `AgentLoop`, no LLM wiring, no skills.
- `crates/swal-loop/Cargo.toml` — no dependencies.
- Foundation (#1) provides: git deps `gestalt_core` + `synapse-agentic`, `swal-core::tool::ToolRegistry`, `swal-store` sessions.

## Desired State (DELTA)
- **`crates/swal-loop/src/loop.rs`**: `AgentLoop` struct with `run(user_message: &str) -> Result<LoopResponse>`. Loop: build prompt → call LLM (synapse-agentic `LLMProvider` trait, OpenAI-compatible messages with tool definitions) → parse `tool_calls` → execute via `swal_core::tool::ToolRegistry` (which dispatches to gestalt tools: shell/read/write/git) → feed results back → repeat until final answer (max 10 iterations) → return final text + tool trace.
- **`crates/swal-loop/src/provider.rs`**: `ProviderFactory` — builds an `LLMProvider` from config: default OpenRouter via synapse-agentic, model override via env `SWAL_LLM_MODEL`; a `MockProvider` (for tests, returns scripted tool_call then final answer).
- **`crates/swal-loop/src/skills.rs`**: skills loader — scan `skills/**/SKILL.md` (or `~/.hermes/skills` via env `SWAL_SKILLS_DIR`), parse frontmatter (name, description), keep in DashMap; expose `select(query) -> Vec<Skill>` by description keyword match. Do NOT import gestalt's skills cache yet — this is the thin NEW loader (gestalt's 2-layer cache is REUSED later in Wave 2).
- **`crates/swal-loop/src/lib.rs`**: export `loop_`, `provider`, `skills` modules.
- **New test**: `crates/swal-loop/tests/loop_test.rs` — AgentLoop with MockProvider executes one tool_call and produces final answer; verifies tool trace non-empty.

## 🌐 Web Research Required
1. search: "openai chat completions tool calling function call loop pattern"
2. search: "rust serde tagged enum parse tool_calls JSON"
3. search: "frontmatter parse rust crate 2026"
4. search: "async_trait LLM provider abstraction rust"

## 🔬 Agent Session Prompt
"Before implementing:
1. Read `docs/ARCHITECTURE.md` (swal-loop data flow) and `docs/REUSE-MAP.md` (which pieces come from synapse-agentic vs NEW).
2. Read `crates/swal-core/src/tool.rs` from #1 — the `Tool`/`ToolRegistry` API you must call.
3. Inspect the `synapse-agentic` crate (git dep) source under `~/.cargo/git/checkouts/` — find the `LLMProvider` trait and provider structs; confirm method signatures.
4. Document findings, then implement."

## Existing Code Patterns
- All crates use `version.workspace = true` manifest pattern.
- `swal-core::tool` types are the shared contract — reuse, don't redefine.
- Tests never require network/API keys — always MockProvider.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `cargo check -p swal-loop` — 0 errors
- [ ] `cargo test -p swal-loop 2>&1 | grep "test result: ok"` — 1 match
- [ ] `grep -c "pub struct AgentLoop" crates/swal-loop/src/loop.rs` >= 1
- [ ] `grep -c "MockProvider" crates/swal-loop/src/provider.rs` >= 1
- [ ] `grep -c "struct Skill" crates/swal-loop/src/skills.rs` >= 1
- [ ] `git show HEAD --name-only | grep -cE "crates/swal-loop/src/(loop|provider|skills)\.rs"` >= 3

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-loop/Cargo.toml` | no deps | Add synapse-agentic (git), swal-core (path), serde, serde_json, async-trait, dashmap, tracing | MED |
| `crates/swal-loop/src/lib.rs` | doc only | Export modules | LOW |
| `crates/swal-loop/src/loop.rs` | — | NEW: `AgentLoop` + tool-calling iteration | HIGH |
| `crates/swal-loop/src/provider.rs` | — | NEW: `ProviderFactory` + `MockProvider` | MED |
| `crates/swal-loop/src/skills.rs` | — | NEW: skills loader (frontmatter parse + select) | MED |
| `crates/swal-loop/tests/loop_test.rs` | — | NEW: mock-driven loop test | LOW |

## DO NOT touch (Anti-Regression)
- `docs/*.md`, `README.md`, `AGENTS.md` — canonical
- `crates/swal-agent/`, `crates/swal-gateway/`, `crates/swal-sched/` — other islands
- `crates/swal-core/`, `crates/swal-store/` — owned by #1 (merge first)
- Root `Cargo.toml` workspace section
- gestalt / synapse-agentic repos — never edit them, consume only

## Anti-Hallucination Guard ⚠️
1. **READ before write**: read `swal-core::tool` API and the actual synapse-agentic source (git checkout) before coding the loop.
2. **Loop termination**: hard cap 10 iterations — infinite loops are a FAILURE.
3. **Tool errors are data**: tool execution errors must be fed back to the LLM as text, never panic.
4. **MockProvider first**: the loop must be testable with zero network — real providers behind a trait.
5. **No API keys in code/tests** — config via env only (`SWAL_LLM_MODEL`, `OPENROUTER_API_KEY`).
6. Empty PRs are forbidden — verify `git diff --stat HEAD` before push.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git status --porcelain` shows the new/modified files BEFORE opening the PR
- [ ] `git diff --stat HEAD` lists the files (NOT empty)
- [ ] The PR MUST contain >= 1 source file: verify with `git ls-files` before push
- [ ] If the work could not be completed: DO NOT open a PR — comment the blocker on the issue

## Verification
```bash
cargo check -p swal-loop
cargo test -p swal-loop 2>&1 | grep "test result: ok"
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
```

## Dependencies & Merge Order
- **Depends on:** #1 (foundation: Tool trait + git deps)
- **Blocked by:** #1
- **Parallel with:** none
- **Merge order within wave:** 2 of 3
- **Expected effort:** Large (4h+)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| `cargo check` fails on synapse-agentic API mismatch | Read its actual source (git checkout) and adapt; pin gestalt/synapse-agentic commits if needed |
| Tool-call parse fails | The provider returns tool_calls in OpenAI format — match serde struct to that exact shape |
| Test hangs | MockProvider must return a terminal answer within N calls — assert iteration cap |
| PR conflicts | Rebase on main, re-run verification |
