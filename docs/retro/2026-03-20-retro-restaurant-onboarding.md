# Pipeline Retro: restaurant-onboarding (2026-03-20)

## Overall Score: 8.7/10

Best track execution in this pipeline run. Clean single start, legitimate redo that caught a real security bug, fast iterations.

## Pipeline Efficiency

| Metric | Value | Rating |
|--------|-------|--------|
| Total iterations | 6 | |
| Productive iterations | 5 (83%) | YELLOW |
| Wasted iterations | 1 (17%) | YELLOW |
| Pipeline restarts | 0 | GREEN |
| Max-iter hits | 0 | GREEN |
| Total duration | 52 min | GREEN |

**Note:** The 1 "wasted" iteration was a review redo that caught a TOCTOU security vulnerability — arguably the most valuable iteration in the track.

## Per-Stage Breakdown

| Stage | Attempts | Successes | Waste % | Notes |
|-------|----------|-----------|---------|-------|
| build | 2 | 2 | 0% | 35 min (initial) + 5 min (fix) |
| deploy | 2 | 2 | 0% | 4.5 min + 2 min |
| review | 2 | 1 | 50% | iter 3 redo (TOCTOU + clippy), iter 6 SHIP |

## Failure Patterns

### Pattern 1: Review → Redo (TOCTOU Security Bug)
- **Occurrences:** 1 redo cycle (iters 3→4→5→6)
- **Root cause:** `update_menu_item` handler in both `handlers/restaurants.rs` and `worker/lib.rs` modified data BEFORE verifying ownership. Any authenticated user could change any restaurant's menu items.
- **Wasted:** 1 iteration (review) + 3 extra iterations (build→deploy→review)
- **Verdict:** This was the pipeline working correctly. The review skill caught a real security bug that would've shipped to production. Cost: 3 extra iterations. Value: prevented an auth bypass vulnerability.
- **Fix:** Pre-build lint/security check could catch TOCTOU patterns earlier (static analysis for "mutate then check" patterns).

### Pattern 2: Doubled Redo Signal
- **Occurrences:** 1 (iter 3 review)
- **Root cause:** Review skill output `<solo:redo/>` twice — once at the start of its verdict text and once at the end.
- **Wasted:** 0 iterations
- **Fix:** Cosmetic — no action needed.

## Plan Fidelity

| Track | Criteria Met | Tasks Done | SHAs | Rating |
|-------|-------------|------------|------|--------|
| restaurant-onboarding_20260320 | 100% (10/10) | 92% (17/18.5) | Yes | GREEN |

**Unchecked items (3):** All are manual verification — wrangler dev test, visual e2e test, production runtime log check. Cannot be done autonomously.

**Phase 5 (review fixes):** 4/4 tasks completed with SHA annotations. TOCTOU fixed in both handlers and worker. `make check` passes.

## Code Quality (Quick)

- **Tests:** 98 pass, 0 fail — 7 new tests added (ownership enforcement, role promotion, menu CRUD)
- **Build:** PASS (all targets including wasm32)
- **Clippy:** Clean (0 warnings)
- **Fmt:** Clean
- **Commits:** 128 total, 93% conventional format (119/128)

## Context Health

- Iteration quality trend: **STABLE** — no degradation across 6 iters
- Observation masking: **NOT USED** — but track was small enough (13 tasks) to not need it
- Plan recitation: **OBSERVED** — build loaded plan at each session start via Context Handoff section
- CLAUDE.md size: **13,521 chars — OK** (well under 40K threshold)

## Three-Axis Growth

| Axis | Score | Evidence |
|------|-------|----------|
| **Technical** (code, tools, architecture) | 9/10 | Full CRUD ownership model, migration, 3 frontend pages, TOCTOU fix — solid Rust + Dioxus work |
| **Cognitive** (understanding, strategy, decisions) | 8/10 | Correct dependency ordering (auth-payments → restaurant-onboarding), auto-promote role decision, !Send workaround maintained |
| **Process** (harness, skills, pipeline, docs) | 8/10 | Review caught security bug (quality gate working), plan Phase 5 added dynamically, clean redo→fix→ship cycle |

## Cumulative Pipeline Summary (Full Day)

| Plan Track | Iters | Waste | Restarts | Duration | Score |
|------------|-------|-------|----------|----------|-------|
| cf-workers-deploy | 0 | — | — | 5m (retro only) | N/A (archived at 0%) |
| pilot-infra | 3 | 0% | 0 | 62m | 10/10 (retro #3) |
| repo-abstraction | 3 | 0% | 1 | 42m | 9/10 (retro #4) |
| auth-payments | 8 | 37% | 2 | 83m | 7/10 (retro #5) |
| **restaurant-onboarding** | **6** | **17%** | **0** | **52m** | **8.7/10** |
| **TOTAL** | **20** | **20%** | **3** | **~4h active** | **7.4 avg** |

Pipeline cycled 5 plans, completed 4 feature tracks, grew from 55→98 tests, deployed 4 times to CF Workers.

## Recommendations

1. **[MEDIUM]** Add pre-build `cargo fmt --check` gate to build skill — would've prevented the auth-payments redo cycle (retro #5). Same issue didn't recur here but remains unfixed.

2. **[MEDIUM]** Split `crates/frontend/src/main.rs` (1822 lines) into component modules — review flagged this. A refactor track should be queued.

3. **[LOW]** Add pre-commit hooks (`cargo clippy` + `cargo fmt`) — review recommended this. Would catch lint issues before they reach the pipeline.

4. **[LOW]** Add `wrangler tail` log monitoring to post-deploy checklist in deploy skill — useful for catching runtime errors.

5. **[LOW]** Add tarpaulin or similar for test coverage tracking — currently no coverage measurement.

## Suggested Patches

### Patch 1: CLAUDE.md — Remove cf-workers-deploy from active mentions

**What:** cf-workers-deploy was archived at 0% and its work was absorbed by repo-abstraction. Remove stale references.
**Why:** Prevents future agents from being confused by a completed plan that appears incomplete.

No code change needed — cf-workers-deploy is already in `plan-done/` and the spec makes it clear it was superseded.

### Patch 2: Build skill — Pre-build fmt check

**What:** Add `cargo fmt --check` before starting task loop in build skill.
**Why:** Prevents fmt failures from cascading into review → redo cycles.

```diff
# In build skill pre-flight or task loop:
+ ## Pre-Build Gate
+ Before starting the task loop, run:
+ ```bash
+ cargo fmt --check --all 2>&1
+ ```
+ If formatting issues exist, fix them first and commit: `style: fix formatting`
```

## Scoring Breakdown

| Dimension | Weight | Score | Weighted |
|-----------|--------|-------|----------|
| Efficiency (17% waste) | 25% | 7 | 1.75 |
| Stability (0 restarts) | 20% | 10 | 2.00 |
| Fidelity (100% criteria) | 20% | 9 | 1.80 |
| Quality (98 tests, 0 fail) | 15% | 10 | 1.50 |
| Commits (93% conventional) | 5% | 7 | 0.35 |
| Docs (SHAs, plan complete) | 5% | 10 | 0.50 |
| Signals (1 cosmetic double) | 5% | 7 | 0.35 |
| Speed (52 min) | 5% | 8 | 0.40 |
| **TOTAL** | **100%** | | **8.65 → 8.7** |
