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
fn timer_pack_parses() {
    let raw = fs::read_to_string(Path::new("../../packs/events/timer.yaml")).unwrap();
    let pack: Pack = serde_yaml_bw::from_str(&raw).unwrap();
    assert_eq!(pack.events.providers.len(), 1);

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
