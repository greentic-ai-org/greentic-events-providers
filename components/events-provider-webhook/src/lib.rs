#![deny(unsafe_op_in_unsafe_fn)]

use anyhow::{Context, Result};
#[cfg(target_arch = "wasm32")]
use greentic_interfaces_guest::http_client;
use greentic_interfaces_guest::provider_core;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProviderDescribe {
    provider_type: String,
    capabilities: Value,
    ops: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ProviderConfig {
    target_url: String,
    #[serde(default = "default_method")]
    method: String,
    #[serde(default)]
    headers: BTreeMap<String, String>,
    #[serde(default)]
    auth: Option<String>,
    #[serde(default)]
    timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct PublishInput {
    config: ProviderConfig,
    #[serde(default)]
    event: Value,
}

fn default_method() -> String {
    "POST".into()
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Component;

impl provider_core::Guest for Component {
    fn describe() -> Vec<u8> {
        let describe = ProviderDescribe {
            provider_type: "events.webhook".into(),
            capabilities: json!({
                "operations": ["publish"],
                "transport": "http",
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
    let receipt_id = stable_receipt_id(&input.event);
    let request = build_request(&input.config, &input.event)?;
    let dispatched = dispatch(&request).is_ok();

    Ok(json!({
        "receipt_id": receipt_id,
        "status": if dispatched { "published" } else { "queued" },
        "dispatched": dispatched,
        "request": request,
    })
    .to_string()
    .into_bytes())
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, PartialEq)]
struct OutgoingRequest {
    method: String,
    url: String,
    headers: BTreeMap<String, String>,
    body: Value,
}

#[allow(dead_code)]
fn build_request(config: &ProviderConfig, event: &Value) -> Result<OutgoingRequest> {
    if config.target_url.trim().is_empty() {
        anyhow::bail!("target_url is required");
    }
    let mut headers = config.headers.clone();
    headers
        .entry("content-type".into())
        .or_insert_with(|| "application/json".into());
    if let Some(token) = &config.auth {
        headers
            .entry("authorization".into())
            .or_insert_with(|| format!("Bearer {token}"));
    }

    Ok(OutgoingRequest {
        method: config.method.to_uppercase(),
        url: config.target_url.clone(),
        headers,
        body: json!({ "event": event }),
    })
}

#[allow(dead_code)]
fn stable_receipt_id(event: &Value) -> String {
    let bytes = serde_json::to_vec(event).unwrap_or_default();
    Uuid::new_v5(&Uuid::NAMESPACE_OID, &bytes).to_string()
}

#[allow(dead_code)]
fn dispatch(request: &OutgoingRequest) -> Result<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let headers: Vec<(String, String)> = request
            .headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let body = serde_json::to_vec(&request.body)?;
        let send_req = http_client::Request {
            method: request.method.clone(),
            url: request.url.clone(),
            headers,
            body: Some(body),
        };

        let resp = http_client::send(&send_req, None)
            .map_err(|err| anyhow::anyhow!("http send failed: {err}"))?;
        if resp.status >= 200 && resp.status < 400 {
            return Ok(());
        }
        anyhow::bail!("http send returned {}", resp.status);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Host tests run without HTTP; signal as queued/no-op.
        let _ = request;
        anyhow::bail!("http client not available on host")
    }
}

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
                target_url: "https://example.test/hook".into(),
                method: "POST".into(),
                headers: BTreeMap::from([("x-test".into(), "1".into())]),
                auth: Some("token123".into()),
                timeout_ms: Some(5000),
            },
            event: json!({"id": 1, "kind": "test"}),
        }
    }

    #[test]
    fn builds_request_with_auth() {
        let input = sample_input();
        let req = build_request(&input.config, &input.event).expect("build");
        assert_eq!(req.method, "POST");
        assert_eq!(req.url, input.config.target_url);
        assert_eq!(
            req.headers.get("authorization").map(|s| s.as_str()),
            Some("Bearer token123")
        );
        assert_eq!(
            req.body
                .get("event")
                .and_then(|v| v.get("id"))
                .and_then(|v| v.as_i64()),
            Some(1)
        );
    }

    #[test]
    fn receipt_is_deterministic() {
        let input = sample_input();
        let id1 = stable_receipt_id(&input.event);
        let id2 = stable_receipt_id(&input.event);
        assert_eq!(id1, id2);
    }

    #[test]
    fn handle_publish_returns_payload() {
        let input = sample_input();
        let out = handle_publish(&input).expect("publish");
        let json: Value = serde_json::from_slice(&out).expect("json");
        assert!(json.get("receipt_id").is_some());
        assert_eq!(json.get("status").and_then(|v| v.as_str()), Some("queued"));
        assert_eq!(
            json.get("request")
                .and_then(|r| r.get("url"))
                .and_then(|v| v.as_str()),
            Some("https://example.test/hook")
        );
    }

    #[test]
    fn pack_builds_with_provider_extension() {
        let pack_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/packs/events_provider_webhook");
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
        assert_eq!(
            manifest.pack_id.as_str(),
            "greentic.events.provider.webhook"
        );
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
            Some("events.webhook")
        );
    }
}
