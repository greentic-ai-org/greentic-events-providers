# PR-EVP-05: Add events-provider-email-sendgrid (inbound email webhooks)

## Repo
`greentic-events-providers`

## Goal
Provide inbound email event ingestion via webhook provider.
First target vendor: SendGrid Inbound Parse.

## Component
`components/events-provider-email-sendgrid/`
- Ops:
  - `ingest_http` -> parse SendGrid payload -> emit `email.received` events
- Notes:
  - payload model includes multipart form-data and raw email (MIME) preservation where available
  - emitted events follow canonical Events V1 envelope:
    - Required: `event_id`, `event_type`, `occurred_at`, `source{domain,provider,handler_id}`, `scope{tenant,team?,correlation_id?}`, `payload`
    - Recommended HTTP metadata: `http{method,path,query,headers,remote_addr}`
    - Recommended raw body preservation: `raw`
- QA + i18n

## Pack
`packs/events-email-sendgrid/`
- HTTP handler declaration + lifecycle flows generated via tooling
- HTTP handlers point to op_id=`ingest_http`
- Use canonical ingress route pattern:
  - `/v1/{domain}/ingress/{provider}/{tenant}/{team?}/{handler?}`
  - for this pack: `domain=events`, `provider=email.sendgrid`
- Lifecycle naming uses `update.ygtc` (not `upgrade.ygtc`)

## Testing
- component test with sample inbound payload fixtures
- offline doctor/validate

## Acceptance criteria
- inbound email webhook yields normalized event envelopes suitable for routing to app flows
