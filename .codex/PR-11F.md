# PR-11F.md (greentic-events-providers)
# Title: SMS and Email events providers as thin adapters (provider-core packs)

## Goal
Implement `events.sms` and `events.email` as provider-core packs.
These can be thin adapters that call existing messaging providers via provider-core,
or directly send via external APIs depending on your architecture.

## Preferred v1 approach (least duplication)
- Implement as adapters:
  - `events.sms.publish` calls a messaging provider op "send" with an SMS-formatted payload
  - `events.email.publish` calls messaging provider op "send" with email payload
This keeps "transport send" logic in messaging providers.

## Deliverables
1) Schemas:
- `schemas/events/sms/config.schema.json`:
  - messaging_provider_id (required)  // points at messaging provider instance
  - from (optional)
- `schemas/events/email/config.schema.json`:
  - messaging_provider_id (required)
  - from (optional)
2) Components:
- `components/events-provider-sms/` provider_type `events.sms` op `publish`
- `components/events-provider-email/` provider_type `events.email` op `publish`
Behavior:
- invoke("publish"):
  - transforms event payload into a messaging send input JSON
  - calls messaging provider-core (requires a runner-side "provider invocation from provider" OR uses flow-based composition)
If direct provider-to-provider call is hard right now, then v1 alternative:
- simply store a "send request" into state-store for the runner/flow to execute.

3) Packs:
- `packs/events-sms.gtpack/`, `packs/events-email.gtpack/`

4) Tests
- Deterministic tests:
  - publish → writes a request record to state-store (assert exists)
  - rely on dummy messaging provider in integration tests to complete the send in a flow (if you wire it)

## Acceptance criteria
- SMS/Email event providers exist without duplicating messaging provider logic.
- Deterministic CI tests pass without external services.
