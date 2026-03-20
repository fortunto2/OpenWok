# Pipeline Retro: openwok (2026-03-20) — Full Session

## Overall Score: 7.4/10

Pipeline session covering 6 plan tracks auto-planned and executed sequentially: pilot-infra, repo-abstraction, auth-payments, restaurant-onboarding, courier-dispatch, frontend-split.

## Pipeline Efficiency

| Metric | Value | Rating |
|--------|-------|--------|
| Total iterations | 26 | |
| Productive iterations | 22 (84.6%) | YELLOW |
| Wasted iterations | 4 (15.4%) | YELLOW |
| Pipeline restarts | 3 | YELLOW |
| Max-iter hits | 0 | GREEN |
| Redo cycles | 2 | YELLOW |
| Total duration | 316m (~5h16m) | RED |
| Plans cycled | 6 | GREEN |
| Avg duration/track | 53m | GREEN |

## Per-Stage Breakdown

| Stage | Attempts | Successes | Waste % | Notes |
|-------|----------|-----------|---------|-------|
| build | 10 | 8 | 20% | 2 wasted on auth-payments (build not completing) |
| deploy | 8 | 8 | 0% | Zero-waste streak all 6 tracks |
| review | 8 | 6 | 25% | 2 redo cycles (auth-payments, restaurant-onboarding) |

## Per-Track Breakdown

| Track | Starts | Iters | Productive | Wasted | Waste% | Duration | Redo |
|-------|--------|-------|------------|--------|--------|----------|------|
| pilot-infra | 1 | 3 | 3 | 0 | 0% | 62m | 0 |
| repo-abstraction | 2 | 3 | 3 | 0 | 0% | 42m | 0 |
| auth-payments | 3 | 8 | 5 | 3 | 37.5% | 83m | 1 |
| restaurant-onboarding | 1 | 6 | 5 | 1 | 16.7% | 58m | 1 |
| courier-dispatch | 1 | 3 | 3 | 0 | 0% | 55m | 0 |
| frontend-split | 1 | 3 | 3 | 0 | 0% | 16m | 0 |
| **TOTAL** | **9** | **26** | **22** | **4** | **15.4%** | **316m** | **2** |

## Failure Patterns

### Pattern 1: Large Plan Overwhelms Session (auth-payments)

- **Occurrences:** 3 iterations across 2 restarts
- **Root cause:** auth-payments had 48 tasks — too large for a single build session. Agent was interrupted/aborted twice before completing, losing partial progress each time. Build couldn't signal done because not all tasks were finished.
- **Wasted:** 2 build iterations (run 2 iter 1, run 3 iter 1 = continuing with no state file)
- **Fix:** `skills/plan/SKILL.md` — auto-split plans >30 tasks into sub-tracks. Pipeline `scripts/solo-dev.sh` — detect commits since last successful iter, skip completed phases on restart.

### Pattern 2: Redo Cycles from Preventable Issues

- **Occurrences:** 2 cycles (auth-payments: cargo fmt failures; restaurant-onboarding: TOCTOU vulnerability)
- **Root cause:** auth-payments redo was caused by `cargo fmt` failures — would have been prevented by pre-commit hooks. Restaurant-onboarding redo caught a real TOCTOU security bug — legitimate quality gate.
- **Wasted:** 1 iteration (auth-payments redo was preventable; restaurant-onboarding redo was productive)
- **Fix:** Pre-commit hooks now implemented (frontend-split track). Future auth-style redos prevented.

### Pattern 3: Silent Session Abort (repo-abstraction)

- **Occurrences:** 1 (run 1 at 14:01, no ITER result, restarted at 15:33 — 1.5h gap)
- **Root cause:** Session interrupted without capturing output. No ITER line logged, no partial work recovered.
- **Wasted:** ~92 minutes of wall-clock time (no iteration logged, so 0 counted waste, but real time lost)
- **Fix:** `scripts/solo-dev.sh` — pipe iter output to file in real-time (tee), not just on completion.

## Plan Fidelity

| Track | Criteria Met | Tasks Done | SHAs | Rating |
|-------|-------------|------------|------|--------|
| pilot-infra | 100% (10/10) | 100% (31/31) | yes | GREEN |
| repo-abstraction | 100% (8/8) | 90% (28/31) | yes | GREEN |
| auth-payments | 100% (11/11) | 96% (46/48) | yes | GREEN |
| restaurant-onboarding | 100% (10/10) | 83% (29/35) | yes | YELLOW |
| courier-dispatch | 100% (11/11) | 89% (24/27) | yes | GREEN |
| frontend-split | 100% (10/10) | 100% (40/40) | yes | GREEN |
| **Average** | **100%** | **93%** | all yes | GREEN |

Notes: All acceptance criteria met across all tracks. Incomplete tasks are mostly manual verification items ([N/A] markers used inconsistently) or deferred sub-items.

## Code Quality (Quick)

- **Tests:** 101 passed, 0 failed, 2 ignored (doc-tests) — GREEN
- **Build:** PASS (clippy clean, zero warnings) — GREEN
- **Commits:** 151 total, 142 conventional (94%) — GREEN
- **Test growth across session:** 37 → 55 → 91 → 98 → 101 (from evolution.md progression)

## Context Health

- **Iteration quality trend:** STABLE — later tracks (courier-dispatch, frontend-split) were cleaner than earlier ones (auth-payments). No degradation detected.
- **Observation masking:** NOT USED — no `scratch/` directory. Large builds (auth-payments 48 tasks) would have benefited from this.
- **Plan recitation:** OBSERVED — auto-plan logs show plan.md read at each track start.
- **CLAUDE.md size:** 14,512 chars — OK (under 40K threshold)

## Scoring Breakdown

| Dimension | Weight | Score | Weighted |
|-----------|--------|-------|----------|
| Efficiency (waste %) | 25% | 7 | 1.75 |
| Stability (restarts) | 20% | 5 | 1.00 |
| Fidelity (criteria met) | 20% | 9 | 1.80 |
| Quality (test pass rate) | 15% | 10 | 1.50 |
| Commits (conventional %) | 5% | 7 | 0.35 |
| Docs (plan completeness) | 5% | 10 | 0.50 |
| Signals (clean handling) | 5% | 8 | 0.40 |
| Speed (total duration) | 5% | 2 | 0.10 |
| **Overall** | **100%** | | **7.4** |

## Three-Axis Growth

| Axis | Score | Evidence |
|------|-------|----------|
| **Technical** (code, tools, architecture) | 9/10 | 5-crate workspace, Repository trait abstraction, Stripe Connect integration, WebSocket events, auto-dispatch, 14-route Dioxus SPA split into 11 modules, 101 tests, pre-commit hooks |
| **Cognitive** (understanding, strategy, decisions) | 8/10 | Retro feedback loop working (3 retro recommendations → pre-commit hooks finally implemented), plan scoping learned (smaller tracks after auth-payments pain), redo cycles catching real bugs (TOCTOU) |
| **Process** (harness, skills, pipeline, docs) | 7/10 | Auto-plan cycling works, deploy 0-waste streak, but restart recovery still missing, observation masking not adopted, pre-commit hooks delayed by 3 retros |

## Recommendations

1. **[CRITICAL]** Partial progress recovery on restart — pipeline loses all work when session aborts. Detect commits since last ITER, skip completed phases. File: `scripts/solo-dev.sh`
2. **[HIGH]** Auto-split plans >30 tasks — auth-payments (48 tasks) caused 3 restarts and 37% waste. File: `skills/plan/SKILL.md`
3. **[MEDIUM]** Real-time iter log streaming — pipe output via `tee` during execution, not just on completion. File: `scripts/solo-dev.sh`
4. **[MEDIUM]** Observation masking convention — `scratch/` dir for large build outputs to reduce context pressure. File: `skills/build/SKILL.md`
5. **[LOW]** Plan archival validation — check task completion % before archiving (cf-workers at 0% is still in plan-done). File: `scripts/solo-dev.sh`
6. **[LOW]** Handler-level tests in handlers crate (currently 0 tests there). File: `crates/handlers/`

## Suggested Patches

### Patch 1: `skills/plan/SKILL.md` — Add plan size warning

**What:** Warn when plan exceeds 30 tasks, suggest splitting into sub-tracks
**Why:** auth-payments 48 tasks caused 3 restarts (Pattern 1)

### Patch 2: `scripts/solo-dev.sh` — Add progress recovery on restart

**What:** On START, check `git log --oneline` for commits since last ITER, skip completed phases
**Why:** Repo-abstraction lost 92 minutes, auth-payments rebuilt from scratch twice (Pattern 3)

### Patch 3: Pre-commit hooks — DONE (this session)

**What:** `.githooks/pre-commit` with `cargo fmt --check && cargo clippy --all`
**Why:** Would have prevented auth-payments redo cycle (Pattern 2). Implemented in frontend-split track.

---

Generated by `/retro` on 2026-03-20. Pipeline session: 6 tracks, 26 iterations, 316 minutes.
