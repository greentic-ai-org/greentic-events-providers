# PR-EVP Execution Backlog

## Scope Baseline (Locked)
- Email vendor first target: SendGrid Inbound Parse.
- Canonical HTTP ingress op id: `ingest_http`.
- Canonical lifecycle naming: `default/setup/update/remove` (no new `upgrade`).
- Canonical ingress route pattern: `/v1/{domain}/ingress/{provider}/{tenant}/{team?}/{handler?}`.
- Canonical event envelope: Events V1 (`event_id`, `event_type`, `occurred_at`, `source`, `scope`, `payload`, with optional `http`/`raw`).
- Twilio `send_sms`: optional for MVP behavior, but op must exist (can be explicit not-enabled stub).
- CI no-hand-rolling checks: required on `pull_request` and `push` to `main`.
- Tooling versions: always install/use latest (`greentic-component`, `greentic-flow`, `greentic-pack`) and log versions.

## Delivery Order
1. PR-EVP-01 bootstrap/tooling baseline
2. PR-EVP-06 CI no-hand-rolling enforcement
3. PR-EVP-02 webhook provider
4. PR-EVP-03 timer provider
5. PR-EVP-04 Twilio SMS provider
6. PR-EVP-05 SendGrid email provider

## Dependency Graph
- PR-EVP-01 -> PR-EVP-06
- PR-EVP-01 -> PR-EVP-02
- PR-EVP-01 -> PR-EVP-03
- PR-EVP-01 -> PR-EVP-04
- PR-EVP-01 -> PR-EVP-05
- PR-EVP-06 should land before or with PR-EVP-02 onward (to guard generated artifacts continuously).
- PR-EVP-05 depends on PR-EVP-02 ingress conventions and envelope conventions.

## Status Tracker (2026-02-16)
- [x] Repo baseline directories exist (`components/`, `packs/`, `ci/`).
- [x] Added `CONTRIBUTING.md` with generated-artifact policy.
- [x] Added dedicated no-hand-rolling script: `ci/no_hand_rolling.sh`.
- [x] Added latest-tool install/version reporting in CI for `greentic-pack`, `greentic-flow`, `greentic-component`.
- [x] Added CI tool version logging.
- [x] Wired no-hand-rolling checks into local CI mirror script: `ci/local_check.sh`.
- [x] Remove banned fixture/source `pack.lock.json` artifacts.
- [x] Align CI policy with current toolchain: keep flow sidecars (`*.resolve.json`, `*.resolve.summary.json`) as validated regeneration artifacts.
- [x] Implement deterministic temp-workspace flow auto-binding for missing sidecar entries.
- [x] Remove legacy pack build skip path.
- [x] Ensure `pack.lock.cbor` exists for each source pack after regeneration (validated in temp regen pipeline).
- [~] Add/confirm `greentic-component build` + `doctor` execution path for real component manifests.
  - blocker: current checked-in stub component manifests/wasm do not expose required 0.6 interfaces (`component-descriptor`, `component-runtime`, `component-qa`, `component-i18n`), so `greentic-component doctor` fails.
- [x] Rename/provider split work: Twilio + SendGrid dedicated component/pack names.
- [x] Introduce canonical `/v1/{domain}/ingress/...` routes in pack metadata for HTTP ingress providers (`webhook`, `sms.twilio`, `email.sendgrid`) in both source packs and fixture packs.
- [x] Complete Events V1 envelope normalization across all providers.
  - core envelope fields (`event_id`, `event_type`, `occurred_at`, `source`, `scope`, `payload`) are emitted in webhook/timer/sms/email (+ twilio/sendgrid variants).
  - optional `http` and `raw` capture is implemented on HTTP-origin providers (`webhook`, `sms`, `sms-twilio`, `email-sendgrid`) and retained as optional fields.
- [x] Complete lifecycle flow set for provider packs (`setup_default`, `remove`, plus `setup_custom`/`update` where meaningful) in:
  - `packs/events-webhook`
  - `packs/events-timer`
  - `packs/events-sms-twilio`
  - `packs/events-email-sendgrid`
- [x] PR-EVP-02 done: webhook component supports canonical op `ingest_http` (with `publish` alias), emits Events V1 envelope, and pack declares canonical ingress route/handler metadata.
- [x] PR-EVP-03 done: timer component supports canonical op `timer_tick` (with `publish` alias), emits Events V1 `timer.tick` envelope, and timer pack validates/builds.
- [x] PR-EVP-04 done: SMS/Twilio component supports canonical `ingest_http` (with `publish` alias), includes `send_sms` stub op, emits Events V1 `sms.received`, and dedicated twilio pack/fixture path is in place.
- [x] PR-EVP-05 done: Email/SendGrid component supports canonical `ingest_http` (with `publish` alias), emits Events V1 `email.received` (with optional `http`/`raw`), and dedicated sendgrid pack/fixture path is in place.

## Phase Backlog

### Phase 1: Bootstrap Tooling (PR-EVP-01)
Tasks:
- [x] Create/verify repo layout: `components/`, `packs/`, `ci/`.
- [x] Add `CONTRIBUTING.md` rule: generated assets must come from CLIs.
- [x] Standardize all new lifecycle references in PR specs to `default/setup/update/remove`.

Definition of done:
- Project contains baseline directories and contribution policy.
- No legacy lifecycle naming in new specs/assets.

### Phase 2: CI Guardrails (PR-EVP-06)
Tasks:
- [x] Add/update workflow under `.github/workflows/` to install latest tool versions:
  - `greentic-pack`
  - `greentic-flow`
  - `greentic-component`
- [x] Add tool version reporting to CI logs.
- [x] Add/update checks script (`ci/no_hand_rolling.sh`) to run:
  - `greentic-component build`
  - `greentic-component doctor`
  - `greentic-flow doctor`
  - `greentic-pack update`
  - `greentic-pack resolve`
  - `greentic-pack build`
  - `greentic-pack doctor`
- [x] Add banlist check failing on:
  - `pack.manifest.json`
  - `pack.lock.json`
- [x] Validate flow sidecars via regen checks in temp workspace (instead of banning them)
- [x] Add clean-regeneration check: `git diff --exit-code`.
- [x] Ensure workflow runs on `pull_request` and `push` to `main`.
- [~] Replace placeholder component build/doctor invocation with executable provider component-manifest path once manifests are added.
  - blocker: provider component manifests are not yet present; existing templating stub manifest fails doctor due missing 0.6 exports.
- [x] Replace placeholder pack components (source/sink/templating stubs) with self-describing runtime-valid components so strict `resolve/build/doctor` can be fully hard-fail.

Definition of done:
- CI fails for banned artifacts, regen drift, and version drift.

### Phase 3: Webhook Provider (PR-EVP-02)
Tasks:
- Create component `components/events-provider-webhook/` with 0.6 self-describing surfaces:
  - `describe`, `runtime.invoke`, `qa_spec`, `apply_answers`, `component-i18n`.
- Implement op `ingest_http` returning HTTP response + emitted events.
- Emit Events V1 envelope.
- Create pack `packs/events-webhook/`:
  - provider extension wiring
  - HTTP handler metadata mapping to `op_id="ingest_http"`
  - canonical route pattern using `domain=events`, `provider=webhook`.
- Generate lifecycle flows:
  - required: `setup_default.ygtc`, `remove.ygtc`
  - optional/meaningful: `setup_custom.ygtc`, `update.ygtc`.
- Add component tests for sample webhook payload ingestion.

Definition of done:
- Pack builds to `.gtpack`, includes `pack.lock.cbor`, and validates offline.

### Phase 4: Timer Provider (PR-EVP-03)
Tasks:
- Create component `components/events-provider-timer/` with `timer_tick`.
- Add QA config for schedules, enable/disable, event type, payload template.
- Emit Events V1 envelope with `event_type=timer.tick` (or provider-specific equivalent).
- Create pack `packs/events-timer/` with timer handler declarations.
- Generate lifecycle flows (`setup_default`, `setup_custom`, `update`, `remove`).
- Add tick simulation tests.

Definition of done:
- Scheduler-facing timer invocations produce normalized events and pack validates offline.

### Phase 5: Twilio SMS Provider (PR-EVP-04)
Tasks:
- Create component `components/events-provider-sms-twilio/` with:
  - `ingest_http` for inbound Twilio payload parsing
  - `send_sms` op (working or explicit not-enabled stub).
- QA + i18n:
  - secrets: `account_sid`, `auth_token`
  - config: `from_number`, optional `messaging_service_sid`.
- Emit inbound `sms.received` as Events V1 envelope.
- Create pack `packs/events-sms-twilio/` with handler metadata to `ingest_http` and canonical route pattern (`domain=events`, `provider=sms.twilio`).
- Generate lifecycle flows with `update` naming.
- Add tests:
  - inbound form payload fixture test
  - outbound `send_sms` behavior test.

Definition of done:
- Inbound SMS ingestion works and outbound op is present/tested; pack validates offline.

### Phase 6: SendGrid Email Provider (PR-EVP-05)
Tasks:
- Create component `components/events-provider-email-sendgrid/` with `ingest_http`.
- Parse SendGrid Inbound Parse payloads (multipart form-data, envelope fields, raw MIME where available).
- Emit `email.received` as Events V1 envelope, including optional `http` and `raw` when available.
- Add QA + i18n assets.
- Create pack `packs/events-email-sendgrid/` with handler metadata to `ingest_http` and canonical route pattern (`domain=events`, `provider=email.sendgrid`).
- Generate lifecycle flows with `update` naming.
- Add component tests with fixture payloads.

Definition of done:
- Inbound email webhook payloads normalize to routable envelopes and pack validates offline.

## Cross-Cutting Implementation Rules
- All generated artifacts must come from CLI tooling only.
- Keep pack artifacts CBOR-first (`pack.manifest.cbor`, `pack.lock.cbor`).
- Do not check in banned JSON-generated artifacts.
- Prefer deterministic fixtures for component tests.
- Keep provider-specific parsing in components; keep ingress routing conventions operator-owned and consistent.

## Exit Criteria (Program-Level)
- All six PR scopes implemented according to locked decisions.
- CI green on PR and main with deterministic regen and latest-tool checks.
- All packs build and pass doctor/validation with no banned artifacts.
