use provider_core::secrets::{SecretProvider, resolve_secret};
use provider_core::{ProviderError, new_event};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TwilioSourceConfig {
    /// Map of inbound phone numbers to aliases for topic suffixes.
    pub phone_aliases: BTreeMap<String, String>,
    /// Optional reference to a signing secret for webhook validation.
    pub signing_secret_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TwilioWebhookPayload {
    pub from: String,
    pub to: String,
    pub body: String,
    pub message_sid: String,
    #[serde(default)]
    pub raw: Value,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    /// Host-provided flag indicating whether the webhook signature has been validated.
    #[serde(default)]
    pub signature_validated: bool,
}

pub fn handle_inbound_sms(
    cfg: &TwilioSourceConfig,
    tenant: greentic_types::TenantCtx,
    payload: TwilioWebhookPayload,
) -> Result<greentic_types::EventEnvelope, ProviderError> {
    let alias = cfg
        .phone_aliases
        .get(&payload.to)
        .cloned()
        .unwrap_or_else(|| "unknown".into());
    let topic = format!("sms.in.twilio.{}", alias);

    let mut metadata = BTreeMap::new();
    metadata.insert("provider".into(), "twilio".into());
    metadata.insert("from".into(), payload.from.clone());
    metadata.insert("to".into(), payload.to.clone());
    metadata.insert("message_sid".into(), payload.message_sid.clone());
    let signature_valid = payload.signature_validated || cfg.signing_secret_ref.is_none();
    metadata.insert("signature_valid".into(), signature_valid.to_string());
    for (k, v) in payload.headers.iter() {
        metadata.insert(format!("header:{}", k.to_lowercase()), v.clone());
    }

    Ok(new_event(
        topic,
        "com.greentic.sms.twilio.inbound.v1",
        "sms-provider",
        tenant,
        Some(payload.to.clone()),
        Some(payload.message_sid.clone()),
        json!({
            "from": payload.from,
            "to": payload.to,
            "body": payload.body,
            "message_sid": payload.message_sid,
            "raw": payload.raw
        }),
        metadata,
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TwilioSinkConfig {
    pub account_sid: String,
    pub auth_token_ref: Option<String>,
    pub default_from: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwilioSendRequest {
    pub account_sid: String,
    pub auth_token_ref: Option<String>,
    pub url: String,
    pub body: BTreeMap<String, String>,
    pub secret_events: Vec<greentic_types::EventEnvelope>,
}

pub fn build_send_request(
    cfg: &TwilioSinkConfig,
    event: &greentic_types::EventEnvelope,
    secrets: &dyn SecretProvider,
) -> Result<TwilioSendRequest, ProviderError> {
    if !event.topic.starts_with("sms.out.twilio") {
        return Err(ProviderError::Config(format!(
            "unsupported sms topic {}",
            event.topic
        )));
    }

    let mut secret_events = Vec::new();
    if let Some(key) = cfg.auth_token_ref.as_ref() {
        let resolution = resolve_secret(
            secrets,
            key,
            "tenant",
            event.tenant.clone(),
            "sms-provider",
            "twilio auth token",
        )?;
        secret_events.extend(resolution.events);
    }

    let to = expect_string(&event.payload, "to")?;
    let body = expect_string(&event.payload, "body")?;
    let from = event
        .payload
        .get("from")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| cfg.default_from.clone());

    let mut form = BTreeMap::new();
    form.insert("To".into(), to);
    form.insert("Body".into(), body);
    if let Some(from) = from {
        form.insert("From".into(), from);
    }

    Ok(TwilioSendRequest {
        account_sid: cfg.account_sid.clone(),
        auth_token_ref: cfg.auth_token_ref.clone(),
        url: format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            cfg.account_sid
        ),
        body: form,
        secret_events,
    })
}

/// Convenience wrapper that resolves secrets via the Greentic secrets-store (wasm32).
pub fn build_send_request_with_secrets_store(
    cfg: &TwilioSinkConfig,
    event: &greentic_types::EventEnvelope,
) -> Result<TwilioSendRequest, ProviderError> {
    let provider = provider_core::secrets::SecretsStoreProvider;
    build_send_request(cfg, event, &provider)
}

fn expect_string(value: &Value, key: &str) -> Result<String, ProviderError> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| ProviderError::Config(format!("missing string field {}", key)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use provider_core::secrets::StaticSecretProvider;
    use serde_json::json;
    use std::collections::BTreeMap as Map;

    fn tenant() -> greentic_types::TenantCtx {
        use greentic_types::{EnvId, TenantCtx, TenantId};

        let env = EnvId::try_from("dev").unwrap();
        let tenant = TenantId::try_from("acme").unwrap();
        TenantCtx::new(env, tenant)
    }

    #[test]
    fn maps_twilio_webhook_to_event() {
        let cfg = TwilioSourceConfig {
            phone_aliases: BTreeMap::from([("+15550001".into(), "support".into())]),
            signing_secret_ref: Some("TWILIO_AUTH_TOKEN".into()),
        };
        let payload = TwilioWebhookPayload {
            from: "+15559999".into(),
            to: "+15550001".into(),
            body: "Hello".into(),
            message_sid: "SM123".into(),
            raw: json!({"MessageSid": "SM123"}),
            headers: BTreeMap::from([("X-Twilio-Signature".into(), "sig".into())]),
            signature_validated: true,
        };

        let event = handle_inbound_sms(&cfg, tenant(), payload).expect("event");
        assert_eq!(event.topic, "sms.in.twilio.support");
        assert_eq!(event.metadata.get("signature_valid"), Some(&"true".into()));
    }

    #[test]
    fn builds_send_request() {
        let cfg = TwilioSinkConfig {
            account_sid: "AC123".into(),
            auth_token_ref: Some("TWILIO_AUTH_TOKEN".into()),
            default_from: Some("+15550001".into()),
        };
        let event = greentic_types::EventEnvelope {
            id: greentic_types::EventId::new("evt-1").unwrap(),
            topic: "sms.out.twilio".into(),
            r#type: "t".into(),
            source: "s".into(),
            tenant: tenant(),
            subject: None,
            time: chrono::Utc::now(),
            correlation_id: None,
            payload: json!({"to": "+15559999", "body": "Hello"}),
            metadata: BTreeMap::new(),
        };

        let secrets =
            StaticSecretProvider::new(Map::from([("TWILIO_AUTH_TOKEN".into(), b"token".to_vec())]));
        let req = build_send_request(&cfg, &event, &secrets).expect("req");
        assert_eq!(req.body.get("To"), Some(&"+15559999".into()));
        assert_eq!(req.body.get("From"), Some(&"+15550001".into()));
        assert_eq!(req.secret_events.len(), 1);
        assert_eq!(req.secret_events[0].topic, "greentic.secrets.put");
    }

    #[test]
    fn missing_secret_emits_missing_detected() {
        let cfg = TwilioSinkConfig {
            account_sid: "AC123".into(),
            auth_token_ref: Some("TWILIO_AUTH_TOKEN".into()),
            default_from: Some("+15550001".into()),
        };
        let event = greentic_types::EventEnvelope {
            id: greentic_types::EventId::new("evt-2").unwrap(),
            topic: "sms.out.twilio".into(),
            r#type: "t".into(),
            source: "s".into(),
            tenant: tenant(),
            subject: None,
            time: chrono::Utc::now(),
            correlation_id: None,
            payload: json!({"to": "+15559999", "body": "Hello"}),
            metadata: BTreeMap::new(),
        };

        let secrets = StaticSecretProvider::empty();
        let req = build_send_request(&cfg, &event, &secrets).expect("req");
        assert_eq!(req.secret_events.len(), 1);
        assert_eq!(
            req.secret_events[0].topic,
            "greentic.secrets.missing.detected"
        );
    }
}
