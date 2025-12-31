# Dummy events provider

Deterministic provider used by CI/integration to exercise the provider-core runtime without reaching external services.

- **Provider type:** `events.dummy`
- **Operations:** `publish` (and `echo` for simple round-trip tests)
- **Runtime:** `greentic:provider/schema-core@1.0.0`
- **State key:** `events/dummy/last_published.json` stores the most recent publish payload (metadata only).
