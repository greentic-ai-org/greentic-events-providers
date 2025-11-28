use chrono::{DateTime, Utc};
use provider_core::{ProviderError, new_event};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EmailProvider {
    MsGraph,
    Gmail,
}

/// Minimal inbound email representation supplied by host/poller.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InboundEmail {
    pub provider: EmailProvider,
    pub folder_or_label: String,
    pub message_id: String,
    pub subject: String,
    pub from: String,
    pub to: Vec<String>,
    #[serde(default)]
    pub cc: Vec<String>,
    #[serde(default)]
    pub bcc: Vec<String>,
    pub received_at: DateTime<Utc>,
    pub body: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
}

/// Build an EventEnvelope for an inbound email.
pub fn map_inbound_email(
    tenant: greentic_types::TenantCtx,
    email: &InboundEmail,
) -> greentic_types::EventEnvelope {
    let topic = match email.provider {
        EmailProvider::MsGraph => format!("email.in.msgraph.{}", email.folder_or_label),
        EmailProvider::Gmail => format!("email.in.gmail.{}", email.folder_or_label),
    };

    let mut metadata = BTreeMap::new();
    metadata.insert(
        "provider".into(),
        format!("{:?}", email.provider).to_lowercase(),
    );
    metadata.insert("folder_or_label".into(), email.folder_or_label.clone());
    metadata.insert("message_id".into(), email.message_id.clone());

    for (k, v) in email.headers.iter() {
        metadata.insert(format!("header:{}", k.to_lowercase()), v.clone());
    }

    new_event(
        topic,
        "com.greentic.email.generic.v1",
        "email-provider",
        tenant,
        Some(email.subject.clone()),
        Some(email.message_id.clone()),
        json!({
            "subject": email.subject,
            "from": email.from,
            "to": email.to,
            "cc": email.cc,
            "bcc": email.bcc,
            "body": email.body,
            "received_at": email.received_at,
            "headers": email.headers,
        }),
        metadata,
    )
}

/// Generic outbound email request payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutboundEmail {
    pub provider: EmailProvider,
    pub to: Vec<String>,
    pub subject: String,
    pub body: String,
    #[serde(default)]
    pub cc: Vec<String>,
    #[serde(default)]
    pub bcc: Vec<String>,
    pub from_override: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmailSendRequest {
    pub provider: EmailProvider,
    pub payload: Value,
}

/// Translate an outbound EventEnvelope into a provider-specific request representation.
pub fn build_send_request(
    event: &greentic_types::EventEnvelope,
) -> Result<EmailSendRequest, ProviderError> {
    let provider = detect_outbound_provider(&event.topic)?;
    let payload = event.payload.clone();
    let to = expect_array_strings(&payload, "to")?;
    let subject = expect_string(&payload, "subject")?;
    let body = expect_string(&payload, "body")?;
    let cc = optional_array_strings(&payload, "cc");
    let bcc = optional_array_strings(&payload, "bcc");
    let from_override = payload
        .get("from")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    match provider {
        EmailProvider::MsGraph => Ok(EmailSendRequest {
            provider,
            payload: json!({
                "message": {
                    "subject": subject,
                    "body": { "contentType": "HTML", "content": body },
                    "toRecipients": to.iter().map(|addr| json!({"emailAddress": {"address": addr}})).collect::<Vec<_>>(),
                    "ccRecipients": cc.iter().map(|addr| json!({"emailAddress": {"address": addr}})).collect::<Vec<_>>(),
                    "bccRecipients": bcc.iter().map(|addr| json!({"emailAddress": {"address": addr}})).collect::<Vec<_>>(),
                    "from": from_override.as_ref().map(|addr| json!({"emailAddress": {"address": addr}})),
                },
                "saveToSentItems": false
            }),
        }),
        EmailProvider::Gmail => Ok(EmailSendRequest {
            provider,
            payload: json!({
                "message": {
                    "subject": subject,
                    "body": body,
                    "to": to,
                    "cc": cc,
                    "bcc": bcc,
                    "from": from_override,
                }
            }),
        }),
    }
}

fn detect_outbound_provider(topic: &str) -> Result<EmailProvider, ProviderError> {
    if topic.starts_with("email.out.msgraph") {
        Ok(EmailProvider::MsGraph)
    } else if topic.starts_with("email.out.gmail") {
        Ok(EmailProvider::Gmail)
    } else {
        Err(ProviderError::Config(format!(
            "unsupported outbound email topic: {}",
            topic
        )))
    }
}

fn expect_string(payload: &Value, key: &str) -> Result<String, ProviderError> {
    payload
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| ProviderError::Config(format!("missing string field {}", key)))
}

fn expect_array_strings(payload: &Value, key: &str) -> Result<Vec<String>, ProviderError> {
    payload
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty())
        .ok_or_else(|| ProviderError::Config(format!("missing string array field {}", key)))
}

fn optional_array_strings(payload: &Value, key: &str) -> Vec<String> {
    payload
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn maps_inbound_email_to_event() {
        let email = InboundEmail {
            provider: EmailProvider::MsGraph,
            folder_or_label: "inbox".into(),
            message_id: "msg-1".into(),
            subject: "Hello".into(),
            from: "ops@example.com".into(),
            to: vec!["team@example.com".into()],
            cc: vec![],
            bcc: vec![],
            received_at: Utc::now(),
            body: "Test".into(),
            headers: BTreeMap::from([("X-Test".into(), "1".into())]),
        };

        let event = map_inbound_email(sample_tenant(), &email);
        assert_eq!(event.topic, "email.in.msgraph.inbox");
        assert_eq!(event.subject, Some("Hello".into()));
        assert_eq!(event.correlation_id, Some("msg-1".into()));
        assert_eq!(event.metadata.get("provider"), Some(&"msgraph".into()));
    }

    #[test]
    fn builds_msgraph_send_request() {
        let payload = json!({
            "to": ["a@example.com"],
            "subject": "Hi",
            "body": "<p>Body</p>",
            "cc": ["c@example.com"],
            "bcc": [],
            "from": "noreply@example.com"
        });
        let event = greentic_types::EventEnvelope {
            id: greentic_types::EventId::new("evt-1").unwrap(),
            topic: "email.out.msgraph".into(),
            r#type: "t".into(),
            source: "s".into(),
            tenant: sample_tenant(),
            subject: Some("Hi".into()),
            time: Utc::now(),
            correlation_id: None,
            payload: payload.clone(),
            metadata: BTreeMap::new(),
        };

        let request = build_send_request(&event).expect("request");
        assert_eq!(request.provider, EmailProvider::MsGraph);
        assert!(
            request
                .payload
                .get("message")
                .and_then(|m| m.get("toRecipients"))
                .is_some()
        );
    }

    #[test]
    fn rejects_unknown_topic() {
        let payload = json!({"to": ["a@example.com"], "subject": "Hi", "body": "text"});
        let event = greentic_types::EventEnvelope {
            id: greentic_types::EventId::new("evt-1").unwrap(),
            topic: "email.out.other".into(),
            r#type: "t".into(),
            source: "s".into(),
            tenant: sample_tenant(),
            subject: Some("Hi".into()),
            time: Utc::now(),
            correlation_id: None,
            payload,
            metadata: BTreeMap::new(),
        };

        let err = build_send_request(&event).unwrap_err();
        matches!(err, ProviderError::Config(_));
    }
}
