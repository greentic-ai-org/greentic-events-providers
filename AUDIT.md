# greentic-events-providers Audit (Greentic 0.6.0 + Pack Validation)

Date: 2026-02-18
Audited repo: `/Users/maarten/Documents/GitHub/agentic/greentic-events-providers`

## Scope and method
- Audited source packs in `packs/*` (excluding `packs/components`) and provider components in `components/events-provider-*`.
- Ran latest local tools in this environment:
  - `greentic-pack 0.4.91`
  - `greentic-flow 0.4.41`
  - `greentic-component 0.4.53`
- Ran `cargo test` on provider components, `greentic-component build/doctor`, and `bash ci/no_hand_rolling.sh`.

## 1) Repo inventory

### Packs and classification

| Pack | Path | Kind | Classification |
|---|---|---|---|
| greentic.events.provider.dummy | `packs/events-dummy` | `application` | provider-extension pack |
| greentic.events.email | `packs/events-email` | `application` | application-pack |
| greentic.events.email.sendgrid | `packs/events-email-sendgrid` | `application` | application-pack + provider-extension metadata |
| greentic.events.sms | `packs/events-sms` | `application` | application-pack |
| greentic.events.sms.twilio | `packs/events-sms-twilio` | `application` | application-pack + provider-extension metadata |
| greentic.events.timer | `packs/events-timer` | `application` | application-pack |
| greentic.events.webhook | `packs/events-webhook` | `application` | application-pack |

### Flows/components/assets and legacy artifacts
- Lifecycle flows (`setup_default`, `setup_custom`, `update`, `remove`) are present for webhook/timer/twilio/sendgrid packs.
- All packs resolve/build with CBOR lock generation in CI temp workspace (`pack.lock.cbor`).
- Legacy JSON-first artifacts still exist in source flows as sidecars (`*.resolve.json`, `*.resolve.summary.json`) by current toolchain design.
- Provider component manifests exist for:
  - `components/events-provider-webhook/component.manifest.json`
  - `components/events-provider-timer/component.manifest.json`
  - `components/events-provider-sms-twilio/component.manifest.json`
  - `components/events-provider-email-sendgrid/component.manifest.json`

## 2) Pack validation audit

### Where validator/checks are invoked
- `ci/no_hand_rolling.sh`
- `ci/local_check.sh`
- `scripts/build_packs.sh`

### Current validation status
- `bash ci/no_hand_rolling.sh` passes.
- `greentic-pack doctor --validate` on built pack artifacts passes in script runs.
- Twilio and SendGrid provider extension metadata now validate in packs.

### Remaining validation/tool failures
1. `greentic-component doctor` linker failures for provider components importing host instances:
   - `greentic:state/state-store@1.0.0`
   - `greentic:http/http-client@1.0.0` (webhook)
2. `greentic-component doctor` still reports templating stub mismatch (missing 0.6 interfaces/world mismatch) for:
   - `packs/components/templating.handlebars/component.manifest.json`

These are currently treated as known tool/runtime mismatches in `ci/no_hand_rolling.sh` allowlist.

## 3) 0.6.0 self-describing component audit

### Migrated providers
- Webhook, timer, Twilio SMS, SendGrid email components now expose manifest + component entrypoint scaffolding and use `world: greentic:component/component@0.6.0` in source manifests/describe payloads.
- Component tests pass for all four migrated providers.

### Outstanding 0.6 blockers
- `greentic-component build` still logs fallback world output (`root:component/root`) and expected `component@0.5.0` message, even after 0.6 source declaration.
- `greentic-component doctor` cannot fully instantiate provider components due linker import implementation gaps (see bug list below).

## 4) greentic-qa + greentic-i18n audit

### What is in place
- Lifecycle flows exist for provider packs where requested (`default/setup/update/remove`).
- Pack/provider extension metadata for Twilio + SendGrid now includes config schema/docs refs.

### What is still missing
- Strict component-level `component-qa` + `component-i18n` doctor compliance remains blocked by tool/runtime mismatch on templating and provider doctor paths.
- i18n for flow prompt content remains mostly template-text based, not fully key-driven across all flows.

## 5) Ingress/egress integration points

### Inbound ingress
- Canonical ingress op id is `ingest_http`.
- Canonical routes are in pack extensions:
  - Twilio: `/v1/events/ingress/sms.twilio/{tenant}/{team}/{handler}`
  - SendGrid: `/v1/events/ingress/email.sendgrid/{tenant}/{team}/{handler}`
  - Webhook: `/v1/events/ingress/webhook/{tenant}/{team}/{handler}`

### Outbound path
- Twilio `send_sms` op exists (MVP stub behavior supported).
- Other provider components use normalized emitted event envelopes (`sms.received`, `email.received`, `timer.tick`, etc.)

## Summary table

| Pack | VALIDATION | 0.6.0 | QA | I18N | INGRESS NOTES |
|---|---|---|---|---|---|
| greentic.events.provider.dummy | PASS | FAIL | FAIL | FAIL | no HTTP ingress |
| greentic.events.email | PASS | FAIL | FAIL | FAIL | no direct ingress extension |
| greentic.events.email.sendgrid | PASS | FAIL | FAIL | FAIL | canonical `ingest_http` route present |
| greentic.events.sms | PASS | FAIL | FAIL | FAIL | no direct ingress extension |
| greentic.events.sms.twilio | PASS | FAIL | FAIL | FAIL | canonical `ingest_http` route present |
| greentic.events.timer | PASS | FAIL | FAIL | FAIL | timer-triggered, no HTTP ingress |
| greentic.events.webhook | PASS | FAIL | FAIL | FAIL | canonical `ingest_http` route present |

Interpretation:
- `VALIDATION PASS`: current pack/CI validation pipelines pass.
- `0.6.0/QA/I18N FAIL`: strict end-to-end 0.6 doctor/runtime expectations are not yet fully met due remaining tool/runtime mismatches and incomplete QA+i18n surface compliance.

## Prioritized fix list

### P0
1. Resolve `greentic-component doctor` linker bug for host imports (`state-store`, `http-client`).
2. Resolve templating component doctor incompatibility (`component-descriptor/component-qa/component-i18n/component-runtime` + world expectation mismatch).
3. Keep CI allowlist narrow and tied to explicit signatures until upstream/tool fix lands.

### P1
1. Complete strict QA/i18n component surfaces for provider packs and flow prompt keying.
2. Extend provider-extension coverage consistently across all relevant packs.

### P2
1. Reduce sidecar churn / binding noise by stabilizing flow sidecar generation strategy.
2. Add regression tests around provider-extension docs/config refs for all provider packs.

## Commands used (exact)

```bash
cargo test -p events-provider-webhook -p events-provider-timer -p events-provider-sms-twilio -p events-provider-email-sendgrid
bash ci/no_hand_rolling.sh
cd packs/components/templating.handlebars && greentic-component doctor component.manifest.json
cd components/events-provider-sms-twilio && greentic-component build --manifest component.manifest.json && greentic-component doctor component.manifest.json
cd components/events-provider-webhook && greentic-component build --manifest component.manifest.json && greentic-component doctor component.manifest.json
bash scripts/build_stub_components.sh
```

## Sample failure logs (remaining tool/runtime bugs)

```text
greentic-component: doctor failure: doctor: failed to load component: component imports instance `greentic:state/state-store@1.0.0`, but a matching implementation was not found in the linker
```

```text
greentic-component: doctor failure: doctor: failed to load component: component imports instance `greentic:http/http-client@1.0.0`, but a matching implementation was not found in the linker
```

```text
error[doctor.export.call_failed] component-descriptor.describe: component-descriptor.describe failed: missing export interface component-descriptor
error[doctor.world.mismatch] world: component world mismatch: component world mismatch (expected `greentic:component/component-v0-v6-v0@0.6.0`, found `root:component/root`)
```

## Patch status
- Applied safe changes for vendor pack/provider migration:
  - Twilio and SendGrid provider-extension blocks + schema/docs assets in packs.
  - Vendor provider wasm artifacts added to `packs/components`.
  - Provider component manifest/source updates for 0.6 declarations.
- No destructive changes applied.
