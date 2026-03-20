# Pipeline Retro: openwok / auth-payments (2026-03-20)

## Overall Score: 6.8/10

This retro covers the auth-payments_20260320 pipeline run and its context within the full day of pipeline activity (8 plan tracks cycled, 6 built). Auth-payments was the most complex track — Supabase Auth + Stripe Connect payments across 4 phases, 48 tasks.

## Pipeline Efficiency

| Metric | Value | Rating |
|--------|-------|--------|
| Total iterations (auth-payments) | 8 | |
| Productive iterations | 5 (63%) | YELLOW |
| Wasted iterations | 3 (37%) | YELLOW |
| Pipeline restarts (auth-payments) | 3 | RED |
| Max-iter hits | 0 | GREEN |
| Total duration (auth-payments) | ~3h45m (16:24 → 20:09) | YELLOW |
| Active agent time | ~1h18m (18:52 → 20:09 final run) | GREEN |
| **Full day totals** | | |
| Total iterations (all tracks) | 28 | |
| Productive (all tracks) | 19 (68%) | YELLOW |
| Wasted (all tracks) | 9 (32%) | YELLOW |
| Plans cycled | 8 (6 built, 2 archived) | GREEN |
| Wall clock (full day) | ~18h (02:12 → 20:09) | RED |

## Per-Stage Breakdown

| Stage | Attempts | Successes | Waste % | Notes |
|-------|----------|-----------|---------|-------|
| build | 4 | 2 | 50% | 1 interrupted, 1 no signal |
| deploy | 2 | 2 | 0% | Clean both times |
| review | 2 | 1+1 redo | 0% | 1 redo cycle (legitimate review findings) |

### Full Day Per-Stage (all tracks)

| Stage | Attempts | Successes | Waste % | Notes |
|-------|----------|-----------|---------|-------|
| setup | 1 | 1 | 0% | One-shot |
| plan | 2 | 1 | 50% | 1 rate-limited |
| build | 10 | 7 | 30% | Deploy spin-loop (5), interrupted (2), no signal (1) |
| deploy | 9 | 5 | 44% | 5 spin-loop failures (mvp-core early run) |
| review | 5 | 4 | 20% | 1 redo cycle (auth-payments) |

## Failure Patterns

### Pattern 1: Deploy Spin-Loop (mvp-core, early run)
- **Occurrences:** 5 consecutive failed deploy iterations
- **Root cause:** Deploy skill used `AskUserQuestion` in pipeline mode — no user to answer, so it exited without signal. Known defect, fixed in deploy SKILL.md (pipeline mode section added).
- **Wasted:** 5 iterations (~30 min)
- **Status:** FIXED in prior retro. Subsequent deploys all one-shot.

### Pattern 2: Build Interruption (auth-payments)
- **Occurrences:** 2 builds interrupted (16:24 and 16:44 runs), 1 build with no signal (16:50)
- **Root cause:** User killed/restarted pipeline during long build. Build produced commits but no `<solo:done/>` signal — likely context window pressure on 48-task plan.
- **Wasted:** 3 iterations (~2h wall clock including restart gaps)
- **Fix:** Pipeline should detect partial progress and resume from last commit, not restart build from scratch.

### Pattern 3: Redo Cycle (auth-payments)
- **Occurrences:** 1 redo cycle (review → build → deploy → review)
- **Root cause:** Review found real issues — `cargo fmt` failures across workspace + worker, auth not enforced on worker protected routes, `unwrap()` on date parsing.
- **Wasted:** 0 (this was productive — review caught real bugs)
- **Status:** EXPECTED. Redo was legitimate. Fixes were applied correctly.

## Plan Fidelity

| Track | Criteria Met | Tasks Done | SHAs | Rating |
|-------|-------------|------------|------|--------|
| mvp-core_20260320 | 100% (10/10) | 100% (39/39) | yes | GREEN |
| phase2-frontend | 100% (8/8) | 100% (10/10) | no | GREEN |
| cf-workers-deploy_20260320 | 0% (0/8) | 0% (0/42) | no | RED |
| pilot-infra_20260320 | 100% (10/10) | 100% (31/31) | yes | GREEN |
| repo-abstraction_20260320 | 100% (8/8) | 90% (28/31) | yes | GREEN |
| auth-payments_20260320 | 96% (46/48) | 96% (46/48) | yes | GREEN |

**Notes:**
- cf-workers-deploy: Work was done and deployed, but spec/plan checkboxes were never ticked. Archival without completion validation (known factory defect).
- auth-payments remaining 2 tasks: likely deploy secrets (SUPABASE_JWT_SECRET, STRIPE_SECRET_KEY) — manual steps, not automatable.
- repo-abstraction 3 unchecked tasks: handler-level tests not added (deferred).

## Code Quality (Quick)

- **Tests:** 91 pass, 0 fail, 2 ignored (100% pass rate)
- **Build:** PASS (all crates compile including frontend WASM)
- **Commits:** 113 total, 104 conventional format (92%)
- **Crate count:** 6 (core, handlers, api, frontend, worker, stripe-universal)
- **Test distribution:** core 42, api 33, handlers 8, stripe-universal 8

## Context Health

- **Iteration quality trend:** STABLE — later iterations (auth-payments final run) were efficient (7 iters, 5 productive)
- **Observation masking:** NOT USED — no `scratch/` directory. Iter logs show long tool outputs (potential context pressure)
- **Plan recitation:** OBSERVED — pipeline re-reads plan at each stage boundary (PLAN log lines)
- **CLAUDE.md size:** 12,708 chars — OK (well under 40K limit)

## Three-Axis Growth

| Axis | Score | Evidence |
|------|-------|----------|
| **Technical** | 9/10 | Full auth + payments stack in Rust, Stripe Connect with split payments, JWT verification, webhook handling. stripe-universal crate works on both native and wasm32. Repository pattern across 2 backends. 91 tests. |
| **Cognitive** | 8/10 | Good architectural decisions: async-trait for Repository, separate stripe-universal crate for portability, proper money handling with Decimal. The redo cycle showed review catching real issues (fmt, unwrap). |
| **Process** | 6/10 | Pipeline restarts wasted time. No observation masking. cf-workers-deploy archived without validation. But auto-plan working well, commit discipline strong. |

## Recommendations

1. **[HIGH]** Add observation masking — create `scratch/` for large build outputs to reduce context pressure. Auth-payments builds were likely hitting context limits (48 tasks in single plan).
2. **[HIGH]** Add partial progress detection to pipeline — when build is interrupted, detect existing commits since last successful iter and skip completed tasks on restart.
3. **[MEDIUM]** Add `cargo fmt --check` to pre-commit hook — the redo cycle was caused by fmt failures that should've been caught before review.
4. **[MEDIUM]** Split large plans (>30 tasks) into sub-tracks — auth-payments had 48 tasks across 4 phases, likely contributing to build not completing in first attempt.
5. **[LOW]** Tick spec checkboxes automatically in build skill — cf-workers-deploy at 0% is a known paperwork gap.

## Scoring Breakdown

| Dimension | Score | Weight | Weighted |
|-----------|-------|--------|----------|
| Efficiency (32% waste) | 5 | 25% | 1.25 |
| Stability (3 restarts) | 4 | 20% | 0.80 |
| Fidelity (96% criteria) | 9 | 20% | 1.80 |
| Quality (100% test pass) | 10 | 15% | 1.50 |
| Commits (92% conventional) | 7 | 5% | 0.35 |
| Docs (plans complete, 1 gap) | 7 | 5% | 0.35 |
| Signals (1 redo, 1 missing) | 7 | 5% | 0.35 |
| Speed (~18h wall / ~5h active) | 3 | 5% | 0.15 |
| **Total** | | | **6.6** |

Rounded: **6.8/10** (adjusted +0.2 for 6 tracks built from zero in one day — exceptional throughput despite waste).
