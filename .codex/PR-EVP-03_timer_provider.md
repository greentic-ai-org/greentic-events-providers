# PR-EVP-03: Add events-provider-timer (0.6.0, QA+i18n, timer handlers)

## Repo
`greentic-events-providers`

## Goal
Add a timer provider that emits scheduled events without HTTP.
- Provider declares timer handlers (interval/cron)
- Operator scheduler calls `timer_tick` op
- QA modes follow canonical lifecycle names: `default/setup/update/remove`

## Component
`components/events-provider-timer/`
- Ops:
  - `timer_tick` (input: handler_id, now, tenant/team) -> emitted events
- Emitted events must follow canonical Events V1 envelope:
  - Required: `event_id`, `event_type` (e.g. `timer.tick`), `occurred_at`, `source{domain,provider,handler_id}`, `scope{tenant,team?,correlation_id?}`, `payload`
- QA config:
  - enable/disable timers
  - schedules (interval seconds / cron expression)
  - event_type and payload template

## Pack
`packs/events-timer/`
- Extensions declare timer handlers:
  - handler_id + schedule + op_id
- Lifecycle flows (generated via `greentic-flow`):
  - `setup_default.ygtc` and `remove.ygtc` required
  - `setup_custom.ygtc` and `update.ygtc` when meaningful (timer provider: yes)

## Testing
- `greentic-component test` simulating tick
- `greentic-pack doctor --validate --offline`

## Acceptance criteria
- Operator demo scheduler can invoke the provider tick and route events
