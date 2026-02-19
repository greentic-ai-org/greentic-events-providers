# PR-11D.md (greentic-events-providers)
# Title: Webhook events provider-core pack + schema + publish op (real provider)

## Goal
Implement webhook events as a provider-core pack.

Interpretation:
- `publish` posts the event to a configured HTTP endpoint (webhook target).
- (Future) `subscribe` could register callbacks, but not required now.

## Deliverables
1) Schema:
- `schemas/events/webhook/config.schema.json`:
  - target_url (required)
  - method (default POST)
  - headers (optional object)
  - auth (optional: bearer token ref; x-secret)
  - timeout_ms (optional)
2) Component:
- `components/events-provider-webhook/`
- provider_type: `events.webhook`
- ops: `publish`
Behavior:
- invoke("publish"):
  - builds HTTP request from config + publish input
  - posts JSON payload containing publish input OR just `event` (document choice)
  - returns receipt_id + status (published/queued)
3) Pack:
- `packs/events-webhook.gtpack/`
- provider extension inline

4) Tests
- Use mocked HTTP import if available; otherwise:
  - contract/unit tests for request construction
  - integration tests rely on dummy provider for CI stability

## Acceptance criteria
- Pack self-describing.
- publish op implemented using HTTP capability (no live network in CI).
