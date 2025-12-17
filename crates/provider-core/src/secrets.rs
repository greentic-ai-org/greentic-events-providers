use crate::events::new_event;
use chrono::Utc;
use greentic_types::TenantCtx;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

/// Standard schema version for secrets metadata payloads.
pub const SECRET_EVENT_SCHEMA_VERSION: &str = "v1";

/// Result of attempting to resolve a secret, capturing emitted metadata events.
pub struct SecretResolution {
    pub value: Option<Vec<u8>>,
    pub events: Vec<greentic_types::EventEnvelope>,
}

/// Interface for resolving secrets. Real components should use the Greentic secrets-store
/// implementation; tests can supply an in-memory map.
pub trait SecretProvider {
    fn get_secret(&self, key: &str) -> Result<Option<Vec<u8>>, crate::ProviderError>;
}

/// Default provider that resolves secrets via `greentic:secrets-store@1.0.0`.
pub struct SecretsStoreProvider;

#[cfg(target_arch = "wasm32")]
impl SecretProvider for SecretsStoreProvider {
    fn get_secret(&self, key: &str) -> Result<Option<Vec<u8>>, crate::ProviderError> {
        use greentic_interfaces_guest::secrets_store::SecretsStore;

        SecretsStore::get(key)
            .map_err(|err| crate::ProviderError::Auth(format!("secrets-store error: {err:?}")))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl SecretProvider for SecretsStoreProvider {
    fn get_secret(&self, _key: &str) -> Result<Option<Vec<u8>>, crate::ProviderError> {
        Err(crate::ProviderError::Auth(
            "secrets-store is only available in wasm32 targets".into(),
        ))
    }
}

/// Static provider for tests/fixtures.
pub struct StaticSecretProvider {
    secrets: BTreeMap<String, Vec<u8>>,
}

impl StaticSecretProvider {
    pub fn new(secrets: BTreeMap<String, Vec<u8>>) -> Self {
        Self { secrets }
    }

    pub fn empty() -> Self {
        Self {
            secrets: BTreeMap::new(),
        }
    }
}

impl SecretProvider for StaticSecretProvider {
    fn get_secret(&self, key: &str) -> Result<Option<Vec<u8>>, crate::ProviderError> {
        Ok(self.secrets.get(key).cloned())
    }
}

/// Resolve a secret via the supplied provider and emit metadata-only events.
pub fn resolve_secret(
    secrets: &dyn SecretProvider,
    key: &str,
    scope: &str,
    tenant: TenantCtx,
    source: &str,
    context: &str,
) -> Result<SecretResolution, crate::ProviderError> {
    match secrets.get_secret(key)? {
        Some(bytes) => Ok(SecretResolution {
            value: Some(bytes),
            events: vec![secret_put_event(key, scope, tenant, source)],
        }),
        None => Ok(SecretResolution {
            value: None,
            events: vec![secret_missing_detected_event(
                key, scope, tenant, source, context, source,
            )],
        }),
    }
}

/// Builds the payload for `greentic.secrets.put` events.
pub fn secret_put_event(
    key: &str,
    scope: &str,
    tenant: TenantCtx,
    source: &str,
) -> greentic_types::EventEnvelope {
    new_event(
        "greentic.secrets.put",
        "com.greentic.secrets.audit.v1",
        source,
        tenant.clone(),
        None,
        None,
        json!(put_delete_payload(key, scope, tenant, "success")),
        BTreeMap::new(),
    )
}

/// Builds the payload for `greentic.secrets.delete` events.
pub fn secret_delete_event(
    key: &str,
    scope: &str,
    tenant: TenantCtx,
    source: &str,
    result: &str,
) -> greentic_types::EventEnvelope {
    new_event(
        "greentic.secrets.delete",
        "com.greentic.secrets.audit.v1",
        source,
        tenant.clone(),
        None,
        None,
        json!(put_delete_payload(key, scope, tenant, result)),
        BTreeMap::new(),
    )
}

/// Builds the payload for `greentic.secrets.rotate.requested` events.
pub fn secret_rotate_requested_event(
    key: &str,
    scope: &str,
    rotation_id: &str,
    result: &str,
    tenant: TenantCtx,
    source: &str,
    error: Option<&str>,
) -> greentic_types::EventEnvelope {
    new_event(
        "greentic.secrets.rotate.requested",
        "com.greentic.secrets.audit.v1",
        source,
        tenant.clone(),
        None,
        None,
        rotation_payload(key, scope, rotation_id, result, error, tenant),
        BTreeMap::new(),
    )
}

/// Builds the payload for `greentic.secrets.rotate.completed` events.
pub fn secret_rotate_completed_event(
    key: &str,
    scope: &str,
    rotation_id: &str,
    result: &str,
    tenant: TenantCtx,
    source: &str,
    error: Option<&str>,
) -> greentic_types::EventEnvelope {
    new_event(
        "greentic.secrets.rotate.completed",
        "com.greentic.secrets.audit.v1",
        source,
        tenant.clone(),
        None,
        None,
        rotation_payload(key, scope, rotation_id, result, error, tenant),
        BTreeMap::new(),
    )
}

/// Builds the payload for `greentic.secrets.missing.detected` events.
pub fn secret_missing_detected_event(
    key: &str,
    scope: &str,
    tenant: TenantCtx,
    detected_by: &str,
    context: &str,
    source: &str,
) -> greentic_types::EventEnvelope {
    new_event(
        "greentic.secrets.missing.detected",
        "com.greentic.secrets.audit.v1",
        source,
        tenant.clone(),
        None,
        None,
        json!({
            "schema_version": SECRET_EVENT_SCHEMA_VERSION,
            "key": key,
            "scope": scope,
            "detected_by": detected_by,
            "context": context,
            "timestamp_utc": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "tenant_ctx": tenant_context_payload(&tenant),
        }),
        BTreeMap::new(),
    )
}

fn put_delete_payload(key: &str, scope: &str, tenant: TenantCtx, result: &str) -> Value {
    json!({
        "schema_version": SECRET_EVENT_SCHEMA_VERSION,
        "key": key,
        "scope": scope,
        "tenant_ctx": tenant_context_payload(&tenant),
        "result": result,
        "timestamp_utc": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    })
}

fn rotation_payload(
    key: &str,
    scope: &str,
    rotation_id: &str,
    result: &str,
    error: Option<&str>,
    tenant: TenantCtx,
) -> Value {
    let mut payload = Map::new();
    payload.insert(
        "schema_version".into(),
        Value::String(SECRET_EVENT_SCHEMA_VERSION.into()),
    );
    payload.insert("key".into(), Value::String(key.into()));
    payload.insert("scope".into(), Value::String(scope.into()));
    payload.insert("rotation_id".into(), Value::String(rotation_id.into()));
    payload.insert("result".into(), Value::String(result.into()));
    payload.insert(
        "timestamp_utc".into(),
        Value::String(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
    );
    payload.insert("tenant_ctx".into(), tenant_context_payload(&tenant));
    if let Some(err) = error {
        payload.insert("error".into(), Value::String(err.into()));
    }
    Value::Object(payload)
}

fn tenant_context_payload(tenant: &TenantCtx) -> Value {
    json!({
        "env": tenant.env.as_str(),
        "tenant": tenant.tenant.as_str(),
        "team": tenant.team.as_ref().map(|t| t.as_str()),
        "user": tenant.user.as_ref().map(|u| u.as_str()),
    })
}
