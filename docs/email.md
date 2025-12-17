# Email provider

Purpose: inbound/outbound email events through Microsoft Graph or Gmail/Google Workspace.

- Component IDs: `events-email-source@1.0.0`, `events-email-sink@1.0.0`.
- Topics: inbound `email.in.msgraph.<folder>` / `email.in.gmail.<label>`; outbound `email.out.msgraph` / `email.out.gmail`.
- Inbound: host polls/receives mail and passes normalized email JSON; component emits `EventEnvelope`.
- Outbound: component builds provider-specific send payloads; host executes HTTP/SMTP. `EmailSendRequest` includes `secret_events` for hosts to forward before/alongside the send.
- Secrets/tokens: secrets provisioned via `greentic-secrets` using requirements in the pack (`MSGRAPH_CLIENT_SECRET`, `GMAIL_CLIENT_SECRET`, `GMAIL_REFRESH_TOKEN`); components read via `greentic:secrets-store@1.0.0` (no env fallbacks).
- Secrets events: emit metadata-only payloads on `greentic.secrets.put|delete|rotate.*|missing.detected` when secrets are resolved or missing (no values).
- Packs: `packs/events/email.yaml`.
- Flows: `flows/events/email/in_default.ygtc`, `flows/events/email/out_default.ygtc`.
