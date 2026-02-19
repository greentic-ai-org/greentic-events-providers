# PR-11C.md (greentic-events-providers)
# Title: Add provider-core dummy events provider component + pack fixture (CI deterministic)

## Goal
Create a deterministic provider-core events provider to unblock runner/deployer/store testing
and to avoid half-migration.

This provider does NOT contact external services.

## Deliverables
1) WASM component (wasm32-wasip2) implementing provider-core:
- crate: `components/events-provider-dummy/`
- provider_type: `events.dummy`
- ops: `publish` (and optionally `echo` for simple tests)
Behavior:
- invoke("publish", input_json):
  - validates JSON parses
  - returns output_json with:
    - receipt_id = stable id (uuid or hash of input)
    - status = "published"
- Additionally (strongly recommended):
  - store the publish input into state-store under a fixed key:
    - `events/dummy/last_published.json`
  This enables integration tests to assert publish happened.
- validate-config accepts any config that parses
- healthcheck returns ok

2) Pack fixture:
- `packs/events-dummy.gtpack/` (or your pack format)
Includes:
- config schema:
  - `schemas/events/dummy/config.schema.json`
- provider extension inline:
  - key `greentic.ext.provider`
  - runtime.world pinned to `greentic:provider/schema-core@1.0.0`
- component artifact reference to the built WASM

3) Tests
- Build test for the WASM component
- Pack validation test (extension + schema exists)
- Smoke test invoking publish and asserting:
  - output contains receipt_id
  - state-store key updated (if you implement state-store write)

## Acceptance criteria
- CI can run provider-core event publish flows without network.
- This is the baseline fixture for runner PR-08 and integration PR-14.
