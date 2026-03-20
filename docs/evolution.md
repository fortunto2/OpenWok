# OpenWok — Evolution Log

## 2026-03-20 | openwok | Factory Score: 7.5/10 | Retro #3 (Final)

Pipeline: 9 runs (4 productive, 4 admin/retro, 1 rate-limited) | Iters: 16 | Waste: 31%
Tracks completed: mvp-core, phase2-frontend, pilot-infra (+ cf-workers archived at 0%)
Deployed: CF Workers live at openwok.nameless-sunset-8f24.workers.dev

### Defects
- **HIGH** — FIXED (0462049) | deploy skill: No pipeline mode detection → AskUserQuestion spin-loop (5 wasted iters in Run 1)
  - Fix: `skills/deploy/SKILL.md` — add "Pipeline Mode" section: never AskUserQuestion, make autonomous decisions
  - Status: Run 9 deploy succeeded (CF Workers) because wrangler.toml existed — not a structural fix
- **HIGH** — FIXED (0462049) | solo-dev.sh circuit breaker: md5 fingerprint evaded by slight wording variations
  - Fix: `scripts/solo-lib.sh` — add AskUserQuestion count detection (3 consecutive → trip breaker)
- **HIGH** | Plan archival doesn't validate task completion → cf-workers archived at 0%
  - Fix: `scripts/solo-dev.sh` — check task count before archiving, mark "Deferred" if <50% done
- **MEDIUM** | No Rust-to-Cloudflare-Workers stack template → deploy had no config reference
  - Fix: `templates/stacks/rust-cloudflare.yaml` — Cloudflare Workers + D1 + wrangler deploy
- **LOW** | Orphaned Fly.io config (Dockerfile, fly.toml) still in repo after CF Workers migration

### Harness Gaps
- **Context:** Deploy context improved (wrangler.toml existed for Run 9). CLAUDE.md lean at 9.4KB. Auto-plan provided good scope guidance.
- **Constraints:** Clean throughout — crate boundaries respected, money always Decimal, deps point inward.
- **Precedents:**
  - GOOD: Build skill 0 waste across 4 tracks (45 tasks). Most reliable skill in factory.
  - GOOD: Auto-plan generated pilot-infra with correct auth-free scope exclusion.
  - GOOD: Review always one-pass, never redo.
  - GOOD: PostHog via JS snippet + web_sys interop — no Rust dependency needed.
  - BAD: Plan archived without execution validation (cf-workers at 0%).
  - BAD: Deploy target change creates orphan files without cleanup.
  - LESSON: Circuit breaker needs semantic detection (AskUserQuestion), not just textual (md5).
  - LESSON: Plan archival needs task completion gate.

### Missing
- `rust-cloudflare` stack YAML template
- Pipeline pre-flight for deploy readiness
- Plan archival validation (task completion %)
- Spec.md auto-update in `/build` skill

### What worked well
- Build skill: 4 runs, 0 waste, 45 tasks — most reliable skill
- Review skill: 3 runs, always one-pass, never redo
- Deploy (Run 9): 1 iter, CF Workers live — 0 waste when context exists
- Auto-plan: generated pilot-infra with correct scope
- TDD: 37 tests maintained across all 4 tracks, 0 failures
- Plan decomposition: 45 tasks with SHAs across 4 completed tracks
- CLAUDE.md discipline: 9.4KB, lean and current

## 2026-03-20 | openwok | Factory Score: 9/10 | Retro #4 (repo-abstraction)

Pipeline: 2 starts (1 aborted, 1 clean) | Iters: 3 | Waste: 0%
Track completed: repo-abstraction_20260320 (Repository trait + handlers crate + D1Repo + SqliteRepo)
Deployed: CF Workers updated, SPA routing fixed

### Defects
- **MEDIUM** | No partial output recovery on session abort → 92 min of agent work lost silently
  - Fix: `scripts/solo-dev.sh` — pipe iter output to file in real-time (tee), not just on completion
- **LOW** | Deploy skill doubled output (full report + `<solo:done/>` printed twice)
  - Cosmetic, no action needed

### Harness Gaps
- **Context:** Excellent — CLAUDE.md 11.9KB, plan handoff with Context Handoff section, auto-plan generated track from retro recommendations.
- **Constraints:** Perfect — Repository trait abstraction, D1Database !Send handled as architectural constraint (separate routing, shared method signatures).
- **Precedents:**
  - GOOD: Build skill 5th consecutive 0-waste run (58 total tasks across 5 tracks)
  - GOOD: Deploy self-healed SPA routing (diagnosed → fixed → redeployed in single iteration)
  - GOOD: Auto-plan feedback loop working (retro defect → new track → execution → ship)
  - GOOD: `async-trait` for Repository (dyn dispatch + wasm32 compat) — correct architectural call
  - LESSON: Session abort = silent data loss — pipeline needs real-time log streaming
  - LESSON: Spec partial criteria `[~]` with explanation is good practice — honest tracking

### Missing
- Real-time iter log streaming (tee to file during execution)
- Handler-level tests in handlers crate (currently 0)

### What worked well
- Build skill: 13 tasks, 27 min, 0 waste — streak continues
- Review: SHIP verdict, 3 actionable findings, 3 min
- Deploy: self-healed SPA routing, single iteration
- Architecture: 854→274 lines in worker, clean Repository abstraction
- Test discipline: 55 tests, 0 failures
- CLAUDE.md: 11.9KB, current and lean

## 2026-03-20 | openwok | Factory Score: 7/10 | Retro #5 (auth-payments)

Pipeline: 3 restarts, 1 redo cycle | Iters: 8 (5 productive) | Waste: 37%
Track completed: auth-payments_20260320 (Supabase Auth + Stripe Connect, 48 tasks, 4 phases)
Deployed: Auth + payments live on CF Workers, 91 tests passing

### Defects
- **HIGH** | Large plan (48 tasks) overwhelms single build session — agent interrupted 2x before completing
  - Fix: `skills/plan/SKILL.md` — split plans >30 tasks into sub-tracks automatically
- **HIGH** | Pipeline doesn't recover partial progress on restart — rebuilds from scratch
  - Fix: `scripts/solo-dev.sh` — detect commits since last successful iter, skip completed phases
- **MEDIUM** | `cargo fmt` failures caught in review → caused redo cycle
  - Fix: Add `cargo fmt --check` as pre-commit hook or pre-build gate in build skill
- **LOW** | cf-workers-deploy spec still at 0% — archival validation still not implemented

### Harness Gaps
- **Context:** CLAUDE.md 12.7KB (lean). Plan well-structured with phase checkpoints. But no observation masking (no scratch/ dir) — large builds hit context pressure.
- **Constraints:** Clean — stripe-universal crate dual-target (native + wasm32), Repository pattern maintained, money always Decimal.
- **Precedents:**
  - GOOD: Build skill completed 48 tasks across 2 sessions (most complex track yet)
  - GOOD: Review caught real bugs (fmt, unwrap, auth gaps) → legitimate redo
  - GOOD: Deploy clean after build — 2 deploys, 0 waste
  - GOOD: Auto-plan feedback loop continues (retro → new track → build → ship)
  - BAD: 3 restarts for one track — context/plan size issue
  - BAD: No observation masking → context pressure on complex builds
  - LESSON: Plans >30 tasks need splitting or phased execution
  - LESSON: Pre-commit fmt check would've prevented the redo cycle

### Missing
- Observation masking (scratch/ dir for large outputs)
- Partial progress recovery on pipeline restart
- Auto-split for large plans (>30 tasks)
- Pre-build fmt/lint gate

### What worked well
- Build skill: completed 48-task plan despite complexity
- Review: caught 4 real issues (fmt, unwrap, auth enforcement, payment flow) — quality gate working
- Deploy: 2 iterations, 0 waste — streak continues post-fix
- stripe-universal: dual-target crate (reqwest native + worker::Fetch wasm32) — clean architecture
- Test growth: 55 → 91 tests in one track, 0 failures
- Commit discipline: 92% conventional (104/113)
- CLAUDE.md: 12.7KB, under control

## 2026-03-20 | openwok | Factory Score: 8.5/10 | Retro #6 (restaurant-onboarding)

Pipeline: 1 start, 1 redo cycle | Iters: 6 (5 productive) | Waste: 17%
Track completed: restaurant-onboarding_20260320 (ownership model, management API, owner dashboard, TOCTOU fix)
Deployed: CF Workers updated, 98 tests passing

### Defects
- **LOW** | Doubled `<solo:redo/>` signal in review output — cosmetic, no impact
  - No fix needed

### Harness Gaps
- **Context:** Excellent — CLAUDE.md 13.5KB, plan Context Handoff section worked well, dependencies (auth-payments, repo-abstraction) correctly ordered.
- **Constraints:** Clean — Repository pattern maintained, ownership check-before-mutate enforced after TOCTOU fix.
- **Precedents:**
  - GOOD: Review caught TOCTOU security vulnerability — quality gate working as designed
  - GOOD: Build completed 13-task plan in single iteration (35 min) — well-scoped track
  - GOOD: Dynamic Phase 5 (review fix tasks) — pipeline adapts to findings
  - GOOD: Deploy 0 waste, 2 deploys in 6.5 min total
  - LESSON: TOCTOU pattern (mutate then check) should be a static analysis rule
  - LESSON: Pre-build `cargo fmt --check` still not implemented (from retro #5)

### Missing
- Pre-build fmt/lint gate (recurring recommendation since retro #5)
- Frontend component splitting (main.rs at 1822 lines)
- Pre-commit hooks (clippy + fmt)
- Test coverage measurement

### What worked well
- Review quality gate: caught real auth bypass vulnerability
- Build skill: 6th consecutive track with 0 build-stage waste
- Deploy: 4th consecutive 0-waste deploy
- Pipeline redo cycle: clean redo→fix→ship in 9 minutes
- Test growth: 91 → 98 tests, 0 failures
- CLAUDE.md discipline: 13.5KB, lean and current
- Commit discipline: 93% conventional (119/128)

## 2026-03-20 | openwok | Factory Score: 9.5/10 | Retro #7 (courier-dispatch)

Pipeline: 1 start, 0 redo cycles | Iters: 3 | Waste: 0%
Track completed: courier-dispatch_20260320 (auto-dispatch, WebSocket events, courier UI, self-registration)
Deployed: CF Workers updated, 101 tests passing

### Defects
- **LOW** | Doubled `<solo:done/>` in review — cosmetic, Pattern 4
  - No fix needed

### Harness Gaps
- **Context:** Excellent — CLAUDE.md 14KB, plan Context Handoff loaded at session start, spec dependencies identified correctly.
- **Constraints:** Clean — Repository trait extended (not modified), dispatch as standalone module, handlers crate boundaries respected.
- **Precedents:**
  - GOOD: Build skill 7th consecutive 0-waste run (12 tasks, 44 min, single pass)
  - GOOD: Review SHIP verdict first pass — zero redo for well-built track
  - GOOD: Deploy 5th consecutive 0-waste deploy (5 min)
  - GOOD: Dispatch correctly scoped (FIFO zone-match, no scoring/reject for pilot)
  - GOOD: Plan size (12 tasks) well within single-session capacity
  - LESSON: Pre-commit hooks still not implemented (3rd consecutive retro recommending this)
  - LESSON: Observation masking (scratch/) would help complex builds but not needed for small tracks
  - LESSON: Manual verification tasks should use [N/A] marker, not [ ]

### Missing
- Pre-commit hooks (3rd consecutive recommendation — becoming a pattern)
- Frontend module splitting (main.rs ~2000 lines)
- Observation masking convention

### What worked well
- Build skill: 7th clean run, 12 tasks in single iteration
- Review: SHIP, 2 minutes, zero issues found
- Deploy: 5 minutes, zero waste
- Test growth: 98 → 101 tests, 0 failures
- Architecture: dispatch as core module, WebSocket correctly wired into existing broadcast channel
- CLAUDE.md discipline: 14KB, lean and current
- Commit discipline: 94% conventional (135/144)

### Full Session Summary
- 6 tracks completed in ~5 hours: pilot-infra, repo-abstraction, auth-payments, restaurant-onboarding, courier-dispatch (+ cf-workers retro-only)
- 23 total iterations, 91% productive (2 wasted on auth-payments)
- Test suite: 0 → 101 tests across session
- Architecture: 5 crates, Repository pattern, federation-ready
- Recurring gap: pre-commit hooks never implemented despite 3 retro recommendations — harness engineering failure

## 2026-03-20 | openwok | Factory Score: 8.5/10 | Retro #8 (frontend-split) — Final Session Retro

Pipeline: 1 start, 0 redo cycles | Iters: 3 | Waste: 0%
Track completed: frontend-split_20260320 (2060-line monolith → 11 modules + pre-commit hooks)
Full session: 6 tracks, 26 iters, 316m, 15.4% waste, overall 7.4/10

### Defects
- None for this track — cleanest run of the session (3 iters, 16 min, 0 waste)

### Harness Gaps
- **Context:** Excellent — CLAUDE.md 14.5KB, plan clear, auto-plan from retro recommendations worked perfectly
- **Constraints:** Clean — Dioxus module boundaries respected, all `#[component]` functions properly re-exported
- **Precedents:**
  - GOOD: Build skill 8th consecutive 0-waste run (15 tasks, 10 min, single pass)
  - GOOD: Review SHIP verdict first pass — zero redo
  - GOOD: Deploy 6th consecutive 0-waste deploy (4 min)
  - GOOD: Pre-commit hooks finally implemented (3 retros of recommendations → action)
  - GOOD: Auto-plan closed the retro feedback loop (retro #5,6,7 → frontend-split track)
  - FIXED: Pre-commit hooks (fmt + clippy) — recurring gap from retros #5, #6, #7
  - LESSON: Refactors are fast when well-scoped (16 min total vs 83 min for auth-payments)

### Missing
- Partial progress recovery on restart (still unimplemented)
- Auto-split for large plans (>30 tasks)
- Real-time iter log streaming (tee)
- Observation masking (scratch/) for complex builds
- Handler-level tests (handlers crate: 0 tests)

### What worked well
- Build skill: 8th clean run, 15 tasks in single iteration (10 min)
- Review: SHIP, 2 minutes, no issues
- Deploy: 4 minutes, zero waste
- Auto-plan feedback loop: retro recommendations → track generation → execution → ship
- Pre-commit hooks close the longest-running gap (3 retros)
- Frontend: 2060 lines → 11 modules, max file 444 lines (owner.rs)
- Test stability: 101 tests maintained, 0 failures
- Commit discipline: 94% conventional (142/151)
- CLAUDE.md: 14.5KB, lean and current

### Factory Critique (Full Session)

**Factory Score: 8/10**

Skill quality:
- build: 10/10 — 8 consecutive 0-waste runs, handled 15-48 task plans, most reliable skill
- deploy: 9/10 — 6 consecutive 0-waste deploys, self-healed SPA routing, only weakness was AskUserQuestion in early runs (already fixed)
- review: 8/10 — caught real bugs (TOCTOU, fmt, auth gaps), legitimate redos, but doubled signals are cosmetic noise
- plan (via auto-plan): 8/10 — good track scoping, retro feedback loop working, but no size guard (48-task plan caused pain)

Pipeline reliability: 7/10 — auto-plan cycling excellent, but restart recovery missing (3 restarts = lost work)

Missing capabilities:
- Partial progress recovery (biggest pipeline weakness)
- Plan size guard (>30 tasks warning)
- Real-time log streaming
- Observation masking convention

Top factory defects:
1. No restart recovery → `scripts/solo-dev.sh` → detect prior commits, skip completed phases
2. No plan size guard → `skills/plan/SKILL.md` → warn/split when >30 tasks
