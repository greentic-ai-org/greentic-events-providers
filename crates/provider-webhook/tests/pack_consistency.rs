use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Pack {
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

#[test]
fn webhook_pack_parses_and_references_flows() {
    let raw = fs::read_to_string(Path::new("../../packs/events/webhook.yaml")).unwrap();
    let pack: Pack = serde_yaml_bw::from_str(&raw).unwrap();
    assert!(!pack.events.providers.is_empty());

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
