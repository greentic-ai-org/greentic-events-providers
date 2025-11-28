use provider_core::{
    HttpEndpointConfig, Schedule, SchedulerConfig, WebhookRoute, new_event, set_idempotency_key,
};
use serde_json::json;
use std::collections::BTreeMap;

fn sample_tenant() -> greentic_types::TenantCtx {
    use greentic_types::{EnvId, TeamId, TenantCtx, TenantId};

    let env = EnvId::try_from("dev").unwrap();
    let tenant = TenantId::try_from("acme").unwrap();
    let team = Some(TeamId::try_from("core").unwrap());
    TenantCtx::new(env, tenant).with_team(team)
}

#[test]
fn http_endpoint_config_roundtrip() {
    let cfg = HttpEndpointConfig {
        base_path: "/webhook".into(),
        routes: vec![
            WebhookRoute {
                path: "/stripe".into(),
                secret_ref: Some("events/webhook/dev/acme/stripe".into()),
                topic_prefix: "webhook.stripe".into(),
            },
            WebhookRoute {
                path: "/github".into(),
                secret_ref: None,
                topic_prefix: "webhook.github".into(),
            },
        ],
    };

    let value = serde_json::to_string(&cfg).expect("serialize");
    let roundtrip: HttpEndpointConfig = serde_json::from_str(&value).expect("deserialize");
    assert_eq!(cfg, roundtrip);
}

#[test]
fn scheduler_config_roundtrip() {
    let cfg = SchedulerConfig {
        schedules: vec![
            Schedule {
                name: "daily".into(),
                cron: "0 0 * * *".into(),
                topic: "timer.daily.report".into(),
                payload: json!({"kind":"daily"}),
            },
            Schedule {
                name: "hourly".into(),
                cron: "0 * * * *".into(),
                topic: "timer.hourly.sync".into(),
                payload: json!({"kind":"hourly"}),
            },
        ],
    };

    let value = serde_json::to_string(&cfg).expect("serialize");
    let roundtrip: SchedulerConfig = serde_json::from_str(&value).expect("deserialize");
    assert_eq!(cfg, roundtrip);
}

#[test]
fn new_event_sets_defaults() {
    let tenant = sample_tenant();
    let mut metadata = BTreeMap::new();
    metadata.insert("http_method".into(), "POST".into());

    let event = new_event(
        "webhook.stripe.payment_succeeded",
        "com.greentic.webhook.generic.v1",
        "webhook-gateway",
        tenant,
        Some("/webhook/stripe".into()),
        Some("req-123".into()),
        json!({"id": "evt_1"}),
        metadata.clone(),
    );

    assert!(!event.id.as_str().is_empty());
    assert_eq!(event.topic, "webhook.stripe.payment_succeeded");
    assert_eq!(event.r#type, "com.greentic.webhook.generic.v1");
    assert_eq!(event.source, "webhook-gateway");
    assert_eq!(event.subject, Some("/webhook/stripe".into()));
    assert_eq!(event.correlation_id, Some("req-123".into()));
    assert_eq!(event.payload, json!({"id": "evt_1"}));
    assert!(event.time.timestamp() > 0);
    assert_eq!(event.metadata, metadata);
}

#[test]
fn idempotency_key_helper_sets_metadata() {
    let mut metadata = BTreeMap::new();
    set_idempotency_key(&mut metadata, "abc");
    assert_eq!(metadata.get("idempotency_key"), Some(&"abc".to_string()));
    set_idempotency_key(&mut metadata, "def");
    assert_eq!(metadata.get("idempotency_key"), Some(&"def".to_string()));
}
