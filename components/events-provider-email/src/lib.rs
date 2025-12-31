#![deny(unsafe_op_in_unsafe_fn)]

use anyhow::{Context, Result};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProviderDescribe {
    provider_type: String,
    capabilities: Value,
    ops: Vec<String>,
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
struct PublishInput {
    config: ProviderConfig,
    #[serde(default)]
    event: Value,
}

#[allow(dead_code)]
struct Component;

impl provider_core::Guest for Component {
    fn describe() -> Vec<u8> {
        let describe = ProviderDescribe {
            provider_type: "events.email".into(),
            capabilities: json!({
                "operations": ["publish"],
                "persistence": "state-store",
                "deterministic": true,
            }),
            ops: vec!["publish".into()],
        };
        serde_json::to_vec(&describe).unwrap_or_default()
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
    let parsed: PublishInput = serde_json::from_slice(input_json)
        .with_context(|| "publish input must include config and event")?;
    match op {
        "publish" => handle_publish(&parsed),
        other => anyhow::bail!("unsupported op {other}"),
    }
}

#[allow(dead_code)]
fn handle_publish(input: &PublishInput) -> Result<Vec<u8>> {
    if input.config.messaging_provider_id.trim().is_empty() {
        anyhow::bail!("messaging_provider_id is required");
    }
    let receipt_id = stable_receipt_id(&input.event);
    let key = state_key(&input.config, &receipt_id);
    persist_request(&key, input)?;

    Ok(json!({
        "receipt_id": receipt_id,
        "status": "queued",
        "state_key": key,
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
        .unwrap_or("events/email/queued");
    format!("{prefix}/{receipt_id}.json")
}

fn stable_receipt_id(event: &Value) -> String {
    let bytes = serde_json::to_vec(event).unwrap_or_default();
    Uuid::new_v5(&Uuid::NAMESPACE_OID, &bytes).to_string()
}

fn persist_request(key: &str, input: &PublishInput) -> Result<()> {
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
    use greentic_types::decode_pack_manifest;
    use serde_json::json;
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    fn sample_input() -> PublishInput {
        PublishInput {
            config: ProviderConfig {
                messaging_provider_id: "messaging.email.provider".into(),
                from: Some("noreply@example.com".into()),
                persistence_key_prefix: None,
            },
            event: json!({"to": "user@example.com", "subject": "Hello", "body": "Test"}),
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
    fn publish_writes_state_host() {
        let input = sample_input();
        let out = handle_publish(&input).expect("publish");
        let json: Value = serde_json::from_slice(&out).expect("json");
        let key = json
            .get("state_key")
            .and_then(|v| v.as_str())
            .expect("state_key");
        let stored = host_read(key).expect("stored entry");
        let entry: QueuedEmail = serde_json::from_slice(&stored).expect("queued");
        assert_eq!(entry.messaging_provider_id, "messaging.email.provider");
    }

    #[test]
    fn pack_builds_with_provider_extension() {
        let pack_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/packs/events_provider_email");
        let temp = tempfile::tempdir().expect("tempdir");
        let manifest_out = temp.path().join("manifest.cbor");
        let gtpack_out = temp.path().join("pack.gtpack");

        let status = Command::new("packc")
            .arg("build")
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
            Ok(exit) => panic!("packc exited with {}", exit),
            Err(err) => {
                eprintln!("packc not available: {err}");
                return;
            }
        }

        let manifest_bytes = fs::read(&manifest_out).expect("manifest bytes");
        let manifest = decode_pack_manifest(&manifest_bytes).expect("decode manifest");
        assert_eq!(manifest.pack_id.as_str(), "greentic.events.provider.email");
        let ext = manifest
            .extensions
            .as_ref()
            .and_then(|exts| exts.get("greentic.ext.provider"))
            .and_then(|ext| ext.inline.as_ref())
            .cloned()
            .expect("provider extension inline payload");
        let providers = ext
            .get("providers")
            .and_then(|v| v.as_array())
            .expect("providers array");
        let entry = providers.first().expect("provider present");
        assert_eq!(
            entry.get("provider_type").and_then(|v| v.as_str()),
            Some("events.email")
        );
    }
}
