# Email events provider

Provider-core adapter that queues email send requests via state-store for a downstream messaging provider.

- **Provider type:** `events.email`
- **Operation:** `publish` (queues send request, `status=queued`)
- **State key:** `events/email/queued/<receipt_id>.json` by default
- **Receipt:** deterministic UUID v5 derived from the event payload.
