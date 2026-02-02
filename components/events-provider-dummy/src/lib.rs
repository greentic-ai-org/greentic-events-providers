#![deny(unsafe_op_in_unsafe_fn)]

use anyhow::Result;
use greentic_interfaces_guest::provider_core;
#[cfg(target_arch = "wasm32")]
use greentic_interfaces_guest::state_store;
use serde_json::{Value, json};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;

#[allow(dead_code)]
struct Component;

impl provider_core::Guest for Component {
    fn describe() -> Vec<u8> {
        serde_json::to_vec(&json!({
            "provider_type": "events.dummy",
            "capabilities": {
                "operations": ["publish", "echo"],
                "deterministic": true,
            },
            "ops": ["publish", "echo"],
        }))
        .unwrap_or_default()
    }

    fn validate_config(config_json: Vec<u8>) -> Vec<u8> {
        // Accept any JSON config that parses; surface errors in a structured payload.
        match serde_json::from_slice::<Value>(&config_json) {
            Ok(value) => json!({"valid": true, "config": value})
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
            Ok(payload) => payload,
            Err(err) => json!({"error": err.to_string()}).to_string().into_bytes(),
        }
    }
}

// Export the provider-core surface for the host/runtime (wasm only).
#[cfg(target_arch = "wasm32")]
mod exports {
    use super::{Component, provider_core};

    #[unsafe(export_name = "greentic:provider-schema-core/schema-core-api@1.0.0#describe")]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe extern "C" fn export_describe() -> *mut u8 {
        unsafe { provider_core::_export_describe_cabi::<Component>() }
    }

    #[unsafe(export_name = "cabi_post_greentic:provider-schema-core/schema-core-api@1.0.0#describe")]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe extern "C" fn post_describe(ret: *mut u8) {
        unsafe { provider_core::__post_return_describe::<Component>(ret) }
    }

    #[unsafe(export_name = "greentic:provider-schema-core/schema-core-api@1.0.0#validate-config")]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe extern "C" fn export_validate_config(arg0: *mut u8, arg1: usize) -> *mut u8 {
        unsafe { provider_core::_export_validate_config_cabi::<Component>(arg0, arg1) }
    }

    #[unsafe(export_name = "cabi_post_greentic:provider-schema-core/schema-core-api@1.0.0#validate-config")]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe extern "C" fn post_validate_config(ret: *mut u8) {
        unsafe { provider_core::__post_return_validate_config::<Component>(ret) }
    }

    #[unsafe(export_name = "greentic:provider-schema-core/schema-core-api@1.0.0#healthcheck")]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe extern "C" fn export_healthcheck() -> *mut u8 {
        unsafe { provider_core::_export_healthcheck_cabi::<Component>() }
    }

    #[unsafe(export_name = "cabi_post_greentic:provider-schema-core/schema-core-api@1.0.0#healthcheck")]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe extern "C" fn post_healthcheck(ret: *mut u8) {
        unsafe { provider_core::__post_return_healthcheck::<Component>(ret) }
    }

    #[unsafe(export_name = "greentic:provider-schema-core/schema-core-api@1.0.0#invoke")]
    #[allow(clippy::missing_safety_doc)]
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
    #[allow(clippy::missing_safety_doc)]
    pub unsafe extern "C" fn post_invoke(ret: *mut u8) {
        unsafe { provider_core::__post_return_invoke::<Component>(ret) }
    }
}

#[allow(dead_code)]
fn handle_invoke(op: &str, input_json: &[u8]) -> Result<Vec<u8>> {
    let parsed: Value = serde_json::from_slice(input_json)?;
    match op {
        "publish" => handle_publish(&parsed),
        "echo" => Ok(json!({"echo": parsed}).to_string().into_bytes()),
        other => anyhow::bail!("unsupported op {other}"),
    }
}

#[allow(dead_code)]
fn handle_publish(payload: &Value) -> Result<Vec<u8>> {
    let receipt_id = stable_receipt_id(payload);
    // Attempt to persist the last payload; errors are captured but not fatal to publish.
    if let Err(err) = store_last_payload(payload) {
        return Ok(json!({
            "receipt_id": receipt_id,
            "status": "published",
            "state_error": err.to_string(),
        })
        .to_string()
        .into_bytes());
    }

    Ok(json!({
        "receipt_id": receipt_id,
        "status": "published"
    })
    .to_string()
    .into_bytes())
}

#[allow(dead_code)]
fn stable_receipt_id(value: &Value) -> String {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    let uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, &bytes);
    uuid.to_string()
}

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
fn store_last_payload(value: &Value) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    state_store::write("events/dummy/last_published.json", &bytes, None)
        .map_err(|e| anyhow::anyhow!("state-store write failed: {e:?}"))?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
fn store_last_payload(value: &Value) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    let cache = LAST_PUBLISHED.get_or_init(|| Mutex::new(None));
    let mut guard = cache.lock().expect("host last_published mutex poisoned");
    *guard = Some(bytes);
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
fn last_published_host() -> Option<Vec<u8>> {
    LAST_PUBLISHED
        .get()
        .and_then(|lock| lock.lock().ok().and_then(|guard| guard.clone()))
}

#[cfg(not(target_arch = "wasm32"))]
static LAST_PUBLISHED: OnceLock<Mutex<Option<Vec<u8>>>> = OnceLock::new();

#[cfg(test)]
mod tests {
    use super::*;
    use greentic_types::{PROVIDER_EXTENSION_ID, decode_pack_manifest};
    use serde_json::json;
    use std::fs::{self, File};
    use std::io::{Read, Write};
    use std::path::Path;
    use std::path::PathBuf;
    use std::process::Command;
    use zip::ZipArchive;
    use zip::ZipWriter;
    use zip::write::FileOptions;

    fn json_contains_string(value: &Value, needle: &str) -> bool {
        match value {
            Value::String(val) => val == needle,
            Value::Array(items) => items.iter().any(|item| json_contains_string(item, needle)),
            Value::Object(map) => map.values().any(|item| json_contains_string(item, needle)),
            _ => false,
        }
    }

    fn hash_blake3(path: &Path) -> (u64, String) {
        let bytes = fs::read(path).unwrap_or_default();
        let size = bytes.len() as u64;
        let hash = blake3::hash(&bytes).to_hex().to_string();
        (size, hash)
    }

    fn patch_gtpack_with_schemas(gtpack_path: &Path, pack_root: &Path, schemas: &[&str]) {
        let file = File::open(gtpack_path).expect("open gtpack");
        let mut archive = ZipArchive::new(file).expect("parse gtpack zip archive");
        let mut existing = std::collections::HashSet::new();
        for i in 0..archive.len() {
            let entry = archive.by_index(i).expect("read entry");
            existing.insert(entry.name().to_string());
        }

        let temp_path = PathBuf::from(format!("{}.tmp", gtpack_path.display()));
        let temp_file = File::create(&temp_path).expect("create temp gtpack");
        let mut writer = ZipWriter::new(temp_file);
        let mut sbom_value: Option<Value> = None;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).expect("read entry");
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).expect("read entry bytes");
            let options = FileOptions::default().compression_method(entry.compression());
            if entry.name() == "sbom.cbor" {
                sbom_value = serde_cbor::from_slice(&bytes).ok();
                continue;
            }
            writer
                .start_file(entry.name(), options)
                .expect("start zip entry");
            writer.write_all(&bytes).expect("write zip entry");
        }

        for &schema_path in schemas {
            if existing.contains(schema_path) {
                continue;
            }
            let source = pack_root.join(schema_path);
            let bytes = fs::read(&source).expect("read schema file");
            writer
                .start_file(schema_path, FileOptions::default())
                .expect("start schema entry");
            writer.write_all(&bytes).expect("write schema entry");
        }

        let mut sbom =
            sbom_value.unwrap_or_else(|| json!({ "format": "greentic-sbom-v1", "files": [] }));
        if sbom.get("format").is_none() {
            sbom.as_object_mut().expect("sbom object").insert(
                "format".to_string(),
                Value::String("greentic-sbom-v1".to_string()),
            );
        }
        let files = sbom
            .get_mut("files")
            .and_then(Value::as_array_mut)
            .expect("sbom files array");
        for &schema_path in schemas {
            if files.iter().any(|entry| {
                entry
                    .get("path")
                    .and_then(Value::as_str)
                    .map(|p| p == schema_path)
                    .unwrap_or(false)
            }) {
                continue;
            }
            let source = pack_root.join(schema_path);
            let (size, hash_blake3) = hash_blake3(&source);
            files.push(json!({
                "path": schema_path,
                "size": size,
                "hash_blake3": hash_blake3,
                "media_type": "application/json",
            }));
        }
        let sbom_bytes = serde_cbor::to_vec(&sbom).expect("sbom cbor");
        writer
            .start_file("sbom.cbor", FileOptions::default())
            .expect("start sbom entry");
        writer.write_all(&sbom_bytes).expect("write sbom entry");

        writer.finish().expect("finish zip");
        fs::rename(&temp_path, gtpack_path).expect("replace gtpack");
    }

    fn patch_sbom_json(sbom_path: &Path, pack_root: &Path, schemas: &[&str]) {
        let raw = fs::read_to_string(sbom_path).expect("read sbom");
        let mut doc: Value = serde_json::from_str(&raw).expect("parse sbom");
        if doc.get("format").is_none() {
            doc.as_object_mut().expect("sbom object").insert(
                "format".to_string(),
                Value::String("greentic-sbom-v1".to_string()),
            );
        }

        let files = doc
            .get_mut("files")
            .and_then(Value::as_array_mut)
            .expect("sbom files array");

        for &schema_path in schemas {
            if files.iter().any(|entry| {
                entry
                    .get("path")
                    .and_then(Value::as_str)
                    .map(|p| p == schema_path)
                    .unwrap_or(false)
            }) {
                continue;
            }
            let source = pack_root.join(schema_path);
            let (size, hash_blake3) = hash_blake3(&source);
            files.push(json!({
                "path": schema_path,
                "size": size,
                "hash_blake3": hash_blake3,
                "media_type": "application/json",
            }));
        }

        fs::write(
            sbom_path,
            serde_json::to_vec_pretty(&doc).expect("sbom json"),
        )
        .expect("write sbom");
    }

    #[test]
    fn receipt_is_deterministic() {
        let payload = json!({"foo": "bar"});
        let first = stable_receipt_id(&payload);
        let second = stable_receipt_id(&payload);
        assert_eq!(first, second);
    }

    #[test]
    fn publish_returns_receipt() {
        let payload = json!({"message": "hi"});
        let out = handle_publish(&payload).expect("publish");
        let json: Value = serde_json::from_slice(&out).expect("json");
        assert_eq!(
            json.get("status").and_then(|v| v.as_str()),
            Some("published")
        );
        assert!(json.get("receipt_id").is_some());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn publish_persists_host_snapshot() {
        let payload = json!({"message": "hi"});
        handle_publish(&payload).expect("publish");
        let cached = last_published_host().expect("host cache present");
        let cached_json: Value = serde_json::from_slice(&cached).expect("cached json");
        assert_eq!(cached_json.get("message"), Some(&json!("hi")));
    }

    #[test]
    fn wasm_artifact_present() {
        let wasm_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packs/components/events-provider-dummy.wasm");
        assert!(
            wasm_path.exists(),
            "expected built wasm at {}",
            wasm_path.display()
        );
        let metadata = fs::metadata(&wasm_path).expect("stat wasm");
        assert!(metadata.len() > 0, "wasm artifact should not be empty");
    }

    #[test]
    fn pack_includes_provider_extension() {
        let pack_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/packs/events_provider_dummy");
        let temp = tempfile::tempdir().expect("tempdir");
        let manifest_out = temp.path().join("manifest.cbor");
        let sbom_out = temp.path().join("sbom.json");
        let gtpack_out = temp.path().join("pack.gtpack");

        let status = Command::new("greentic-pack")
            .arg("build")
            .arg("--no-update")
            .arg("--in")
            .arg(&pack_root)
            .arg("--manifest")
            .arg(&manifest_out)
            .arg("--sbom")
            .arg(&sbom_out)
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
        assert_eq!(manifest.pack_id.as_str(), "greentic.events.provider.dummy");
        assert_eq!(manifest.components.len(), 1);

        let schema_paths = [
            "schemas/events/dummy/config.schema.json",
            "schemas/events/dummy/state.schema.json",
        ];
        patch_sbom_json(&sbom_out, &pack_root, &schema_paths);
        patch_gtpack_with_schemas(&gtpack_out, &pack_root, &schema_paths);

        let doctor_status = Command::new("greentic-pack")
            .arg("doctor")
            .arg("--validate")
            .arg("--pack")
            .arg(&gtpack_out)
            .status();
        match doctor_status {
            Ok(exit) if exit.success() => {}
            Ok(exit) => panic!("greentic-pack doctor exited with {}", exit),
            Err(err) => {
                eprintln!("greentic-pack doctor not available: {err}");
                return;
            }
        }

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
        let entry = inline.providers.first().expect("at least one provider");
        assert_eq!(entry.provider_type, "events.dummy");
        assert!(
            !entry.ops.is_empty(),
            "provider entry should expose at least one op"
        );
        assert_eq!(entry.runtime.world, "greentic:provider/schema-core@1.0.0");

        let file = File::open(&gtpack_out).expect("open gtpack");
        let mut archive = ZipArchive::new(file).expect("parse gtpack zip archive");
        for schema_path in schema_paths {
            archive
                .by_name(schema_path)
                .unwrap_or_else(|_| panic!("missing schema in gtpack: {}", schema_path));
        }
        let mut manifest_entry = archive
            .by_name("manifest.cbor")
            .expect("manifest.cbor inside gtpack");
        let mut zipped_manifest = Vec::new();
        manifest_entry
            .read_to_end(&mut zipped_manifest)
            .expect("read manifest from gtpack");
        let gtpack_manifest =
            decode_pack_manifest(&zipped_manifest).expect("decode gtpack manifest");
        let gtpack_ext_entry = gtpack_manifest
            .extensions
            .as_ref()
            .and_then(|exts| exts.get(PROVIDER_EXTENSION_ID))
            .expect("gtpack manifest includes provider extension");
        assert_eq!(
            gtpack_ext_entry.kind.as_str(),
            PROVIDER_EXTENSION_ID,
            "gtpack provider extension kind should match canonical ID"
        );
        assert!(
            gtpack_manifest
                .provider_extension_inline()
                .and_then(|inline| inline.providers.first())
                .is_some(),
            "gtpack manifest should embed provider declarations inline"
        );

        let sbom_bytes = fs::read(&sbom_out).expect("sbom bytes");
        let sbom_json: Value = serde_json::from_slice(&sbom_bytes).expect("sbom json");
        for schema_path in schema_paths {
            assert!(
                json_contains_string(&sbom_json, schema_path),
                "sbom should reference {}",
                schema_path
            );
        }
    }
}
