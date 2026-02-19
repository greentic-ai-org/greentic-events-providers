# events.email.sendgrid

SendGrid Inbound Parse provider extension for Greentic events ingestion.

## Operations

- `ingest_http`: receives inbound SendGrid webhook payloads and emits normalized `email.received` events.

## Config

- `messaging_provider_id` (required): stable provider identifier.
- `from` (optional): default sender used by outbound integrations.
- `persistence_key_prefix` (optional): override state-store key prefix.

## Ingress

- Canonical route: `/v1/events/ingress/email.sendgrid/{tenant}/{team}/{handler}`
- Canonical op: `ingest_http`
