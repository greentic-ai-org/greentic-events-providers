# PR-EVP-04: Add events-provider-sms-twilio (inbound webhook + outbound send op)

## Repo
`greentic-events-providers`

## Goal
Provide Twilio SMS integration:
- Inbound SMS: HTTP webhook handler -> emitted event
- Optional outbound: `send_sms` op (called by application flows)
  - MVP requirement: inbound is mandatory
  - `send_sms` must exist at least as an op stub, even if it initially returns not-enabled/not-configured

## Component
`components/events-provider-sms-twilio/`
- Ops:
  - `ingest_http` parses Twilio payload -> events + HTTP response
  - `send_sms` sends an SMS -> delivery result
- `ingest_http` is the canonical externally-invoked op id for HTTP handlers.
- Emitted inbound events must follow canonical Events V1 envelope:
  - Required: `event_id`, `event_type` (e.g. `sms.received`), `occurred_at`, `source{domain,provider,handler_id}`, `scope{tenant,team?,correlation_id?}`, `payload`
  - Recommended HTTP metadata: `http{method,path,query,headers,remote_addr}`
  - Recommended raw body preservation: `raw`
- QA:
  - secrets: account_sid, auth_token
  - config: from_number, optional messaging_service_sid
- i18n keys for questions

## Pack
`packs/events-sms-twilio/`
- HTTP handler metadata + lifecycle flows generated via tooling
- Use canonical ingress route pattern:
  - `/v1/{domain}/ingress/{provider}/{tenant}/{team?}/{handler?}`
  - for this pack: `domain=events`, `provider=sms.twilio`
- Lifecycle naming uses `update.ygtc` (not `upgrade.ygtc`)

## Testing
- component test: parse Twilio form payload
- offline doctor/validate

## Acceptance criteria
- inbound SMS produces normalized events
- outbound send op is present and test-covered (working or explicit not-enabled stub behavior)
