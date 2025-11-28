# SMS provider

Purpose: Twilio inbound/outbound SMS.

- Component IDs: `events-sms-source@1.0.0`, `events-sms-sink@1.0.0`.
- Topics: inbound `sms.in.twilio.<alias>`; outbound `sms.out.twilio`.
- Inbound: host receives Twilio webhook, passes normalized payload; component emits `EventEnvelope`.
- Outbound: component builds Twilio REST request payload; host performs HTTP.
- Secrets: Twilio creds under `events/sms/<tenant>/twilio` via `greentic-secrets`.
- Packs: `packs/events/sms.yaml`.
- Flows: `flows/events/sms/in_default.ygtc`, `flows/events/sms/out_default.ygtc`.
