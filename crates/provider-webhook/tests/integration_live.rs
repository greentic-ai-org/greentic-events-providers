use provider_webhook::{InboundHttpRequest, WebhookSource};
use std::collections::BTreeMap;
use std::env;
use std::error::Error;

fn should_run() -> bool {
    matches!(env::var("RUN_LIVE_TESTS"), Ok(val) if val == "true")
}

fn should_call_network() -> bool {
    matches!(env::var("RUN_LIVE_HTTP"), Ok(val) if val == "true")
}

fn sample_tenant() -> greentic_types::TenantCtx {
    use greentic_types::{EnvId, TenantCtx, TenantId};
    let env = EnvId::try_from("dev").unwrap();
    let tenant = TenantId::try_from("live-tests").unwrap();
    TenantCtx::new(env, tenant)
}

#[test]
fn live_webhook_smoke() -> Result<(), Box<dyn Error>> {
    if !should_run() {
        eprintln!("Skipping live webhook test; set RUN_LIVE_TESTS=true to enable.");
        return Ok(());
    }

    let cfg = provider_core::HttpEndpointConfig {
        base_path: "/webhook".into(),
        routes: vec![provider_core::WebhookRoute {
            path: "/live".into(),
            secret_ref: None,
            topic_prefix: "webhook.live".into(),
        }],
    };
    let source = WebhookSource::new(cfg);
    let request = InboundHttpRequest {
        method: "POST".into(),
        path: "/webhook/live".into(),
        headers: BTreeMap::from([("content-type".into(), "application/json".into())]),
        body: serde_json::json!({"type": "smoke", "ok": true}),
        correlation_id: Some("live-webhook-1".into()),
        signature_validated: true,
    };
    let event = source.handle_request(sample_tenant(), request)?;
    assert!(event.topic.starts_with("webhook.live"));

    // Optional: call a real HTTP echo to mimic host bridge if enabled.
    if should_call_network() {
        let client = reqwest::blocking::Client::new();
        let res = client
            .post("https://httpbin.org/post")
            .json(&serde_json::json!({"ok": true}))
            .send()?;
        if !res.status().is_success() {
            return Err(format!("Webhook echo call failed: {}", res.status()).into());
        }
    } else {
        eprintln!("RUN_LIVE_HTTP not set; skipping external webhook echo call.");
    }
    Ok(())
}
