# SMS Twilio events provider

Provider-core adapter that queues SMS send requests via state-store for a downstream messaging provider.

- **Provider type:** `events.sms.twilio`
- **Operations:** `ingest_http` (canonical inbound op), `send_sms` (MVP stub), `publish` (legacy alias)
- **State key:** `events/sms/twilio/queued/<receipt_id>.json` by default
- **Receipt:** deterministic UUID v5 derived from the event payload.
