# Timer events provider

Provider-core implementation that queues timer events by writing schedule requests into state-store.

- **Provider type:** `events.timer`
- **Operation:** `timer_tick` (canonical timer op; `publish` retained as legacy alias in component code)
- **State key:** `events/timer/scheduled/<receipt_id>.json` by default
- **Receipt:** deterministic UUID v5 derived from the event payload.
