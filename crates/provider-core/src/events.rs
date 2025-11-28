use chrono::Utc;
use greentic_types::{EventEnvelope, EventId, TenantCtx};
use serde_json::Value;
use std::collections::BTreeMap;
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
/// Build a new EventEnvelope with common defaults.
pub fn new_event(
    topic: impl Into<String>,
    type_: impl Into<String>,
    source: impl Into<String>,
    tenant: TenantCtx,
    subject: Option<String>,
    correlation_id: Option<String>,
    payload: Value,
    metadata: BTreeMap<String, String>,
) -> EventEnvelope {
    EventEnvelope {
        id: EventId::new(Uuid::new_v4().to_string())
            .expect("failed to build EventId from generated uuid"),
        topic: topic.into(),
        r#type: type_.into(),
        source: source.into(),
        tenant,
        subject,
        time: Utc::now(),
        correlation_id,
        payload,
        metadata,
    }
}

/// Set or override the idempotency key in metadata.
pub fn set_idempotency_key(metadata: &mut BTreeMap<String, String>, key: impl Into<String>) {
    metadata.insert("idempotency_key".to_string(), key.into());
}
