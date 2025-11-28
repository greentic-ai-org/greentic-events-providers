# Timer provider

Purpose: cron/interval-based event source.

- Component ID: `events-timer-source@1.0.0`.
- Config: `SchedulerConfig` listing schedules with names, cron expressions, topics, payloads.
- Behaviour: host/deployer handles actual scheduling and calls into the component with a schedule name; component emits `EventEnvelope`.
- Topics: `timer.<name>` (e.g., `timer.daily.summary`).
- Packs: `packs/events/timer.yaml`.
- Flows: `flows/events/timer/default.ygtc`.
