# [Ola 3.12] swal-pwa — PWA wiring + Ola 3 reconciliation

> Ola 3 — Web (finale). Labels: `ola3`, `wave-3` (NO `jules` yet).
> Features: `feat-pwa-leptos` (final wiring), all W3 features reconciliation.

---

## Current State (MEASURABLE)
- `crates/swal-pwa/src/app.rs`: Leptos App shell (3.10), worker real (3.11).
- `crates/swal-core/src/wasm.rs` (3.02), tools-web (3.04-3.06), store-indexeddb (3.07), sync (3.08-3.09) merged.
- `.gitcore/features.json`: wave3 features at claimed targets.

## Desired State (DELTA)
- **`crates/swal-pwa/src/app.rs`** (final wiring, minimal edit): wire the real worker:
  - "Run" button → `worker::run_task(task)` → display result
  - Session list from `swal_store::indexeddb::IndexedDbStore` (wasm) — best-effort, warn on error
- **`crates/swal-pwa/index.html`** (NEW): minimal PWA shell (manifest link, service worker registration optional)
- **`.gitcore/features.json`** (reconciliation, orchestrator-owned — ONLY wave-3 issue allowed to touch it):
  - Set wave3 features to final honest % based on what merged (wasm-core-loop=100 if wasm build passed, tools-opfs/git-web/shell-web=100 if impl merged, store-indexeddb=100, sync-crdt=100, sync-transport=100, pwa-leptos/comlink/webllm=100 or partial)
  - Update `last_updated`
- Verify: `cargo check --workspace` + full test suite green + wasm32 build of swal-core.

## 🌐 Web Research Required
1. search: "pwa manifest json service worker minimal 2026"
2. search: "leptos event listener button onclick"

## Problem
This closes Wave 3 (15%): the PWA is wired end-to-end (worker + WebLLM + IndexedDB + sync) and features.json reflects the honest final %.

## Acceptance Criteria (COMMAND-VERIFIABLE)
- [ ] `grep -c "run_task" crates/swal-pwa/src/app.rs` >= 1 (wired)
- [ ] `ls crates/swal-pwa/index.html` — exists
- [ ] `cargo check --workspace` — 0 errors
- [ ] `cargo test --workspace 2>&1 | grep -c "test result: ok"` >= 6 (loop, store, sync, core, tools-web, sched...)
- [ ] `python3 -c "import json; d=json.load(open('.gitcore/features.json')); print(all(f['progress_pct']>=100 for f in d['features'] if f['wave']=='wave3'))"` — True

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-pwa/src/app.rs` | shell (3.10) | Wire worker + store (minimal) | MED |
| `crates/swal-pwa/index.html` | — | NEW: PWA shell | LOW |
| `.gitcore/features.json` | wave3 targets | Final honest % | LOW |

## DO NOT touch (Anti-Regression)
- `crates/swal-pwa/Cargo.toml` + `lib.rs` + `src/worker.rs` — read-only
- Other crates; `docs/`, `README.md`, `HANDOFF.md`

## Anti-Hallucination Guard ⚠️
1. **features.json is orchestrator-owned** — ONLY this wave-3 issue may touch it; set honest % (100 only if evidence exists).
2. Worker call is synchronous wrapper; keep it simple.
3. IndexedDB wiring best-effort (wasm-gated; native falls back to in-memory).
4. Don't break existing app shell tests.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git diff --stat HEAD` non-empty (3 files)
- [ ] If blocked: comment on issue, no empty PR

## Verification
```bash
cargo check --workspace 2>&1 | tail -3
cargo test --workspace 2>&1 | grep "test result: ok" | wc -l
cargo build -p swal-core --target wasm32-unknown-unknown 2>&1 | tail -1
python3 -c "import json; d=json.load(open('.gitcore/features.json')); print(sum(f['progress_pct']*f['weight'] for f in d['features'])/100)"
```

## Dependencies & Merge Order
- **Depends on:** Ola 3.01-3.11 ALL merged
- **Merge order within wave:** 12 of 12 (LAST — reconciliation)
- **Expected effort:** Medium (1-4h)

## Failure Recovery
| If this happens | Action |
|----------------|--------|
| wasm build fails | Report; set honest partial % |
| Existing tests break | Fix wiring, keep prior behavior |
| features.json evidence missing | Set honest % (not inflated); comment |
