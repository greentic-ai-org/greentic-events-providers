# Email SendGrid events provider

Provider-core adapter that queues email send requests via state-store for a downstream messaging provider.

- **Provider type:** `events.email.sendgrid`
- **Operation:** `ingest_http` (canonical ingress op; `publish` retained as legacy alias in component code)
- **State key:** `events/email/sendgrid/queued/<receipt_id>.json` by default
- **Receipt:** deterministic UUID v5 derived from the event payload.
