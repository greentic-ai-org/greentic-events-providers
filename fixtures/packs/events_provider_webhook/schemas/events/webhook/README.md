# Webhook events provider

Provider-core implementation for publishing events to arbitrary HTTP endpoints.

- **Provider type:** `events.webhook`
- **Operation:** `publish` (POST/PUT/PATCH JSON)
- **Request body:** `{ "event": <input event JSON> }`
- **Headers:** default `content-type: application/json`; optional custom headers and bearer token via `auth`.
- **Receipts:** deterministic `receipt_id` derived from the event payload.
