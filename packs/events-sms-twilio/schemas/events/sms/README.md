# events.sms.twilio

Twilio SMS provider extension for Greentic events ingestion.

## Operations

- `ingest_http`: receives inbound webhook payloads and emits normalized `sms.received` events.
- `send_sms`: outbound SMS operation (MVP stub may return not-enabled).

## Config

- `messaging_provider_id` (required): stable provider identifier.
- `from` (optional): default sender number for outbound messaging.
- `persistence_key_prefix` (optional): override state-store key prefix.

## Ingress

- Canonical route: `/v1/events/ingress/sms.twilio/{tenant}/{team}/{handler}`
- Canonical op: `ingest_http`
