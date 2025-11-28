# Webhook provider

Purpose: generic HTTP ingestion and outgoing POST delivery.

- Component IDs: `events-webhook-source@1.0.0`, `events-webhook-sink@1.0.0`.
- Config: `HttpEndpointConfig` with routes, optional signing secrets, topic prefixes.
- Behaviour: host feeds HTTP request data; component maps to `EventEnvelope` with topic `webhook.<route>.<event_type>`.
- Secrets: signing keys resolved by host via `greentic-secrets`.
- Packs: `packs/events/webhook.yaml`.
- Flows: `flows/events/webhook/in_default.ygtc`, `flows/events/webhook/in_custom_template.ygtc`.
