# PR-EVP-06: CI enforcement for “no hand-rolling” (regen + banlists)

## Repo
`greentic-events-providers`

## Goal
Guarantee the repo never accumulates hand-authored generated artifacts.

## Checks
1) Ban-list patterns:
- `pack.manifest.json`
- `pack.lock.json`
- Flow sidecars (`*.resolve.json`, `*.resolve.summary.json`) are treated as generated flow sidecar inputs for current tooling and are validated by CI regen checks.

2) Regeneration must be clean
In CI:
- `greentic-component build` / `doctor`
- `greentic-flow doctor`
- `greentic-pack update`, `resolve`, `build`, `doctor`
Then:
- `git diff --exit-code`

3) Tool version reporting
- print `--version` for all tools in CI logs

4) Tool version policy
- Install latest `greentic-component`, `greentic-flow`, and `greentic-pack` in CI.
- Always print tool versions in logs for traceability.
- If tool execution fails, surface the failing command and version context as a potential tool bug report payload.

5) Workflow location and triggers
- Place checks in a dedicated workflow under `.github/workflows/` (or existing build workflow if already standardized).
- Enforce on both:
  - `pull_request`
  - `push` to `main`

## Acceptance criteria
- CI fails if banned artifacts exist
- CI fails if running tooling changes tracked files
- CI runs against latest available tool versions and reports exact versions used

## Implementation status (2026-02-16)
- Implemented:
  - `ci/no_hand_rolling.sh` created and wired into `ci/local_check.sh`.
  - `.github/workflows/tests.yaml` installs latest `greentic-pack`, `greentic-flow`, and `greentic-component` each run.
  - Banlist enforcement for:
    - `pack.manifest.json`
    - `pack.lock.json`
  - Temp-workspace regen pipeline runs:
    - `greentic-pack update`
    - `greentic-pack resolve`
    - `greentic-flow doctor` with deterministic sidecar auto-bind when missing
    - `scripts/build_packs.sh` + `greentic-pack doctor --validate`
  - Dirty-worktree-safe regen assertion:
    - compares git status snapshot before/after script, fails only on new drift introduced by checks.

- Resolved:
  - Added dedicated no-WASI 0.6 self-describing stub component crate:
    - `components/stub-component-v060`
  - Added deterministic stub artifact generator:
    - `scripts/build_stub_components.sh`
  - Switched source-pack placeholder references to per-id stub components under:
    - `packs/components/stubs/*.wasm`
  - Hardened stub QA/i18n surfaces so `greentic-pack doctor --validate` passes in temp regen flow.
  - Removed build-time placeholder bypass from `ci/no_hand_rolling.sh`; pack build/doctor failures are hard-fail again.
  - Added Greentic CLI failure diagnostics in `ci/no_hand_rolling.sh` to print command + tool versions for bug reporting.
