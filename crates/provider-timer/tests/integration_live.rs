use provider_core::{Schedule, SchedulerConfig};
use provider_timer::TimerSource;
use std::env;
use std::error::Error;

fn should_run() -> bool {
    matches!(env::var("RUN_LIVE_TESTS"), Ok(val) if val == "true")
}

fn sample_tenant() -> greentic_types::TenantCtx {
    use greentic_types::{EnvId, TenantCtx, TenantId};
    let env = EnvId::try_from("dev").unwrap();
    let tenant = TenantId::try_from("live-tests").unwrap();
    TenantCtx::new(env, tenant)
}

#[test]
fn live_timer_smoke() -> Result<(), Box<dyn Error>> {
    if !should_run() {
        eprintln!("Skipping live timer test; set RUN_LIVE_TESTS=true to enable.");
        return Ok(());
    }

    let source = TimerSource::new(SchedulerConfig {
        schedules: vec![Schedule {
            name: "live".into(),
            cron: "*/5 * * * *".into(),
            topic: "timer.live".into(),
            payload: serde_json::json!({"kind": "live"}),
        }],
    });

    let event = source.fire(sample_tenant(), "live")?;
    assert_eq!(event.topic, "timer.live");
    Ok(())
}
