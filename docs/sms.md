# SMS provider

Purpose: Twilio inbound/outbound SMS.

- Component IDs: `events-sms-source@1.0.0`, `events-sms-sink@1.0.0`.
- Topics: inbound `sms.in.twilio.<alias>`; outbound `sms.out.twilio`.
- Inbound: host receives Twilio webhook, passes normalized payload; component emits `EventEnvelope`.
- Outbound: component builds Twilio REST request payload; host performs HTTP. `TwilioSendRequest` carries `secret_events` for hosts to forward on the bus.
- Secrets: Twilio creds declared as `secret_requirements` in the pack (`TWILIO_AUTH_TOKEN`), resolved via `greentic:secrets-store@1.0.0` (no env fallback).
- Secrets events: metadata-only payloads emitted on `greentic.secrets.put` for resolved tokens and `greentic.secrets.missing.detected` when the token is absent.
- Packs: `packs/events/sms.yaml`.
- Flows: `packs/events-sms/flows/in_default.ygtc`, `packs/events-sms/flows/out_default.ygtc`.

## Setup

- Entry flow: `setup` (collect → validate → apply → summary).
- Required inputs: `twilio.account_sid`, `twilio.from_number`, `twilio.webhook_path`, `public_base_url`.
- Required secrets: `TWILIO_AUTH_TOKEN`.
- Dry-run plan: emits webhook ops for the inbound Twilio callback URL.
