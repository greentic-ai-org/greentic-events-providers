# Webhook provider

Purpose: generic HTTP ingestion and outgoing POST delivery.

- Component IDs: `events-webhook-source@1.0.0`, `events-webhook-sink@1.0.0`.
- Config: `HttpEndpointConfig` with routes, optional signing secrets, topic prefixes.
- Behaviour: host feeds HTTP request data; component maps to `EventEnvelope` with topic `webhook.<route>.<event_type>`. `handle_request` returns both the main event and any `secret_events` to forward.
- Secrets: signing keys declared as `secret_requirements` (`WEBHOOK_SIGNING_SECRET`), resolved via `greentic:secrets-store@1.0.0`; no env-based fallback.
- Secrets events: metadata-only payloads on `greentic.secrets.*` topics describe put/delete/rotate and `greentic.secrets.missing.detected` when validation keys are absent.
- Packs: `packs/events/webhook.yaml`.
- Flows: `packs/events-webhook/flows/in_default.ygtc`, `packs/events-webhook/flows/in_custom_template.ygtc`.

## Setup

- Entry flow: `setup` (collect → validate → apply → summary).
- Required inputs: `webhook.path`, `webhook.topic_prefix`, `public_base_url`.
- Required secrets: `WEBHOOK_SIGNING_SECRET` (optional signing key).
- Dry-run plan: emits webhook ops for the configured callback URL.
