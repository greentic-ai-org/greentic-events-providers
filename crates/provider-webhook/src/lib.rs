use provider_core::{
    HttpEndpointConfig, ProviderError, WebhookRoute, new_event, set_idempotency_key,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Minimal representation of an inbound HTTP request supplied by the host/runner.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InboundHttpRequest {
    pub method: String,
    pub path: String,
    pub headers: BTreeMap<String, String>,
    pub body: Value,
    pub correlation_id: Option<String>,
}

/// Helper to handle inbound webhook requests and map them into EventEnvelope instances.
pub struct WebhookSource {
    config: HttpEndpointConfig,
}

impl WebhookSource {
    pub fn new(config: HttpEndpointConfig) -> Self {
        Self { config }
    }

    pub fn handle_request(
        &self,
        tenant: greentic_types::TenantCtx,
        request: InboundHttpRequest,
    ) -> Result<greentic_types::EventEnvelope, ProviderError> {
        let route = self
            .match_route(&request.path)
            .ok_or_else(|| ProviderError::Config(format!("no route for path {}", request.path)))?;

        let mut metadata = self.request_metadata(&request);
        let signature_valid = if route.secret_ref.is_some() {
            // Host-driven signature validation will be added later; mark as false for now.
            "false"
        } else {
            "true"
        };
        metadata.insert("signature_valid".into(), signature_valid.into());
        metadata.insert("topic_prefix".into(), route.topic_prefix.clone());

        if let Some(key) = request
            .headers
            .get("idempotency-key")
            .or_else(|| request.headers.get("Idempotency-Key"))
        {
            set_idempotency_key(&mut metadata, key.clone());
        }

        let event_type = detect_event_type(&request.body).unwrap_or_else(|| "received".to_string());
        let topic = format!("{}.{}", route.topic_prefix, event_type);

        Ok(new_event(
            topic,
            "com.greentic.webhook.generic.v1",
            "webhook-gateway",
            tenant,
            Some(request.path.clone()),
            request.correlation_id.clone(),
            request.body.clone(),
            metadata,
        ))
    }

    fn match_route(&self, path: &str) -> Option<&WebhookRoute> {
        let normalized = strip_base(&self.config.base_path, path);
        self.config
            .routes
            .iter()
            .find(|route| normalized == route.path)
    }

    fn request_metadata(&self, request: &InboundHttpRequest) -> BTreeMap<String, String> {
        let mut metadata = BTreeMap::new();
        metadata.insert("http_method".into(), request.method.clone());
        metadata.insert("path".into(), request.path.clone());
        if let Some(correlation) = &request.correlation_id {
            metadata.insert("correlation_id".into(), correlation.clone());
        }
        for (key, value) in request.headers.iter() {
            metadata.insert(format!("header:{}", key.to_lowercase()), value.clone());
        }
        metadata
    }
}

/// Outbound sink configuration for webhook sink.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutboundWebhookConfig {
    pub url: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
}

/// Representation of an HTTP request to be executed by the host.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutgoingWebhookRequest {
    pub method: String,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub body: Value,
}

/// Build an outbound HTTP request for the sink component.
pub fn build_outgoing_request(
    config: &OutboundWebhookConfig,
    event: &greentic_types::EventEnvelope,
) -> Result<OutgoingWebhookRequest, ProviderError> {
    let mut headers = config.headers.clone();
    headers.insert("content-type".into(), "application/json".into());
    if let Some(correlation) = &event.correlation_id {
        headers.insert("x-correlation-id".into(), correlation.clone());
    }

    Ok(OutgoingWebhookRequest {
        method: "POST".into(),
        url: config.url.clone(),
        headers,
        body: event.payload.clone(),
    })
}

fn strip_base(base: &str, path: &str) -> String {
    let trimmed_base = base.trim_end_matches('/');
    let mut trimmed_path = path.trim_start_matches(trimmed_base).to_string();
    if trimmed_path.is_empty() {
        trimmed_path.push('/');
    }
    trimmed_path
}

fn detect_event_type(body: &Value) -> Option<String> {
    match body {
        Value::Object(map) => map
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use provider_core::{HttpEndpointConfig, WebhookRoute};
    use serde_json::json;

    fn sample_tenant() -> greentic_types::TenantCtx {
        use greentic_types::{EnvId, TeamId, TenantCtx, TenantId};

        let env = EnvId::try_from("dev").unwrap();
        let tenant = TenantId::try_from("acme").unwrap();
        let team = Some(TeamId::try_from("core").unwrap());
        TenantCtx::new(env, tenant).with_team(team)
    }

    fn sample_config() -> HttpEndpointConfig {
        HttpEndpointConfig {
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
        }
    }

    #[test]
    fn maps_inbound_request_to_event() {
        let source = WebhookSource::new(sample_config());
        let req = InboundHttpRequest {
            method: "POST".into(),
            path: "/webhook/stripe".into(),
            headers: BTreeMap::from([("idempotency-key".into(), "idem-1".into())]),
            body: json!({"type": "payment_succeeded", "id": "evt_1"}),
            correlation_id: Some("req-123".into()),
        };

        let event = source
            .handle_request(sample_tenant(), req.clone())
            .expect("event");

        assert_eq!(event.topic, "webhook.stripe.payment_succeeded");
        assert_eq!(event.subject, Some("/webhook/stripe".into()));
        assert_eq!(event.payload, req.body);
        assert_eq!(
            event.metadata.get("idempotency_key"),
            Some(&"idem-1".to_string())
        );
        assert_eq!(event.metadata.get("signature_valid"), Some(&"false".into()));
    }

    #[test]
    fn builds_outgoing_request() {
        let cfg = OutboundWebhookConfig {
            url: "https://example.test/endpoint".into(),
            headers: BTreeMap::from([("x-custom".into(), "value".into())]),
        };
        let event = new_event(
            "webhook.outgoing",
            "com.greentic.webhook.generic.v1",
            "webhook-gateway",
            sample_tenant(),
            Some("/webhook".into()),
            Some("req-1".into()),
            json!({"hello": "world"}),
            BTreeMap::new(),
        );

        let outgoing = build_outgoing_request(&cfg, &event).expect("build");
        assert_eq!(outgoing.url, cfg.url);
        assert_eq!(outgoing.method, "POST");
        assert_eq!(outgoing.body, json!({"hello": "world"}));
        assert_eq!(
            outgoing.headers.get("x-correlation-id"),
            Some(&"req-1".to_string())
        );
    }
}
