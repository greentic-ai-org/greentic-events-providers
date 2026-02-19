# PR-EVP-01: Bootstrap greentic-events-providers with tooling-only workflow

## Repo
`greentic-events-providers`

## Goal
Create/standardize the repo so **nothing is hand-rolled**:
- packs/components/flows generated and maintained using:
  - `greentic-pack`
  - `greentic-flow`
  - `greentic-component`
- CBOR-first: `pack.manifest.cbor`, `pack.lock.cbor`
- CI enforces a clean regen diff

## Deliverables
- Repo structure:
  - `components/`
  - `packs/`
  - `ci/` scripts
- CI workflow:
  - build components (`greentic-component build`)
  - component doctor (`greentic-component doctor`)
  - flow doctor (`greentic-flow doctor`)
  - pack update/resolve/build/doctor (`greentic-pack update/resolve/build/doctor`)
  - `git diff --exit-code` after regeneration steps

## Policy/guardrails
- Fail CI if any of these exist:
  - `pack.manifest.json`
  - `pack.lock.json`
  - `*.resolve.json`, `*.resolve.summary.json`
- Add `CONTRIBUTING.md` stating: “Do not edit generated artifacts; use the CLIs.”
- Standardize lifecycle naming across packs/components as:
  - `default`, `setup`, `update`, `remove` (do not use `upgrade` in new assets)

## Acceptance criteria
- Fresh clone + CI produces identical generated artifacts (no diff)
- Packs build with `pack.lock.cbor` present and validated
