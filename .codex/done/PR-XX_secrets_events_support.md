# PR: Secrets workflow support for events providers (requirements + topics + fixtures)

## Goal
Make greentic-events-providers fully compatible with the unified secrets workflow:
- packs declare `secret_requirements` so `greentic-secrets init --pack ...` works
- provider components access secrets via `greentic:secrets-store@1.0.0` only
- secrets-related events are emitted using standardized topics with metadata-only payloads (no secret values)
- deterministic fixtures and tests cover requirements + event emission

## Non-goals
- Do not include secret values in any event payload, logs, fixtures, or errors.
- Do not add env-var fallbacks in production code paths.

---

## 1) Standardize secrets events topics (use these exact strings)
Providers must emit/handle these topics as appropriate:

- `greentic.secrets.put`
- `greentic.secrets.delete`
- `greentic.secrets.rotate.requested`
- `greentic.secrets.rotate.completed`
- `greentic.secrets.missing.detected`

Payload rule: metadata only; never secret bytes, never base64 values.

### Payload guidance (JSON examples)
Use a minimal common payload shape:

For put/delete:
```json
{
  "schema_version": "v1",
  "key": "SLACK_BOT_TOKEN",
  "scope": "tenant",
  "tenant_ctx": { "env": "...", "tenant": "...", "team": null, "user": null },
  "result": "success",
  "timestamp_utc": "..."
}


For rotate.requested / rotate.completed:

{
  "schema_version": "v1",
  "key": "MS_GRAPH_CLIENT_SECRET",
  "scope": "tenant",
  "rotation_id": "uuid-or-opaque",
  "result": "requested|success|failed",
  "error": "string-if-failed",
  "timestamp_utc": "..."
}


For missing.detected:

{
  "schema_version": "v1",
  "key": "TELEGRAM_BOT_TOKEN",
  "scope": "tenant",
  "detected_by": "events-provider/<name>",
  "context": "what operation needed it (no values)",
  "timestamp_utc": "..."
}
```

2) Packs/components must declare structured secret_requirements

For every provider pack produced here:

ensure each component manifest declares secret_requirements: Vec<SecretRequirement>

ensure pack build aggregates them into .gtpack metadata (PackMetadata.secret_requirements)

No hand-edited .gtpack bytes; requirements come from component manifests.

3) Runtime secret access: secrets-store only

In all provider components:

remove env var reads and URI-based secret paths

call greentic:secrets-store@1.0.0 for secret bytes

scope lookups using runtime TenantCtx/session identity (no CLI ctx in production)

On missing secret:

return a structured error (if your framework supports) AND

emit greentic.secrets.missing.detected event with the metadata payload above

4) Components/packs in this repo

Identify all “events provider” components and ensure each aligns:

ingress: incoming events → normalize → publish to event bus

egress: outgoing events → call external APIs (if any)

secret usage exists only for credentials needed for external APIs/webhooks

For each provider, declare its keys:

e.g. github: GITHUB_WEBHOOK_SECRET / app keys

msgraph: MS_GRAPH_CLIENT_SECRET etc.

email provider: API keys
(Use consistent key naming; do not embed tenant ids in key strings—TenantCtx scopes it.)

5) Fixtures + tests (deterministic)

Add a minimal fixture pack:

fixtures/packs/secrets_events_smoke/

includes a tiny component that:

attempts secrets-store.get("TEST_API_KEY")

on missing, emits greentic.secrets.missing.detected

on present, emits greentic.secrets.put (metadata-only)

Tests must assert:

pack metadata includes secret_requirements with TEST_API_KEY

emitted events use correct topic strings

payload JSON includes key/scope and does NOT contain values

If the test harness can’t execute wasm components, add unit tests around event payload builders and pack metadata parsers.

6) Docs

Update README/docs for this repo:

“Use greentic-secrets init --pack …”

“No env-based secrets”

document the secrets event topics and payload examples

explicitly warn: never include secret values in events

Acceptance criteria

All packs/components declare secret_requirements and aggregate into .gtpack

All secret reads go through secrets-store

Missing secret emits greentic.secrets.missing.detected with metadata-only payload

Tests cover requirements + topic/payload correctness

Docs updated and cargo test passes
