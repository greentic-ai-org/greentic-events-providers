# Repository Overview

## 1. High-Level Purpose
- Hosts reusable Greentic events providers as Rust/WASM components plus deployable packs for `greentic-events` and `greentic-deployer`.
- Covers provider families: webhook, email (MS Graph/Gmail), SMS (Twilio), timer, plus a deterministic dummy provider for CI/integration flows.
- Uses Rust 2024 (workspace MSRV/toolchain 1.90) and Greentic `0.4` ecosystem crates.

## 2. Main Components and Functionality
- **Path:** `crates/provider-core`
  **Role:** Shared domain and secrets utilities.
  **Key functionality:** Common configs/errors/event helpers; tenant-scoped secret key helpers; secrets-store abstraction (`SecretProvider`) and standardized metadata-only secret events (`greentic.secrets.put|delete|rotate.*|missing.detected`).

- **Path:** `crates/provider-webhook`
  **Role:** Webhook source/sink mapping logic.
  **Key functionality:** Maps inbound HTTP requests into event envelopes, route/topic resolution, metadata propagation, optional signing-secret resolution via secrets-store, and outbound webhook request building.

- **Path:** `crates/provider-email`
  **Role:** Email source/sink mapping logic (MS Graph + Gmail).
  **Key functionality:** Inbound email to event mapping, outbound provider detection from topics, send-request construction, and provider secret resolution/secret event emission.

- **Path:** `crates/provider-sms`
  **Role:** Twilio SMS source/sink mapping logic.
  **Key functionality:** Twilio webhook payload to event mapping, outbound Twilio send request construction, auth-token resolution through secrets-store, and missing-secret event emission.

- **Path:** `crates/provider-timer`
  **Role:** Scheduler/timer source logic.
  **Key functionality:** Fires configured schedules into events with deterministic payload/metadata wiring.

- **Path:** `components/events-provider-*`
  **Role:** Provider-core WASM component crates for deployable provider types.
  **Key functionality:** Implement provider-core world operations (`describe`, `validate-config`, `healthcheck`, `invoke`), including deterministic `publish` behavior and host state writes for dummy/email/sms/timer components.

- **Path:** `crates/sbom-patch`
  **Role:** Build helper binary.
  **Key functionality:** Patches generated pack SBOM/manifests with schema artifacts after `greentic-pack build`.

- **Path:** `packs/events-*`
  **Role:** Pack sources and generated setup artifacts.
  **Key functionality:** Pack definitions for `events-email`, `events-sms`, `events-webhook`, `events-timer`, and `events-dummy`, including schemas, fixtures, setup WAT outputs, and flow files (`*.ygtc`, resolved JSON summaries).

- **Path:** `packs/components`
  **Role:** Shared component assets consumed by pack builds.
  **Key functionality:** Built provider WASM artifacts, template manifests/schemas, and stub/template helper WASM used by pack flow templates.

- **Path:** `fixtures/packs/*`
  **Role:** Deterministic pack fixtures for tests.
  **Key functionality:** Fixture lockfiles/artifacts for provider packs and a `secrets_events_smoke` fixture validating secret requirements/events behavior.

- **Path:** `scripts/build_packs.sh`, `scripts/provision_conformance.sh`, `scripts/publish_packs_oci.sh`, `ci/local_check.sh`
  **Role:** Local CI/build/release orchestration.
  **Key functionality:** Build all packs with `greentic-pack`, run provisioning conformance diffs against fixtures, and publish OCI pack artifacts.

- **Path:** `.github/workflows/tests.yaml`, `.github/workflows/publish_packs.yml`
  **Role:** CI and release automation.
  **Key functionality:** Unit/lint/test/pack/provisioning checks, optional live integration tests via env-gated secrets, and GHCR pack publishing.

## 3. Work In Progress, TODOs, and Stubs
- Pack flow templates intentionally include stub components (`packs/components/stub.wasm`, `packs/components/templating.handlebars/stub.wasm`) and resolved flow placeholders; these are scaffold/default wiring points rather than full bespoke business flows.
- README still references a legacy path (`packs/events/`) while current packs live under `packs/events-*`.

## 4. Broken, Failing, or Conflicting Areas
- No hard failures are documented in-repo.
- Health status is primarily enforced through `ci/local_check.sh` and GitHub Actions; this overview update did not execute the full test/build pipeline in this edit pass.

## 5. Notes for Future Work
- Keep docs and overview in sync with actual pack/workflow paths (`packs/events-*`, current workflow filenames).
- If flow scaffolds are promoted to production-grade defaults, replace remaining stub/template nodes with real operator chains and update fixtures accordingly.
- Continue expanding deterministic coverage around pack metadata (`secret_requirements`, provisioning outputs, subscriptions outputs) as pack schemas evolve.
