#![deny(unsafe_op_in_unsafe_fn)]

use anyhow::{Context, Result};
use chrono::Utc;
use greentic_interfaces_guest::component::node::{InvokeResult, NodeError};
use greentic_interfaces_guest::component_entrypoint;
use greentic_interfaces_guest::provider_core;
#[cfg(target_arch = "wasm32")]
use greentic_interfaces_guest::state_store;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::BTreeMap;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
#[used]
#[unsafe(link_section = ".greentic.wasi")]
static WASI_TARGET_MARKER: [u8; 13] = *b"wasm32-wasip2";

component_entrypoint!({
    manifest: crate::describe_payload,
    invoke: crate::handle_message,
    invoke_stream: true,
});

pub fn describe_payload() -> String {
    serde_json::json!({
        "component": {
            "name": "events-provider-email-sendgrid",
            "org": "ai.greentic",
            "version": "0.1.0",
            "world": "greentic:component/component@0.6.0",
            "schemas": {
                "component": "schemas/component.schema.json",
                "input": "schemas/io/input.schema.json",
                "output": "schemas/io/output.schema.json"
            }
        }
    })
    .to_string()
}

pub fn handle_message(operation: String, input: String) -> InvokeResult {
    match handle_invoke(&operation, input.as_bytes()) {
        Ok(bytes) => InvokeResult::Ok(String::from_utf8_lossy(&bytes).into_owned()),
        Err(err) => InvokeResult::Err(NodeError {
            code: "invoke_error".into(),
            message: err.to_string(),
            retryable: false,
            backoff_ms: None,
            details: None,
        }),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ProviderConfig {
    messaging_provider_id: String,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    persistence_key_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct EmailInput {
    config: ProviderConfig,
    #[serde(default)]
    event: Value,
    #[serde(default)]
    handler_id: Option<String>,
    #[serde(default)]
    tenant: Option<String>,
    #[serde(default)]
    team: Option<String>,
    #[serde(default)]
    correlation_id: Option<String>,
    #[serde(default)]
    http: Option<Value>,
    #[serde(default)]
    raw: Option<Value>,
}

#[allow(dead_code)]
struct Component;

impl provider_core::Guest for Component {
    fn describe() -> Vec<u8> {
        serde_json::to_vec(&json!({
            "provider_type": "events.email.sendgrid",
            "capabilities": {
                "operations": ["ingest_http", "publish"],
                "persistence": "state-store",
                "deterministic": true,
            },
            "ops": ["ingest_http", "publish"],
        }))
        .unwrap_or_default()
    }

    fn validate_config(config_json: Vec<u8>) -> Vec<u8> {
        match serde_json::from_slice::<ProviderConfig>(&config_json) {
            Ok(cfg) => json!({"valid": true, "config": cfg})
                .to_string()
                .into_bytes(),
            Err(err) => json!({"valid": false, "error": err.to_string()})
                .to_string()
                .into_bytes(),
        }
    }

    fn healthcheck() -> Vec<u8> {
        json!({"status": "ok"}).to_string().into_bytes()
    }

    fn invoke(op: String, input_json: Vec<u8>) -> Vec<u8> {
        match handle_invoke(&op, &input_json) {
            Ok(res) => res,
            Err(err) => json!({"error": err.to_string()}).to_string().into_bytes(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod exports {
    use super::{Component, provider_core};

    #[unsafe(export_name = "greentic:provider-schema-core/schema-core-api@1.0.0#describe")]
    pub unsafe extern "C" fn export_describe() -> *mut u8 {
        unsafe { provider_core::_export_describe_cabi::<Component>() }
    }

    #[unsafe(export_name = "cabi_post_greentic:provider-schema-core/schema-core-api@1.0.0#describe")]
    pub unsafe extern "C" fn post_describe(ret: *mut u8) {
        unsafe { provider_core::__post_return_describe::<Component>(ret) }
    }

    #[unsafe(export_name = "greentic:provider-schema-core/schema-core-api@1.0.0#validate-config")]
    pub unsafe extern "C" fn export_validate_config(arg0: *mut u8, arg1: usize) -> *mut u8 {
        unsafe { provider_core::_export_validate_config_cabi::<Component>(arg0, arg1) }
    }

    #[unsafe(export_name = "cabi_post_greentic:provider-schema-core/schema-core-api@1.0.0#validate-config")]
    pub unsafe extern "C" fn post_validate_config(ret: *mut u8) {
        unsafe { provider_core::__post_return_validate_config::<Component>(ret) }
    }

    #[unsafe(export_name = "greentic:provider-schema-core/schema-core-api@1.0.0#healthcheck")]
    pub unsafe extern "C" fn export_healthcheck() -> *mut u8 {
        unsafe { provider_core::_export_healthcheck_cabi::<Component>() }
    }

    #[unsafe(export_name = "cabi_post_greentic:provider-schema-core/schema-core-api@1.0.0#healthcheck")]
    pub unsafe extern "C" fn post_healthcheck(ret: *mut u8) {
        unsafe { provider_core::__post_return_healthcheck::<Component>(ret) }
    }

    #[unsafe(export_name = "greentic:provider-schema-core/schema-core-api@1.0.0#invoke")]
    pub unsafe extern "C" fn export_invoke(
        op_ptr: *mut u8,
        op_len: usize,
        input_ptr: *mut u8,
        input_len: usize,
    ) -> *mut u8 {
        unsafe {
            provider_core::_export_invoke_cabi::<Component>(op_ptr, op_len, input_ptr, input_len)
        }
    }

    #[unsafe(export_name = "cabi_post_greentic:provider-schema-core/schema-core-api@1.0.0#invoke")]
    pub unsafe extern "C" fn post_invoke(ret: *mut u8) {
        unsafe { provider_core::__post_return_invoke::<Component>(ret) }
    }
}

#[allow(dead_code)]
fn handle_invoke(op: &str, input_json: &[u8]) -> Result<Vec<u8>> {
    let parsed: EmailInput = serde_json::from_slice(input_json)
        .with_context(|| "ingest input must include config and event")?;
    match op {
        "ingest_http" | "publish" => handle_ingest_http(&parsed),
        other => anyhow::bail!("unsupported op {other}"),
    }
}

#[allow(dead_code)]
fn handle_ingest_http(input: &EmailInput) -> Result<Vec<u8>> {
    if input.config.messaging_provider_id.trim().is_empty() {
        anyhow::bail!("messaging_provider_id is required");
    }
    let receipt_id = stable_receipt_id(&input.event);
    let key = state_key(&input.config, &receipt_id);
    persist_request(&key, input)?;
    let now = Utc::now().to_rfc3339();

    let mut emitted_event = json!({
        "event_id": receipt_id,
        "event_type": "email.received",
        "occurred_at": now,
        "source": {
            "domain": "events",
            "provider": "events.email.sendgrid",
            "handler_id": input.handler_id.clone().unwrap_or_else(|| "default".to_string()),
        },
        "scope": {
            "tenant": input.tenant.clone().unwrap_or_else(|| "default".to_string()),
            "team": input.team,
            "correlation_id": input.correlation_id,
        },
        "payload": input.event,
    });
    if let Some(http) = &input.http {
        emitted_event["http"] = http.clone();
    }
    if let Some(raw) = &input.raw {
        emitted_event["raw"] = raw.clone();
    }

    Ok(json!({
        "receipt_id": receipt_id,
        "status": "queued",
        "state_key": key,
        "emitted_events": [emitted_event],
    })
    .to_string()
    .into_bytes())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueuedEmail {
    messaging_provider_id: String,
    from: Option<String>,
    event: Value,
    queued_at: String,
}

fn state_key(config: &ProviderConfig, receipt_id: &str) -> String {
    let prefix = config
        .persistence_key_prefix
        .as_deref()
        .unwrap_or("events/email/sendgrid/queued");
    format!("{prefix}/{receipt_id}.json")
}

fn stable_receipt_id(event: &Value) -> String {
    let bytes = serde_json::to_vec(event).unwrap_or_default();
    Uuid::new_v5(&Uuid::NAMESPACE_OID, &bytes).to_string()
}

fn persist_request(key: &str, input: &EmailInput) -> Result<()> {
    let queued = QueuedEmail {
        messaging_provider_id: input.config.messaging_provider_id.clone(),
        from: input.config.from.clone(),
        event: input.event.clone(),
        queued_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    };
    let bytes = serde_json::to_vec(&queued)?;

    #[cfg(target_arch = "wasm32")]
    {
        state_store::write(key, &bytes, None)
            .map_err(|e| anyhow::anyhow!("state-store write failed: {e:?}"))?;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let cache = HOST_STATE.get_or_init(|| Mutex::new(BTreeMap::new()));
        let mut guard = cache.lock().expect("host state mutex poisoned");
        guard.insert(key.to_string(), bytes);
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
fn host_read(key: &str) -> Option<Vec<u8>> {
    HOST_STATE
        .get()
        .and_then(|lock| lock.lock().ok().and_then(|map| map.get(key).cloned()))
}

#[cfg(not(target_arch = "wasm32"))]
static HOST_STATE: OnceLock<Mutex<BTreeMap<String, Vec<u8>>>> = OnceLock::new();

#[cfg(test)]
mod tests {
    use super::*;
    use greentic_types::{PROVIDER_EXTENSION_ID, decode_pack_manifest};
    use serde_json::json;
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    fn sample_input() -> EmailInput {
        EmailInput {
            config: ProviderConfig {
                messaging_provider_id: "messaging.email.sendgrid".into(),
                from: Some("noreply@example.com".into()),
                persistence_key_prefix: None,
            },
            event: json!({"to": "user@example.com", "subject": "Hello", "body": "Test"}),
            handler_id: Some("email-main".into()),
            tenant: Some("tenant-a".into()),
            team: Some("team-1".into()),
            correlation_id: Some("corr-123".into()),
            http: None,
            raw: None,
        }
    }

    #[test]
    fn receipt_is_deterministic() {
        let input = sample_input();
        let id1 = stable_receipt_id(&input.event);
        let id2 = stable_receipt_id(&input.event);
        assert_eq!(id1, id2);
    }

    #[test]
    fn ingest_http_writes_state_host_and_envelope() {
        let input = sample_input();
        let out = handle_ingest_http(&input).expect("ingest_http");
        let json: Value = serde_json::from_slice(&out).expect("json");
        let key = json
            .get("state_key")
            .and_then(|v| v.as_str())
            .expect("state_key");
        let stored = host_read(key).expect("stored entry");
        let entry: QueuedEmail = serde_json::from_slice(&stored).expect("queued");
        assert_eq!(entry.messaging_provider_id, "messaging.email.sendgrid");
        assert_eq!(
            json.get("emitted_events")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.get("event_type"))
                .and_then(|v| v.as_str()),
            Some("email.received")
        );
    }

    #[test]
    fn pack_builds_with_provider_extension() {
        let pack_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/packs/events_provider_email_sendgrid");
        let temp = tempfile::tempdir().expect("tempdir");
        let manifest_out = temp.path().join("manifest.cbor");
        let gtpack_out = temp.path().join("pack.gtpack");

        let status = Command::new("greentic-pack")
            .arg("build")
            .arg("--allow-pack-schema")
            .arg("--no-update")
            .arg("--in")
            .arg(&pack_root)
            .arg("--manifest")
            .arg(&manifest_out)
            .arg("--gtpack-out")
            .arg(&gtpack_out)
            .current_dir(&pack_root)
            .status();

        match status {
            Ok(exit) if exit.success() => {}
            Ok(exit) => panic!("greentic-pack exited with {}", exit),
            Err(err) => {
                eprintln!("greentic-pack not available: {err}");
                return;
            }
        }

        let manifest_bytes = fs::read(&manifest_out).expect("manifest bytes");
        let manifest = decode_pack_manifest(&manifest_bytes).expect("decode manifest");
        assert_eq!(
            manifest.pack_id.as_str(),
            "greentic.events.provider.email.sendgrid"
        );
        let ext_entry = manifest
            .extensions
            .as_ref()
            .and_then(|exts| exts.get(PROVIDER_EXTENSION_ID))
            .expect("provider extension present");
        assert_eq!(
            ext_entry.kind.as_str(),
            PROVIDER_EXTENSION_ID,
            "provider extension kind should match canonical ID"
        );
        let inline = manifest
            .provider_extension_inline()
            .expect("provider extension inline payload");
        let entry = inline.providers.first().expect("provider present");
        assert_eq!(entry.provider_type, "events.email.sendgrid");
    }
}
