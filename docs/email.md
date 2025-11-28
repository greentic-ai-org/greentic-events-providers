# Email provider

Purpose: inbound/outbound email events through Microsoft Graph or Gmail/Google Workspace.

- Component IDs: `events-email-source@1.0.0`, `events-email-sink@1.0.0`.
- Topics: inbound `email.in.msgraph.<folder>` / `email.in.gmail.<label>`; outbound `email.out.msgraph` / `email.out.gmail`.
- Inbound: host polls/receives mail and passes normalized email JSON; component emits `EventEnvelope`.
- Outbound: component builds provider-specific send payloads; host executes HTTP/SMTP.
- Secrets/tokens: `greentic-secrets` for credentials, `greentic-oauth-sdk` for Graph/Gmail tokens.
- Packs: `packs/events/email.yaml`.
- Flows: `flows/events/email/in_default.ygtc`, `flows/events/email/out_default.ygtc`.
