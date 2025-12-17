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
fn timer_pack_parses() {
    let raw = fs::read_to_string(Path::new("../../packs/events/timer.yaml")).unwrap();
    let pack: Pack = serde_yaml_bw::from_str(&raw).unwrap();
    assert_eq!(pack.events.providers.len(), 1);
    assert_eq!(pack.pack_id, "greentic.events.timer");
    assert_eq!(pack.components.len(), 1);

    let provider = &pack.events.providers[0];
    assert_eq!(provider.component, "events-timer-source@1.0.0");
    assert_eq!(provider.capabilities.transport, "timer");
    assert!(
        provider
            .capabilities
            .topics
            .contains(&"timer.*".to_string())
    );
}
