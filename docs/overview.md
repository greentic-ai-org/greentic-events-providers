# Greentic Events Providers

This repository ships reusable WASM event providers plus packs so `greentic-events` and `greentic-deployer` can discover and host them.

- Providers implement `greentic:events@1.0.0` via `greentic-interfaces-guest`.
- Hosts/deployer own HTTP servers, schedulers, and OAuth/token plumbing; components stay pure and deterministic.
- Secrets are provisioned via `greentic-secrets` using pack `secret_requirements`; no env-var fallbacks.
- Provisioning uses `greentic-provision` with deterministic dry-run plans and pack fixtures.
- Requirements fixtures are validated against the setup WAT output because `greentic-provision` does not yet surface requirements output via CLI.
- Secrets events use metadata-only payloads on `greentic.secrets.put|delete|rotate.*|missing.detected` topics.
- OAuth flows (MS Graph/Gmail) go through `greentic-oauth-sdk`.

Families included:
- **webhook**: generic HTTP in/out.
- **email**: inbound/outbound via MS Graph and Gmail.
- **sms**: inbound/outbound via Twilio.
- **timer**: cron/interval sources.
