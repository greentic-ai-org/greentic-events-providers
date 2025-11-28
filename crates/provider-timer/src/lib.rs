use provider_core::{ProviderError, SchedulerConfig, new_event};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimerSource {
    config: SchedulerConfig,
}

impl TimerSource {
    pub fn new(config: SchedulerConfig) -> Self {
        Self { config }
    }

    pub fn fire(
        &self,
        tenant: greentic_types::TenantCtx,
        schedule_name: &str,
    ) -> Result<greentic_types::EventEnvelope, ProviderError> {
        let schedule = self
            .config
            .schedules
            .iter()
            .find(|s| s.name == schedule_name)
            .ok_or_else(|| ProviderError::Config(format!("unknown schedule {}", schedule_name)))?;

        let mut metadata = BTreeMap::new();
        metadata.insert("schedule_name".into(), schedule.name.clone());
        metadata.insert("cron".into(), schedule.cron.clone());

        Ok(new_event(
            schedule.topic.clone(),
            "com.greentic.timer.generic.v1",
            "timer-provider",
            tenant,
            Some(schedule.name.clone()),
            None,
            schedule.payload.clone(),
            metadata,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use provider_core::{Schedule, SchedulerConfig};
    use serde_json::json;

    fn tenant() -> greentic_types::TenantCtx {
        use greentic_types::{EnvId, TenantCtx, TenantId};

        let env = EnvId::try_from("dev").unwrap();
        let tenant = TenantId::try_from("acme").unwrap();
        TenantCtx::new(env, tenant)
    }

    #[test]
    fn fires_schedule_into_event() {
        let source = TimerSource::new(SchedulerConfig {
            schedules: vec![Schedule {
                name: "daily".into(),
                cron: "0 0 * * *".into(),
                topic: "timer.daily.summary".into(),
                payload: json!({"kind": "daily"}),
            }],
        });

        let event = source.fire(tenant(), "daily").expect("event");
        assert_eq!(event.topic, "timer.daily.summary");
        assert_eq!(event.subject, Some("daily".into()));
        assert_eq!(event.payload, json!({"kind": "daily"}));
        assert_eq!(event.metadata.get("cron"), Some(&"0 0 * * *".into()));
    }
}
