# PR-11E.md (greentic-events-providers)
# Title: Timer events provider-core pack + schema + publish op (scheduled emit)

## Goal
Implement timer as an events provider-core pack.

Interpretation:
- `publish` schedules an event for later (or emits immediately if options specify).
- For v1, simplest: `publish` records a schedule request into state-store so the runner (or an external scheduler) can pick it up.
This avoids requiring a new runtime service in this PR.

## Deliverables
1) Schema:
- `schemas/events/timer/config.schema.json`:
  - timezone (default UTC)
  - default_delay_seconds (optional)
  - persistence_key_prefix (optional)
2) Component:
- `components/events-provider-timer/`
- provider_type: `events.timer`
- ops: `publish`
Behavior:
- invoke("publish"):
  - writes schedule request to state-store:
    - `events/timer/scheduled/<receipt_id>.json`
  - returns receipt_id + status="queued"
3) Pack:
- `packs/events-timer.gtpack/`

4) Tests
- deterministic unit + integration test:
  - publish → assert state-store key exists

## Acceptance criteria
- Timer provider pack exists and is testable deterministically.
- No external scheduler required yet (future enhancement can process scheduled keys).
