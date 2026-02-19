# Contributing

## Generated Artifacts Policy
- Do not hand-edit generated artifacts.
- Use the Greentic CLIs to regenerate/update artifacts:
  - `greentic-component`
  - `greentic-flow`
  - `greentic-pack`

## Validation Expectations
- Keep the repository CBOR-first for pack artifacts.
- Do not commit banned generated JSON artifacts:
  - `pack.manifest.json`
  - `pack.lock.json`
- Flow sidecars (`*.resolve.json`, `*.resolve.summary.json`) are part of the current
  `greentic-flow`/`greentic-pack` toolchain and are validated during regeneration checks.
- Regeneration must be clean (`git diff --exit-code` after tooling runs).
