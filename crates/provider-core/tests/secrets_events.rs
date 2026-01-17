use provider_core::secrets::{
    SECRET_EVENT_SCHEMA_VERSION, SecretProvider, StaticSecretProvider, resolve_secret,
    secret_delete_event, secret_missing_detected_event, secret_put_event,
    secret_rotate_completed_event, secret_rotate_requested_event,
};
use std::process::Command;
use std::{fs, path::Path};

fn tenant() -> greentic_types::TenantCtx {
    use greentic_types::{EnvId, TenantCtx, TenantId};

    let env = EnvId::try_from("dev").unwrap();
    let tenant = TenantId::try_from("acme").unwrap();
    TenantCtx::new(env, tenant)
}

#[test]
fn secret_events_use_metadata_only() {
    let tenant = tenant();
    let put = secret_put_event("TEST_API_KEY", "tenant", tenant.clone(), "secrets-tests");
    assert_eq!(put.topic, "greentic.secrets.put");
    assert_eq!(
        put.payload.get("schema_version").and_then(|v| v.as_str()),
        Some(SECRET_EVENT_SCHEMA_VERSION)
    );
    assert_eq!(
        put.payload.get("key").and_then(|v| v.as_str()),
        Some("TEST_API_KEY")
    );
    assert!(
        put.payload.get("value").is_none(),
        "payload must not contain secret bytes"
    );

    let delete = secret_delete_event(
        "TEST_API_KEY",
        "tenant",
        tenant.clone(),
        "secrets-tests",
        "success",
    );
    assert_eq!(delete.topic, "greentic.secrets.delete");

    let rotate = secret_rotate_requested_event(
        "TEST_API_KEY",
        "tenant",
        "rotation-123",
        "requested",
        tenant.clone(),
        "secrets-tests",
        None,
    );
    assert_eq!(rotate.topic, "greentic.secrets.rotate.requested");
    assert!(rotate.payload.get("error").is_none());

    let rotate_completed = secret_rotate_completed_event(
        "TEST_API_KEY",
        "tenant",
        "rotation-123",
        "failed",
        tenant.clone(),
        "secrets-tests",
        Some("network"),
    );
    assert_eq!(rotate_completed.topic, "greentic.secrets.rotate.completed");
    assert_eq!(
        rotate_completed
            .payload
            .get("error")
            .and_then(|v| v.as_str()),
        Some("network")
    );

    let missing = secret_missing_detected_event(
        "TEST_API_KEY",
        "tenant",
        tenant,
        "events-provider/secrets-smoke",
        "resolve TEST_API_KEY",
        "secrets-tests",
    );
    assert_eq!(missing.topic, "greentic.secrets.missing.detected");
    assert!(
        missing.payload.get("detected_by").is_some(),
        "missing payload must include detector metadata"
    );
}

#[test]
fn fixture_pack_exposes_secret_requirement() {
    let pack_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/packs/secrets_events_smoke");
    if !pack_root.exists() {
        panic!("fixture pack missing at {}", pack_root.display());
    }

    let temp = tempfile::tempdir().expect("tempdir");
    let manifest_out = temp.path().join("manifest.cbor");
    let status = Command::new("greentic-pack")
        .arg("build")
        .arg("--no-update")
        .arg("--in")
        .arg(&pack_root)
        .arg("--manifest")
        .arg(&manifest_out)
        .current_dir(&pack_root)
        .status();

    match status {
        Ok(exit) if exit.success() => {}
        Ok(exit) => {
            panic!("greentic-pack exited with {}", exit);
        }
        Err(err) => {
            eprintln!("greentic-pack not available: {err}");
            return;
        }
    }

    let manifest_bytes = fs::read(&manifest_out).expect("manifest bytes");
    let manifest = greentic_types::decode_pack_manifest(&manifest_bytes).expect("decode manifest");
    let requirement_keys: Vec<_> = manifest
        .secret_requirements
        .iter()
        .map(|req| req.key.as_str().to_string())
        .collect();
    assert!(
        requirement_keys.contains(&"TEST_API_KEY".to_string()),
        "expected TEST_API_KEY requirement in pack metadata"
    );
}

#[test]
fn static_secret_provider_resolves_and_rejects_missing() {
    use std::collections::BTreeMap;

    let provider = StaticSecretProvider::new(BTreeMap::from([(
        "TEST_API_KEY".into(),
        b"present".to_vec(),
    )]));
    let found = provider
        .get_secret("TEST_API_KEY")
        .expect("static provider should resolve");
    assert_eq!(found, Some(b"present".to_vec()));

    let missing = provider
        .get_secret("MISSING_KEY")
        .expect("static provider should return Ok(None)");
    assert!(missing.is_none());

    let resolution = resolve_secret(
        &provider,
        "TEST_API_KEY",
        "tenant",
        tenant(),
        "secrets-tests",
        "probe",
    )
    .expect("resolve");
    assert_eq!(resolution.events[0].topic, "greentic.secrets.put");

    let missing_resolution = resolve_secret(
        &provider,
        "MISSING_KEY",
        "tenant",
        tenant(),
        "secrets-tests",
        "probe",
    )
    .expect("resolve missing");
    assert_eq!(
        missing_resolution.events[0].topic,
        "greentic.secrets.missing.detected"
    );

    let rotation_req = secret_rotate_requested_event(
        "TEST_API_KEY",
        "tenant",
        "rot-1",
        "requested",
        tenant(),
        "secrets-tests",
        None,
    );
    assert_eq!(rotation_req.topic, "greentic.secrets.rotate.requested");

    let rotation_done = secret_rotate_completed_event(
        "TEST_API_KEY",
        "tenant",
        "rot-1",
        "success",
        tenant(),
        "secrets-tests",
        None,
    );
    assert_eq!(rotation_done.topic, "greentic.secrets.rotate.completed");

    let delete_evt = secret_delete_event(
        "TEST_API_KEY",
        "tenant",
        tenant(),
        "secrets-tests",
        "success",
    );
    assert_eq!(delete_evt.topic, "greentic.secrets.delete");
}
