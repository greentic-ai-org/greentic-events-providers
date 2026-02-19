# Webhook events provider

Provider-core implementation for inbound webhook ingestion.

- **Provider type:** `events.webhook`
- **Operation:** `ingest_http` (canonical ingress op; `publish` retained as legacy alias in component code)
- **Request body:** `{ "event": <input event JSON> }`
- **Headers:** default `content-type: application/json`; optional custom headers and bearer token via `auth`.
- **Receipts:** deterministic `receipt_id` derived from the event payload.
