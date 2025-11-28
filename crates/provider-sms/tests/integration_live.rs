use provider_sms::{
    TwilioSinkConfig, TwilioSourceConfig, TwilioWebhookPayload, build_send_request,
};
use std::collections::BTreeMap;
use std::env;
use std::error::Error;

fn should_run() -> bool {
    matches!(env::var("RUN_LIVE_TESTS"), Ok(val) if val == "true")
}

fn should_call_network() -> bool {
    matches!(env::var("RUN_LIVE_HTTP"), Ok(val) if val == "true")
}

fn collect_env(required: &[&str]) -> Option<BTreeMap<String, String>> {
    let mut missing = Vec::new();
    let mut vars = BTreeMap::new();
    for key in required {
        match env::var(key) {
            Ok(val) if !val.is_empty() => {
                vars.insert(key.to_string(), val);
            }
            _ => missing.push(*key),
        }
    }
    if missing.is_empty() {
        Some(vars)
    } else {
        eprintln!(
            "Skipping live SMS test; missing env vars: {}",
            missing.join(", ")
        );
        None
    }
}

fn sample_tenant() -> greentic_types::TenantCtx {
    use greentic_types::{EnvId, TenantCtx, TenantId};
    let env = EnvId::try_from("dev").unwrap();
    let tenant = TenantId::try_from("live-tests").unwrap();
    TenantCtx::new(env, tenant)
}

#[test]
fn live_twilio_inbound_smoke() -> Result<(), Box<dyn Error>> {
    if !should_run() {
        eprintln!("Skipping live Twilio inbound test; set RUN_LIVE_TESTS=true to enable.");
        return Ok(());
    }
    let vars = match collect_env(&[
        "TWILIO_ACCOUNT_SID",
        "TWILIO_AUTH_TOKEN",
        "TWILIO_FROM_NUMBER",
        "TWILIO_TO_NUMBER",
    ]) {
        Some(v) => v,
        None => return Ok(()),
    };

    if should_call_network() {
        let client = reqwest::blocking::Client::new();
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}.json",
            vars["TWILIO_ACCOUNT_SID"]
        );
        let res = client
            .get(url)
            .basic_auth(
                &vars["TWILIO_ACCOUNT_SID"],
                Some(&vars["TWILIO_AUTH_TOKEN"]),
            )
            .send()?;
        if !res.status().is_success() {
            return Err(format!("Twilio account check failed: {}", res.status()).into());
        }
    } else {
        eprintln!("RUN_LIVE_HTTP not set; skipping Twilio API call.");
    }

    let cfg = TwilioSourceConfig {
        phone_aliases: BTreeMap::from([(vars["TWILIO_TO_NUMBER"].clone(), "live".into())]),
        signing_secret_ref: None,
    };
    let payload = TwilioWebhookPayload {
        from: vars["TWILIO_FROM_NUMBER"].clone(),
        to: vars["TWILIO_TO_NUMBER"].clone(),
        body: "Live inbound smoke".into(),
        message_sid: "live-sid-1".into(),
        raw: serde_json::json!({"Body": "Live inbound smoke"}),
        headers: BTreeMap::new(),
    };

    let event = provider_sms::handle_inbound_sms(&cfg, sample_tenant(), payload)?;
    assert!(event.topic.starts_with("sms.in.twilio"));
    Ok(())
}

#[test]
fn live_twilio_outbound_smoke() -> Result<(), Box<dyn Error>> {
    if !should_run() {
        eprintln!("Skipping live Twilio outbound test; set RUN_LIVE_TESTS=true to enable.");
        return Ok(());
    }
    let vars = match collect_env(&[
        "TWILIO_ACCOUNT_SID",
        "TWILIO_AUTH_TOKEN",
        "TWILIO_FROM_NUMBER",
        "TWILIO_TO_NUMBER",
    ]) {
        Some(v) => v,
        None => return Ok(()),
    };

    let cfg = TwilioSinkConfig {
        account_sid: vars["TWILIO_ACCOUNT_SID"].clone(),
        auth_token_ref: Some("TWILIO_AUTH_TOKEN".into()),
        default_from: Some(vars["TWILIO_FROM_NUMBER"].clone()),
    };
    let envelope = greentic_types::EventEnvelope {
        id: greentic_types::EventId::new("live-twilio-1")?,
        topic: "sms.out.twilio".into(),
        r#type: "com.greentic.sms.twilio.outbound.v1".into(),
        source: "integration-test".into(),
        tenant: sample_tenant(),
        subject: None,
        time: chrono::Utc::now(),
        correlation_id: None,
        payload: serde_json::json!({
            "to": vars["TWILIO_TO_NUMBER"].clone(),
            "body": "Live outbound smoke"
        }),
        metadata: BTreeMap::new(),
    };

    let req = build_send_request(&cfg, &envelope)?;
    assert!(req.url.contains(&cfg.account_sid));

    if should_call_network() {
        let client = reqwest::blocking::Client::new();
        let res = client
            .post(&req.url)
            .basic_auth(&cfg.account_sid, Some(&vars["TWILIO_AUTH_TOKEN"]))
            .form(&req.body)
            .send()?;
        if !res.status().is_success() {
            return Err(format!("Twilio send failed: {}", res.status()).into());
        }
    } else {
        eprintln!("RUN_LIVE_HTTP not set; skipping Twilio SMS send.");
    }
    Ok(())
}
