# SMS events provider

Provider-core adapter that queues SMS send requests via state-store for a downstream messaging provider.

- **Provider type:** `events.sms`
- **Operation:** `publish` (queues send request, `status=queued`)
- **State key:** `events/sms/queued/<receipt_id>.json` by default
- **Receipt:** deterministic UUID v5 derived from the event payload.
