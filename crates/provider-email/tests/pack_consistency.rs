use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Pack {
    pack_id: String,
    components: Vec<Component>,
    events: Events,
}

#[derive(Debug, Deserialize)]
struct Events {
    providers: Vec<Provider>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Provider {
    name: String,
    kind: String,
    component: String,
    capabilities: Capabilities,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Capabilities {
    transport: String,
    reliability: String,
    topics: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Component {
    id: String,
    version: String,
    capabilities: ComponentCaps,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct ComponentCaps {
    #[serde(default)]
    host: Option<HostCaps>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct HostCaps {
    #[serde(default)]
    secrets: Option<SecretsCaps>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct SecretsCaps {
    #[serde(default)]
    required: Vec<SecretRequirement>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SecretRequirement {
    key: String,
    required: bool,
}

#[test]
fn email_pack_parses() {
    let raw = fs::read_to_string(Path::new("../../packs/events-email/pack.yaml")).unwrap();
    let pack: Pack = serde_yaml_bw::from_str(&raw).unwrap();
    assert_eq!(pack.pack_id, "greentic.events.email");
    assert_eq!(pack.events.providers.len(), 4);
    let email_components = pack
        .components
        .iter()
        .filter(|c| c.id.starts_with("events-email-"))
        .count();
    assert_eq!(email_components, 2);

    let required_keys: Vec<_> = pack
        .components
        .iter()
        .flat_map(|c| {
            c.capabilities
                .host
                .as_ref()
                .and_then(|h| h.secrets.as_ref())
                .map(|s| s.required.iter().map(|r| r.key.clone()).collect::<Vec<_>>())
                .unwrap_or_default()
        })
        .collect();

    for key in [
        "MSGRAPH_CLIENT_SECRET",
        "GMAIL_CLIENT_SECRET",
        "GMAIL_REFRESH_TOKEN",
    ] {
        assert!(
            required_keys.contains(&key.to_string()),
            "missing secret requirement {key}"
        );
    }

    for provider in pack.events.providers {
        assert!(
            provider.component == "events-email-source@1.0.0"
                || provider.component == "events-email-sink@1.0.0"
        );
        assert!(!provider.capabilities.topics.is_empty());
        assert!(
            provider.capabilities.transport == "msgraph"
                || provider.capabilities.transport == "gmail"
        );
    }
}
