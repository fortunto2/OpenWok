# Pipeline Retro: openwok (2026-03-21) — restaurant-orders + Full Session

## Overall Score: 7.6/10

## Pipeline Efficiency

| Metric | Value | Rating |
|--------|-------|--------|
| Total iterations | 36 | |
| Productive iterations | 34 (94.4%) | GREEN |
| Wasted iterations | 2 (5.6%) | GREEN |
| Ghost runs (no ITER) | 2 (~112 min lost) | RED |
| Pipeline restarts | 3 (mid-plan) | YELLOW |
| Max-iter hits | 0 | GREEN |
| Redo cycles | 4 (across 3 plans) | YELLOW |
| Total duration | ~404 min (6.7h active) | YELLOW |
| Plans cycled | 8 tracks (pilot-infra through restaurant-orders) | GREEN |

### Ghost Runs (Invisible Waste)

| Plan | Start | Duration | Cause |
|------|-------|----------|-------|
| repo-abstraction | 14:01 | ~92 min | Session abort — no ITER produced, no partial output saved |
| auth-payments | 16:24 | ~20 min | Session abort — no ITER produced |

**Impact:** 112 min of compute lost with zero output recovery. This is the pipeline's biggest weakness.

## Per-Stage Breakdown

| Stage | Attempts | Successes | Waste % | Notes |
|-------|----------|-----------|---------|-------|
| build | 16 | 12 | 25% | 2 auth-payments continuing + 1 admin-tools redo + 1 admin-tools false redo |
| deploy | 10 | 10 | 0% | Perfect — all 10 deploys succeeded first try |
| review | 10 | 8 | 20% | 3 legitimate redos (auth, onboarding, admin) — quality gate working as designed |

## Failure Patterns

### Pattern 1: Ghost Run / Session Abort (HIGH)
- **Occurrences:** 2 runs (repo-abstraction, auth-payments)
- **Root cause:** Pipeline session killed/aborted before completing first iteration — no partial output saved, all work lost
- **Wasted:** ~112 min of compute time
- **Fix:** `scripts/solo-dev.sh` — pipe iter output to file in real-time via `tee`, not just on completion. Recommended since retro #4 (4 retros ago).

### Pattern 2: False Redo Signal (MEDIUM)
- **Occurrences:** 1 (admin-tools iter-004-build)
- **Root cause:** `grep -q '<solo:redo/>'` matches signal text quoted in build's opening narrative ("The review returned `<solo:redo/>`...")
- **Wasted:** 1 extra build iteration (13 min)
- **Fix:** `scripts/solo-lib.sh:89` — scope grep to `tail -20` of output file instead of full file. Identified in retro #9.

### Pattern 3: Large Plan Overflows Session (LOW)
- **Occurrences:** 1 (auth-payments, 48 tasks)
- **Root cause:** 48-task plan exceeded single build session capacity, causing 2 incomplete builds before completion
- **Wasted:** 2 continuing build iterations + 2 ghost runs
- **Fix:** `skills/plan/SKILL.md` — auto-split plans >30 tasks. Recommended since retro #5.

## Plan Fidelity

| Track | Criteria Met | Tasks Done | SHAs | Rating |
|-------|-------------|------------|------|--------|
| pilot-infra | 10/10 (100%) | 31/31 (100%) | 12 | GREEN |
| repo-abstraction | 8/8 (100%) | 28/31 (90%) | 14 | GREEN |
| auth-payments | 11/11 (100%) | 46/48 (96%) | 18 | GREEN |
| restaurant-onboarding | 10/10 (100%) | 29/35 (83%) | 4 | YELLOW |
| courier-dispatch | 11/11 (100%) | 24/27 (89%) | 13 | GREEN |
| frontend-split | 10/10 (100%) | 40/40 (100%) | 22 | GREEN |
| admin-tools | 11/11 (100%) | 43/43 (100%) | 24 | GREEN |
| restaurant-orders | 11/11 (100%) | 26/26 (100%) | 13 | GREEN |
| **TOTAL** | **82/82 (100%)** | **267/281 (95%)** | **120** | GREEN |

100% acceptance criteria met across all 8 tracks. Outstanding fidelity.

## Code Quality (Quick)

- **Tests:** 107 pass, 0 fail (growth: 0 → 107 across session)
- **Build:** PASS (all crates compile, frontend WASM builds)
- **Commits:** 168 total, 156 conventional (92.9%)
- **CLAUDE.md:** 15,329 chars — well under 40K threshold
- **Secrets scan:** Clean (no hardcoded keys in crates/)

## Context Health

- Iteration quality trend: STABLE — no degradation across 36 iterations
- Observation masking: NOT USED (no scratch/ directory)
- Plan recitation: OBSERVED — build loads plan.md at session start
- CLAUDE.md size: 15,329 chars — OK

## Three-Axis Growth

| Axis | Score | Evidence |
|------|-------|----------|
| **Technical** | 9/10 | 8 full features shipped: auth, payments, restaurant CRUD, courier dispatch, admin tools, order management. 5-crate architecture, Repository pattern, dual-target Stripe client, Dioxus SPA with 14 routes |
| **Cognitive** | 8/10 | Auto-plan feedback loop working (retro → track → ship). Pre-commit hooks closed after 3 retros. Plan scoping improved (12-15 tasks = ideal, 48 = painful). Signal grep issue identified and documented |
| **Process** | 7/10 | 9 retros in evolution log, factory self-critique. But 3 recurring issues still unresolved: restart recovery (4 retros), plan size guard (3 retros), real-time log streaming (4 retros) |

## Scoring Breakdown

| Dimension | Weight | Score | Weighted |
|-----------|--------|-------|----------|
| Efficiency | 25% | 8 | 2.00 |
| Stability | 20% | 5 | 1.00 |
| Fidelity | 20% | 9 | 1.80 |
| Quality | 15% | 10 | 1.50 |
| Commits | 5% | 7 | 0.35 |
| Docs | 5% | 10 | 0.50 |
| Signals | 5% | 7 | 0.35 |
| Speed | 5% | 2 | 0.10 |
| **Total** | **100%** | | **7.6** |

## Recommendations

1. **[CRITICAL]** Implement real-time iter log streaming (`tee` to file during execution). 4th consecutive retro recommending this. Every ghost run = silent data loss. Fix: `scripts/solo-dev.sh` — `claude ... 2>&1 | tee "$iter_log_path"` instead of redirect-on-completion.

2. **[HIGH]** Add plan size guard — warn/auto-split plans >30 tasks. auth-payments (48 tasks) caused 2 ghost runs + 2 wasted iters. Fix: `skills/plan/SKILL.md` — count tasks after generation, split if >30.

3. **[HIGH]** Scope signal grep to output tail. False redo detection in admin-tools caused 1 extra build iteration (13 min). Fix: `scripts/solo-lib.sh:89` — `tail -20 "$OUTFILE" | grep -q '<solo:redo/>'`.

4. **[MEDIUM]** Add partial progress recovery on restart. When pipeline restarts mid-plan, detect prior commits and skip completed phases. Fix: `scripts/solo-dev.sh` — check plan.md for `<!-- sha:` annotations on `[x]` tasks.

5. **[LOW]** Add handler-level tests. `crates/handlers` has 0 tests — all testing is through integration tests in `crates/api`. Not blocking but a gap.

## Suggested Patches

### Patch 1: solo-lib.sh — Scope signal grep to tail

**What:** Prevent false redo detection from quoted signal text in build narrative
**Why:** Build skill opens with "The review returned `<solo:redo/>`" which triggers full-file grep match

```diff
- grep -q '<solo:redo/>' "$OUTFILE"
+ tail -20 "$OUTFILE" | grep -q '<solo:redo/>'
```

### Patch 2: solo-dev.sh — Real-time log streaming

**What:** Stream iter output to log file during execution, not just on completion
**Why:** Ghost runs lose all output — 112 min of compute with zero recovery

```diff
- claude ... > "$OUTFILE" 2>&1
+ claude ... 2>&1 | tee "$OUTFILE"
```

## restaurant-orders Track Summary

The final track ran cleanly: 3 iterations, 0 waste, ~28 minutes.

- **Build (20 min):** 12 tasks across 3 phases — Repository method, API handler, D1Repo, frontend Orders tab with auto-refresh. Single iteration, all tasks completed.
- **Deploy (5 min):** Frontend + worker WASM built and deployed. Clean.
- **Review (3 min):** All 11 acceptance criteria verified, 107 tests pass, `make check` clean. SHIP verdict. One advisory note: N+1 query pattern in `my_orders` (acceptable for pilot volume).
