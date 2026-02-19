# PR-EVP-06 Follow-up: Stub Component Unblock

## Problem
Strict `greentic-pack resolve/build` still fails for packs that reference placeholder stubs because the stub wasm must satisfy:

- export `greentic:component/component-descriptor@0.6.0` (and related 0.6 interfaces),
- instantiate in the pack linker context without unsupported WASI imports.

Current errors:
- `no exported instance named greentic:component/component-descriptor@0.6.0`
- `component imports instance wasi:io/error@0.2.x ... resource implementation is missing`

## What was verified
- A minimal 0.6 descriptor component can be compiled and does export the required interfaces.
- That component still imports WASI (`wasi:io/*`, `wasi:cli/*`) and fails to instantiate in current pack linker context.

## Required unblock
- Produce a no-WASI-import component that exports:
  - `component-descriptor`
  - `component-schema`
  - `component-runtime`
  - `component-qa`
  - `component-i18n`
  from `greentic:component@0.6.0`.

## Suggested implementation path
1. Build a tiny dedicated crate for placeholder stubs (separate from provider-core workspace coupling).
2. Compile in a mode that avoids WASI imports for the emitted component.
3. Replace:
   - `packs/components/stub.wasm`
   - `packs/components/templating.handlebars/stub.wasm`
4. Remove deferred-failure allowances from `ci/no_hand_rolling.sh` for descriptor/WASI mismatch.
5. Re-run:
   - `bash ci/no_hand_rolling.sh`
   - ensure strict `resolve/build/doctor` passes without exceptions.
