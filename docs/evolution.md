# OpenWok — Evolution Log

## 2026-03-20 | openwok | Factory Score: 6.5/10

Pipeline: setup→plan→build→deploy→review | Iters: 11 | Waste: 45%

### Defects
- **HIGH** | deploy skill: No pipeline mode detection → AskUserQuestion spin-loop (5 wasted iters, 45% of pipeline)
  - Fix: `skills/deploy/SKILL.md` — add "Pipeline Mode" section: never AskUserQuestion, make autonomous decisions
- **HIGH** | solo-dev.sh circuit breaker: md5 fingerprint evaded by slight wording variations
  - Fix: `scripts/solo-dev.sh` — add AskUserQuestion count detection (3 consecutive → trip breaker)
- **MEDIUM** | solo-dev.sh state file: "Skipped" content accepted as "stage complete"
  - Fix: `scripts/solo-dev.sh` — validate state file content, warn on "Skipped"
- **MEDIUM** | No rust-native stack template → deploy had no config reference
  - Fix: `templates/stacks/rust-native.yaml` — create with Dockerfile + fly.toml defaults

### Harness Gaps
- **Context:** Deploy stage lacked stack YAML for Rust projects → no deploy strategy reference
- **Constraints:** None — crate boundaries respected throughout
- **Precedents:** TDD with Decimal money = solid pattern. Phase2 build+review = 0 waste (model for future runs)

### Missing
- `rust-native` stack YAML template
- Pipeline pre-flight for deploy readiness (CLI + auth check before entering deploy stage)
- Spec.md auto-update in `/build` skill

### What worked well
- Setup→plan→build chain: 3 iters, 20 min, 0 waste — excellent
- Phase2 build+review: 2 iters, 40 min, 0 waste — flawless
- TDD: 37 tests, 0 failures, clippy clean
- Plan decomposition: 29 tasks across 6 phases with SHAs on every task
- Rate limit handling: detected and waited correctly
