use serde_json::Value;
use std::fs;
use std::path::Path;

fn read_pack_json(pack: &str) -> Value {
    let path = Path::new("../../packs").join(pack).join("pack.json");
    let raw = fs::read_to_string(&path).expect("pack.json exists");
    serde_json::from_str(&raw).expect("pack.json is valid json")
}

fn setup_entry_from_meta(meta: &Value) -> Option<String> {
    let entry_flows = meta.get("entry_flows")?;
    if let Some(map) = entry_flows.as_object() {
        return map
            .get("setup")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
    }
    if let Some(list) = entry_flows.as_array() {
        for item in list {
            let entry = item
                .get("entry")
                .or_else(|| item.get("name"))
                .and_then(|v| v.as_str());
            if entry == Some("setup") {
                return item
                    .get("id")
                    .or_else(|| item.get("flow_id"))
                    .or_else(|| item.get("name"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
        }
    }
    None
}

#[test]
fn provisioning_metadata_present() {
    let packs = [
        "events-email",
        "events-sms",
        "events-webhook",
        "events-timer",
    ];

    for pack in packs {
        let value = read_pack_json(pack);
        let meta = value.get("meta").expect("meta present");
        let setup = setup_entry_from_meta(meta);
        assert!(setup.is_some(), "{pack} should declare setup entry flow");

        let requirements = Path::new("../../packs")
            .join(pack)
            .join("fixtures")
            .join("requirements.expected.json");
        assert!(
            requirements.exists(),
            "{pack} should have requirements.expected.json"
        );

        let requirements_wat = Path::new("../../packs")
            .join(pack)
            .join("setup_default__requirements.wat");
        assert!(
            requirements_wat.exists(),
            "{pack} should include setup_default__requirements.wat"
        );

        let subscriptions_fixture = Path::new("../../packs")
            .join(pack)
            .join("fixtures")
            .join("subscriptions.expected.json");
        if subscriptions_fixture.exists() {
            let subscriptions = meta
                .get("entry_flows")
                .and_then(|entry| entry.as_object())
                .and_then(|map| map.get("subscriptions"))
                .and_then(|v| v.as_str());
            assert!(
                subscriptions.is_some(),
                "{pack} should declare subscriptions entry flow"
            );

            let subscriptions_wat = Path::new("../../packs")
                .join(pack)
                .join("setup_default__subscriptions.wat");
            assert!(
                subscriptions_wat.exists(),
                "{pack} should include setup_default__subscriptions.wat"
            );
        }
    }
}

#[test]
fn dummy_pack_has_no_setup() {
    let value = read_pack_json("events-dummy");
    let meta = value.get("meta").expect("meta present");
    let setup = setup_entry_from_meta(meta);
    assert!(setup.is_none(), "events-dummy should not declare setup");

    let caps = meta
        .get("capabilities")
        .and_then(|v| v.as_array())
        .expect("capabilities present");
    assert!(
        caps.iter()
            .any(|cap| cap.as_str() == Some("provisioning:none")),
        "events-dummy should declare provisioning:none"
    );
}
