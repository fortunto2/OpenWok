# Pipeline Retro: openwok — repo-abstraction_20260320 (2026-03-20)

## Overall Score: 8.9/10

### Dimension Breakdown

| Dimension | Weight | Score | Weighted | Rationale |
|-----------|--------|-------|----------|-----------|
| Efficiency | 25% | 10 | 2.50 | 3/3 completed iterations productive (0% waste) |
| Stability | 20% | 8 | 1.60 | 1 restart (aborted build at 14:01, clean restart at 15:33), 0 maxiter hits |
| Fidelity | 20% | 8 | 1.60 | 89% criteria (8/9 + 1 partial), 87% tasks (13/15, 2 skipped — no wrangler creds) |
| Quality | 15% | 10 | 1.50 | 55/55 tests pass, clippy clean, build clean, worker wasm32 clean |
| Commits | 5% | 10 | 0.50 | 100% conventional (13/13 this track) |
| Docs | 5% | 10 | 0.50 | All phases with checkpoints, all tasks with SHAs, spec updated |
| Signals | 5% | 7 | 0.35 | Clean signals (3/3), doubled output in deploy iter (cosmetic) |
| Speed | 5% | 6 | 0.30 | 37 min clean run, but ~129 min wall clock including aborted run |

## Pipeline Efficiency

| Metric | Value | Rating |
|--------|-------|--------|
| Total iterations | 3 | |
| Productive iterations | 3 (100%) | GREEN |
| Wasted iterations | 0 (0%) | GREEN |
| Pipeline starts | 2 (1 aborted, 1 clean) | YELLOW |
| Max-iter hits | 0 | GREEN |
| Aborted invocations | 1 (~92 min, build at 14:01) | RED |
| Clean run duration | 37 min (15:33→16:10) | GREEN |
| Total wall clock | ~129 min (14:01→16:10) | YELLOW |

## Per-Stage Breakdown

| Stage | Attempts | Successes | Waste % | Duration | Notes |
|-------|----------|-----------|---------|----------|-------|
| build (attempt 1) | 1 | 0 | 100% | ~92 min | Aborted — no ITER saved, session terminated |
| build (attempt 2) | 1 | 1 | 0% | 27 min | Clean: 13 tasks, 4 phases, 12 commits |
| deploy | 1 | 1 | 0% | 8 min | Fixed SPA routing bug in-flight |
| review | 1 | 1 | 0% | 3 min | Verdict: SHIP, 3 minor issues found |

## Failure Patterns

### Pattern 1: Aborted Build Session (~92 min lost)

- **Occurrences:** 1 (14:01:33 → 15:33:16)
- **Root cause:** Build invocation at 14:01 ran for ~92 minutes with no ITER result. Session was either user-cancelled or crashed. No output preserved — no iter log was saved.
- **Wasted:** ~92 min of compute time (no iterations, but wall clock burn)
- **Impact:** Speed penalty only — the second build started fresh and completed cleanly
- **Fix:** Pipeline should save partial iter logs when a session is terminated mid-execution. Currently, if the build agent is killed before outputting `<solo:done/>`, all work from that session is lost silently.

### Pattern 2: Deploy Doubled Output (Cosmetic)

- **Occurrences:** 1 (iter-002-deploy)
- **Root cause:** Deploy skill printed the full deployment report, then `<solo:done/>`, then repeated the entire report + `<solo:done/>` again. Likely context window issue — the agent forgot it already printed the report.
- **Impact:** None — doubled signal is harmless, just noise in the log
- **Fix:** No action needed. Consistent with known Pattern 4 (Doubled Signal) from failure catalog.

### Pattern 3: SPA Route 404 (Self-Healed)

- **Occurrences:** 1 (during deploy verification)
- **Root cause:** Worker router caught all requests; non-`/api/` paths returned 404 because ASSETS binding wasn't configured. The `[assets]` section in `wrangler.toml` lacked `binding = "ASSETS"`.
- **Impact:** Required redeploy within same iteration — self-healed
- **Status:** FIXED (commit 3ca61fd) — Deploy skill correctly diagnosed, patched, rebuilt, redeployed, and verified. Good autonomous behavior.

## Plan Fidelity

| Track | Criteria Met | Tasks Done | SHAs | Rating |
|-------|-------------|------------|------|--------|
| repo-abstraction_20260320 | 89% (8/9 + 1 partial) | 87% (13/15) | Yes (all 13) | GREEN |

**Partial criterion:** Worker doesn't share handlers crate (D1Database is !Send, axum handlers require Send). D1Repo implemented with same method signatures but worker uses `worker::Router` instead of shared axum handlers. Correctly documented as architectural constraint, not a gap.

**Skipped tasks (2):** `wrangler dev` local D1 testing and live URL pre-deploy verification — both require wrangler credentials not available to autonomous agent. Verified post-deploy instead.

## Code Quality (Quick)

- **Tests:** 55 pass (18 api + 37 core), 0 fail GREEN
- **Clippy:** 0 warnings (workspace) GREEN
- **Build:** PASS (workspace + wasm32) GREEN
- **Commits:** 90 total, 81 conventional (90%) GREEN
- **This track:** 13 commits, 13/13 conventional (100%) GREEN
- **Issues found by review:**
  - 2x `unwrap()` in handlers (restaurants.rs:59, couriers.rs:36)
  - 1x unused import (sqlite_repo.rs:705)

## Context Health

- Iteration quality trend: STABLE — no degradation, all 3 iters one-pass
- Observation masking: NOT USED (no scratch/ directory — iter logs manageable)
- Plan recitation: OBSERVED — build re-read plan.md at task boundaries
- CLAUDE.md size: 11,942 chars — GREEN (well within 40K budget)

## Three-Axis Growth

| Axis | Score | Evidence |
|------|-------|----------|
| **Technical** | 8/10 | Repository trait abstraction, shared handlers crate, D1Repo with same SQL, worker 854→274 lines. Clean architecture: `core ← handlers ← api/worker`. |
| **Cognitive** | 9/10 | Correctly identified D1Database !Send as architectural constraint. Adapted plan in-flight (D1Repo has same methods but separate routing). Good scope management — no scope creep. |
| **Process** | 8/10 | 0% iteration waste, clean signals, deploy self-healed SPA routing. Aborted run is sole blemish. Auto-plan generated correct track from retro recommendations. |

## Recommendations

1. **[MEDIUM]** Fix 2 `unwrap()` calls in handlers — `crates/handlers/src/restaurants.rs:59` and `crates/handlers/src/couriers.rs:36`. Replace with proper `match` on `RepoError` → HTTP status code.

2. **[MEDIUM]** Remove unused import `CreateCourierRequest` in `crates/api/src/sqlite_repo.rs:705`.

3. **[MEDIUM]** Add handler-level tests to `crates/handlers/` — currently 0 tests. At minimum: test each handler returns correct status codes for success/not-found/invalid-input.

4. **[LOW]** Add README.md with setup/run/deploy instructions for new contributors.

5. **[LOW]** Add pre-commit hooks (clippy + fmt) — prevents lint regressions.

6. **[LOW]** Pipeline: save partial iter logs on session abort — when build agent is killed, at least save what tools were called and partial output.

## Suggested Patches

### Patch 1: handlers/restaurants.rs — Remove unwrap()

**What:** Replace `unwrap()` with proper error handling on `create_restaurant`
**Why:** Review found 2 `unwrap()` calls that could panic on DB errors

```diff
- let restaurant = repo.create_restaurant(payload).await.unwrap();
+ let restaurant = repo.create_restaurant(payload).await.map_err(|e| {
+     (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
+ })?;
```

### Patch 2: handlers/couriers.rs — Remove unwrap()

**What:** Same fix for `create_courier`
**Why:** Same pattern as Patch 1

```diff
- let courier = repo.create_courier(payload).await.unwrap();
+ let courier = repo.create_courier(payload).await.map_err(|e| {
+     (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
+ })?;
```

### Patch 3: sqlite_repo.rs — Remove unused import

**What:** Clean up unused `CreateCourierRequest` import
**Why:** Clippy warning in test build

```diff
- use crate::sqlite_repo::SqliteRepo;
- use openwok_core::types::{
-     CreateCourierRequest, CreateMenuItemRequest, CreateOrderItemRequest, CreateOrderRequest,
+ use openwok_core::types::{
+     CreateMenuItemRequest, CreateOrderItemRequest, CreateOrderRequest,
```

## Factory Critique

### Factory Score: 9/10 (up from 7.5)

### Skill Quality

| Skill | Score | Assessment |
|-------|-------|-----------|
| build | 10/10 | Flawless: 13 tasks in 27 min, all SHAs annotated, proper phase gates. Read plan at boundaries. |
| deploy | 8/10 | Good: deployed + fixed SPA routing bug autonomously. Doubled output (cosmetic). Successfully built, deployed, verified, diagnosed, fixed, redeployed. |
| review | 10/10 | Clean: 3 min, correct SHIP verdict, identified 3 real issues, verified production. |

### Pipeline Reliability: 8/10

- Stage gating: correct
- State tracking: correct (.solo/states/ markers)
- Signal handling: clean (3/3)
- Restart recovery: clean (second build picked up correctly)
- Gap: aborted session at 14:01 lost ~92 min with no output

### Top Factory Defects

1. **[MEDIUM]** No partial output recovery on session abort — 92 min of agent work lost silently
   - Fix: `scripts/solo-dev.sh` — pipe iter output to file in real-time (tee), not just on completion
2. **[LOW]** Deploy skill outputs doubled report (context window artifact)
   - Fix: Not actionable — cosmetic issue from long deploy sessions

### Harness Evolution

**Context:** Excellent. CLAUDE.md at 11.9KB, repo pattern well-documented, plan handoff clean. Auto-plan correctly generated this track from previous retro recommendations.

**Constraints:** Perfect. Repository trait in core, handlers crate generic over it, api and worker implement separately. D1Database !Send limitation correctly handled as architectural constraint rather than forced workaround.

**Precedents:**
- GOOD: Build skill continues streak: 5 runs, 0 waste, 58 total tasks completed
- GOOD: Deploy self-healed SPA routing (diagnosed → fixed → redeployed in single iteration)
- GOOD: Auto-plan → retro recommendation → auto-plan cycle working (cf-workers archive defect → repo-abstraction track)
- GOOD: Spec partial criteria notation `[~]` with explanation — honest tracking
- LESSON: Session abort = silent data loss. Pipeline needs real-time log streaming.
- LESSON: `async-trait` over native async-in-trait was correct call for dyn dispatch + wasm32

### What Worked Well

- Build skill: 5th consecutive flawless run (13 tasks, 27 min, 0 waste)
- Review skill: 4th consecutive one-pass (SHIP with actionable findings)
- Deploy self-healing: found + fixed SPA routing in single iteration
- Auto-plan feedback loop: retro → defect → auto-plan → track → execution → ship
- Architecture quality: Repository pattern clean, 854→274 lines, proper trait abstraction
- Test discipline: 55 tests maintained, 0 failures
