# PR-EVP-02: Add events-provider-webhook (0.6.0 self-describing, QA+i18n, HTTP ingress handler)

## Repo
`greentic-events-providers`

## Goal
Introduce the first real events provider:
- Generic webhook receiver provider pack
- 0.6.0 self-describing component
- QA modes: default/setup/update/remove
- i18n: keys + bundles
- HTTP ingress handler declaration using canonical operator route:
  - `/v1/{domain}/ingress/{provider}/{tenant}/{team?}/{handler?}`
  - for this pack: `domain=events`, `provider=webhook`

## Component
`components/events-provider-webhook/`
- Implements `describe`, `runtime.invoke`, `qa_spec`, `apply_answers`, `component-i18n`
- Ops:
  - `ingest_http` (canonical externally-invoked op id) returning:
    - HTTP response (status/headers/body)
    - emitted events list
- Emitted events must follow canonical Events V1 envelope:
  - Required: `event_id`, `event_type`, `occurred_at`, `source{domain,provider,handler_id}`, `scope{tenant,team?,correlation_id?}`, `payload`
  - Recommended when HTTP-originated: `http{method,path,query,headers,remote_addr}`
  - Recommended when preserving original body: `raw`

## Pack
`packs/events-webhook/`
- Provider extension points to component
- Extensions declare:
  - http handler(s): method/path/handler_id -> op_id=`ingest_http`
- Lifecycle flows (generated via `greentic-flow`):
  - `setup_default.ygtc` (minimal questions; defaults)
  - `setup_custom.ygtc` only if there are meaningful non-default choices
  - `update.ygtc` only if iterative config changes are meaningful
  - `remove.ygtc`

## Testing
- `greentic-component test` for `ingest_http` with sample payload
- `greentic-pack doctor --validate --offline`

## Acceptance criteria
- Pack builds to `.gtpack` with `pack.lock.cbor`
- Operator demo can dispatch `/events/ingress/webhook/...` to this provider once operator PRs land
