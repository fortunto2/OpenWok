# Pipeline Retro: openwok / admin-tools_20260320 (2026-03-20)

## Overall Score: 8.6/10

Track: admin-tools (block/unblock + dispute resolution)
Pipeline: build -> deploy -> review | 7 iterations | 42 min

## Pipeline Efficiency

| Metric | Value | Rating |
|--------|-------|--------|
| Total iterations | 7 | |
| Productive iterations | 5 (71%) | YELLOW |
| Wasted iterations | 2 (29%) | YELLOW |
| Pipeline restarts | 0 | GREEN |
| Max-iter hits | 0 | GREEN |
| Redo cycles | 1 (legitimate) | YELLOW |
| Total duration | ~42 min | GREEN |

## Per-Stage Breakdown

| Stage | Attempts | Successes | Waste % | Notes |
|-------|----------|-----------|---------|-------|
| build | 3 | 2 | 33% | Iter 4 signal confusion (see Pattern 1), iter 5 instant recovery |
| deploy | 2 | 2 | 0% | 7th consecutive 0-waste deploy |
| review | 2 | 1 (+1 redo) | 0% | First review caught real security gap — legitimate redo |

## Failure Patterns

### Pattern 1: False Redo — Signal Quoting in Build Output (iter 4)

- **Occurrences:** 1 iteration
- **Root cause:** Build iter 4 started with: `The review returned '<solo:redo/>', so I need to find and fix the issues...` — the agent quoted the previous review's redo signal in natural language. The pipeline's `handle_signals()` in `solo-lib.sh:89` uses `grep -q '<solo:redo/>'` which matched this quoted text as a real redo signal. Since `<solo:redo/>` takes precedence over `<solo:done/>` (line 93), the build's actual `<solo:done/>` was ignored. Result: false redo cycle 2/2, all state markers removed, 1 wasted iteration.
- **Wasted:** 1 iteration (iter 5 re-ran build, completed instantly because work was already done)
- **Fix:** `scripts/solo-lib.sh:89` — match signals only on standalone lines, not in quoted context:
  ```bash
  # Before:
  grep -q '<solo:redo/>' "$OUTFILE"
  # After:
  grep -qP '^\s*<solo:redo/>\s*$' "$OUTFILE"
  ```
  Or match only in the last 20 lines of output (signals always appear at the end).

### Pattern 2: Review Redo — Blocked-User Enforcement Gap

- **Occurrences:** 1 redo cycle (iter 3 review → iter 4-7 fix cycle)
- **Root cause:** This is a **legitimate** redo. Review correctly identified that `get_active_user` (blocked-user check) was only applied to admin endpoints and `create_dispute`, but not to restaurant CRUD, courier registration, or order creation. Spec criterion "Blocked users receive 403 on all authenticated endpoints" was not met.
- **Wasted:** 0 (redo was correct, Phase 4 tasks fixed real security gap)
- **Fix:** No fix needed — pipeline working as designed. The review skill caught a real vulnerability.

## Plan Fidelity

| Track | Criteria Met | Tasks Done | SHAs | Rating |
|-------|-------------|------------|------|--------|
| admin-tools_20260320 | 100% (11/11) | 100% (45/45) | 24 | GREEN |

4 phases: Domain+Migration → API+Auth → Frontend+Docs → Review Fixes (blocked-user enforcement)

## Code Quality (Quick)

- **Tests:** 106 pass, 0 fail (grew from 101 this track)
- **Build:** PASS (cargo build all crates)
- **Clippy:** Clean (0 warnings)
- **Formatting:** Clean
- **Commits:** 158 total, 92.4% conventional format (146/158)
- **Pre-commit hooks:** Active (fmt + clippy via .githooks/)

## Context Health

- Iteration quality trend: STABLE (no degradation across 7 iters)
- Observation masking: NOT USED (no scratch/ dir — track was small enough)
- Plan recitation: OBSERVED (build reads plan.md at session start)
- CLAUDE.md size: 14,963 chars — OK

## Three-Axis Growth

| Axis | Score | Evidence |
|------|-------|----------|
| **Technical** | 9/10 | Admin tools (block/unblock + disputes) shipped, blocked-user enforcement on ALL auth endpoints, 5 new repo methods, migration, frontend tabs |
| **Cognitive** | 8/10 | Review caught non-obvious security gap (blocked enforcement scope), Phase 4 added dynamically to fix it |
| **Process** | 7/10 | Redo mechanism worked correctly, but signal confusion in iter 4 wasted 1 iteration |

## Dimension Scores

| Dimension | Weight | Score | Weighted |
|-----------|--------|-------|----------|
| Efficiency | 25% | 6 | 1.50 |
| Stability | 20% | 10 | 2.00 |
| Fidelity | 20% | 10 | 2.00 |
| Quality | 15% | 10 | 1.50 |
| Commits | 5% | 7 | 0.35 |
| Docs | 5% | 10 | 0.50 |
| Signals | 5% | 7 | 0.35 |
| Speed | 5% | 8 | 0.40 |
| **Total** | **100%** | | **8.6** |

## Full Pipeline Session Summary

This was the final track in a 7-track pipeline session. Full session metrics:

| Track | Iters | Productive | Waste | Duration | Score |
|-------|-------|------------|-------|----------|-------|
| pilot-infra | 3 | 3 | 0% | 62m | 10/10 |
| repo-abstraction | 3 (+1 ghost) | 3 | 25% | 42m | 9/10 |
| auth-payments | 7 (+2 ghosts) | 5 | 44% | 83m | 7/10 |
| restaurant-onboarding | 6 | 5 | 17% | 58m | 8.5/10 |
| courier-dispatch | 3 | 3 | 0% | 55m | 9.5/10 |
| frontend-split | 3 | 3 | 0% | 21m | 8.5/10 |
| admin-tools | 7 | 5 | 29% | 42m | 8.6/10 |
| **Session total** | **35** | **27** | **23%** | **~363m** | **7.3/10** |

Key session achievements:
- 7 feature tracks shipped in ~6 hours effective time
- 106 tests, 0 failures, clippy clean
- 158 commits, 92.4% conventional
- 12 completed plan tracks total (from project start)
- Pre-commit hooks finally implemented (closed 3-retro gap)
- CLAUDE.md lean at 14.9KB

## Recommendations

1. **[HIGH]** Fix redo counter reset in `solo-dev.sh` — counter should reset when transitioning from review back to build during a redo cycle, not carry over and cause false redo signals on build stage
2. **[MEDIUM]** Add partial progress recovery on pipeline restart — detect commits since last successful iter, skip completed phases (recurring since retro #4)
3. **[MEDIUM]** Add plan size guard in `/plan` skill — warn/split when >30 tasks (recurring since retro #5)
4. **[LOW]** Add observation masking convention (`scratch/` dir) for large builds — reduces context pressure on complex tracks
5. **[LOW]** Add handler-level tests in handlers crate (currently 0 dedicated tests — all testing via API integration)

## Suggested Patches

### Patch 1: solo-lib.sh:89 — Fix signal detection to avoid false matches in quoted text

**What:** Match `<solo:redo/>` and `<solo:done/>` only on standalone lines (or in last 20 lines)
**Why:** Build agent quoted previous review's `<solo:redo/>` in natural language, causing false redo detection (Pattern 1)

```diff
# solo-lib.sh handle_signals():
- if grep -q '<solo:redo/>' "$OUTFILE" 2>/dev/null; then
+ if tail -20 "$OUTFILE" | grep -q '<solo:redo/>' 2>/dev/null; then
    HAS_REDO=true
  fi
- if [[ "$HAS_REDO" != "true" ]] && grep -q '<solo:done/>' "$OUTFILE" 2>/dev/null; then
+ if [[ "$HAS_REDO" != "true" ]] && tail -20 "$OUTFILE" | grep -q '<solo:done/>' 2>/dev/null; then
```

### Patch 2: CLAUDE.md — Add blocked-user enforcement note

**What:** Document that ALL authenticated endpoints must check blocked status
**Why:** Review caught this gap — future agents should know the pattern

```diff
+ **Blocked-user enforcement:** All authenticated endpoints MUST call `get_active_user` (which checks `blocked == false`). Do NOT use `get_user_by_supabase_id` directly in handlers — it skips the blocked check.
```
