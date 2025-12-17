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
    default_flow: Option<String>,
    custom_flow: Option<String>,
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
fn webhook_pack_parses_and_references_flows() {
    let raw = fs::read_to_string(Path::new("../../packs/events/webhook.yaml")).unwrap();
    let pack: Pack = serde_yaml_bw::from_str(&raw).unwrap();
    assert_eq!(pack.pack_id, "greentic.events.webhook");
    assert!(!pack.events.providers.is_empty());

    assert!(
        pack.components
            .iter()
            .any(|comp| comp.id == "events-webhook-source")
    );
    let secret_keys: Vec<_> = pack
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
    assert!(
        secret_keys.contains(&"WEBHOOK_SIGNING_SECRET".to_string()),
        "expected webhook signing secret requirement"
    );

    for provider in pack.events.providers {
        assert_eq!(provider.component, "events-webhook-source@1.0.0");
        if let Some(flow) = provider.default_flow {
            assert!(Path::new("../../").join(&flow).exists());
        }
        if let Some(flow) = provider.custom_flow {
            assert!(Path::new("../../").join(&flow).exists());
        }
        assert_eq!(provider.capabilities.transport, "webhook");
        assert!(!provider.capabilities.topics.is_empty());
    }
}
