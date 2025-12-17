use provider_core::secrets::{SecretProvider, resolve_secret};
use provider_core::{
    HttpEndpointConfig, ProviderError, WebhookRoute, new_event, set_idempotency_key,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Result of handling a webhook request, including secrets metadata events.
pub struct WebhookResult {
    pub event: greentic_types::EventEnvelope,
    pub secret_events: Vec<greentic_types::EventEnvelope>,
}

/// Minimal representation of an inbound HTTP request supplied by the host/runner.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InboundHttpRequest {
    pub method: String,
    pub path: String,
    pub headers: BTreeMap<String, String>,
    pub body: Value,
    pub correlation_id: Option<String>,
    /// Host-provided flag indicating whether the request signature was already validated.
    #[serde(default)]
    pub signature_validated: bool,
}

/// Helper to handle inbound webhook requests and map them into EventEnvelope instances.
pub struct WebhookSource {
    config: HttpEndpointConfig,
}

impl WebhookSource {
    pub fn new(config: HttpEndpointConfig) -> Self {
        Self { config }
    }

    /// Resolve secrets via the Greentic secrets-store inside the component (wasm32).
    pub fn handle_request_with_secrets_store(
        &self,
        tenant: greentic_types::TenantCtx,
        request: InboundHttpRequest,
    ) -> Result<WebhookResult, ProviderError> {
        let provider = provider_core::secrets::SecretsStoreProvider;
        self.handle_request(tenant, request, &provider)
    }

    pub fn handle_request(
        &self,
        tenant: greentic_types::TenantCtx,
        request: InboundHttpRequest,
        secrets: &dyn SecretProvider,
    ) -> Result<WebhookResult, ProviderError> {
        let route = self
            .match_route(&request.path)
            .ok_or_else(|| ProviderError::Config(format!("no route for path {}", request.path)))?;

        let mut metadata = self.request_metadata(&request);
        let signature_valid = request.signature_validated || route.secret_ref.is_none();
        metadata.insert("signature_valid".into(), signature_valid.to_string());
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

        let secret_events = resolve_webhook_secret(route, secrets, tenant.clone())?;

        Ok(WebhookResult {
            event: new_event(
                topic,
                "com.greentic.webhook.generic.v1",
                "webhook-gateway",
                tenant,
                Some(request.path.clone()),
                request.correlation_id.clone(),
                request.body.clone(),
                metadata,
            ),
            secret_events,
        })
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

/// Resolve a signing secret (when configured) and emit metadata-only events.
pub fn resolve_webhook_secret(
    route: &WebhookRoute,
    secrets: &dyn SecretProvider,
    tenant: greentic_types::TenantCtx,
) -> Result<Vec<greentic_types::EventEnvelope>, ProviderError> {
    if let Some(key) = route.secret_ref.as_ref() {
        let resolution = resolve_secret(
            secrets,
            key,
            "tenant",
            tenant,
            "webhook-gateway",
            "webhook signing secret",
        )?;
        Ok(resolution.events)
    } else {
        Ok(Vec::new())
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
    use provider_core::secrets::StaticSecretProvider;
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
                    secret_ref: Some("WEBHOOK_SIGNING_SECRET".into()),
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
            signature_validated: true,
        };

        let secrets = StaticSecretProvider::new(BTreeMap::from([(
            "WEBHOOK_SIGNING_SECRET".into(),
            b"sig".to_vec(),
        )]));
        let result = source
            .handle_request(sample_tenant(), req.clone(), &secrets)
            .expect("event");

        assert_eq!(result.event.topic, "webhook.stripe.payment_succeeded");
        assert_eq!(result.event.subject, Some("/webhook/stripe".into()));
        assert_eq!(result.event.payload, req.body);
        assert_eq!(
            result.event.metadata.get("idempotency_key"),
            Some(&"idem-1".to_string())
        );
        assert_eq!(
            result.event.metadata.get("signature_valid"),
            Some(&"true".into())
        );
        assert_eq!(result.secret_events.len(), 1);
        assert_eq!(result.secret_events[0].topic, "greentic.secrets.put");
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

    #[test]
    fn resolves_signing_secret() {
        let route = WebhookRoute {
            path: "/stripe".into(),
            secret_ref: Some("WEBHOOK_SIGNING_SECRET".into()),
            topic_prefix: "webhook.stripe".into(),
        };
        let secrets = StaticSecretProvider::new(BTreeMap::from([(
            "WEBHOOK_SIGNING_SECRET".into(),
            b"sig".to_vec(),
        )]));

        let events =
            resolve_webhook_secret(&route, &secrets, sample_tenant()).expect("events present");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].topic, "greentic.secrets.put");
    }

    #[test]
    fn missing_signing_secret_emits_missing_detected() {
        let route = WebhookRoute {
            path: "/stripe".into(),
            secret_ref: Some("WEBHOOK_SIGNING_SECRET".into()),
            topic_prefix: "webhook.stripe".into(),
        };
        let secrets = StaticSecretProvider::empty();
        let events =
            resolve_webhook_secret(&route, &secrets, sample_tenant()).expect("events present");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].topic, "greentic.secrets.missing.detected");
    }
}
