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
