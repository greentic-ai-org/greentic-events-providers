# Greentic Events Providers

This repository ships reusable WASM event providers plus packs so `greentic-events` and `greentic-deployer` can discover and host them.

- Providers implement `greentic:events@1.0.0` via `greentic-interfaces-guest`.
- Hosts/deployer own HTTP servers, schedulers, and OAuth/token plumbing; components stay pure and deterministic.
- Secrets follow `events/<provider>/<tenant>/...` conventions through `greentic-secrets`.
- OAuth flows (MS Graph/Gmail) go through `greentic-oauth-sdk`.

Families included:
- **webhook**: generic HTTP in/out.
- **email**: inbound/outbound via MS Graph and Gmail.
- **sms**: inbound/outbound via Twilio.
- **timer**: cron/interval sources.
