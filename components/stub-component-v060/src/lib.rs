use std::collections::BTreeMap;

use serde::Serialize;

wit_bindgen::generate!({
    path: "wit",
    world: "component-v0-v6-v0",
});

const COMPONENT_ID: &str = match option_env!("STUB_COMPONENT_ID") {
    Some(value) => value,
    None => "stub",
};
const COMPONENT_VERSION: &str = match option_env!("STUB_COMPONENT_VERSION") {
    Some(value) => value,
    None => "0.1.0",
};

#[derive(Serialize)]
struct ComponentInfo {
    id: String,
    version: String,
    role: String,
    display_name: Option<String>,
}

#[derive(Serialize)]
struct SchemaIr {
    r#type: String,
}

#[derive(Serialize)]
struct ComponentDescribe {
    info: ComponentInfo,
    provided_capabilities: Vec<String>,
    required_capabilities: Vec<String>,
    metadata: BTreeMap<String, serde_json::Value>,
    operations: Vec<serde_json::Value>,
    config_schema: SchemaIr,
}

#[derive(Serialize)]
struct I18nText {
    key: String,
    fallback: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum QaModeDoc {
    Default,
    Setup,
    Upgrade,
    Remove,
}

#[derive(Serialize)]
struct ComponentQaSpecDoc {
    mode: QaModeDoc,
    title: I18nText,
    description: Option<I18nText>,
    questions: Vec<serde_json::Value>,
    defaults: BTreeMap<String, serde_json::Value>,
}

fn describe_cbor() -> Vec<u8> {
    let describe = ComponentDescribe {
        info: ComponentInfo {
            id: COMPONENT_ID.to_string(),
            version: COMPONENT_VERSION.to_string(),
            role: "component".to_string(),
            display_name: None,
        },
        provided_capabilities: Vec::new(),
        required_capabilities: Vec::new(),
        metadata: BTreeMap::new(),
        operations: Vec::new(),
        config_schema: SchemaIr {
            r#type: "null".to_string(),
        },
    };
    let mut out = Vec::new();
    ciborium::into_writer(&describe, &mut out).expect("encode describe");
    out
}

fn schema_cbor() -> Vec<u8> {
    let schema = SchemaIr {
        r#type: "null".to_string(),
    };
    let mut out = Vec::new();
    ciborium::into_writer(&schema, &mut out).expect("encode schema");
    out
}

fn qa_spec_cbor(mode: QaModeDoc) -> Vec<u8> {
    let spec = ComponentQaSpecDoc {
        mode,
        title: I18nText {
            key: format!("{}.qa.title", COMPONENT_ID),
            fallback: Some("Stub component configuration".to_string()),
        },
        description: Some(I18nText {
            key: format!("{}.qa.description", COMPONENT_ID),
            fallback: Some("No configuration required for this placeholder component.".to_string()),
        }),
        questions: Vec::new(),
        defaults: BTreeMap::new(),
    };
    let mut out = Vec::new();
    ciborium::into_writer(&spec, &mut out).expect("encode qa spec");
    out
}

struct StubComponent;

impl exports::greentic::component::component_descriptor::Guest for StubComponent {
    fn get_component_info() -> Vec<u8> {
        Vec::new()
    }

    fn describe() -> Vec<u8> {
        describe_cbor()
    }
}

impl exports::greentic::component::component_schema::Guest for StubComponent {
    fn input_schema() -> Vec<u8> {
        schema_cbor()
    }

    fn output_schema() -> Vec<u8> {
        schema_cbor()
    }

    fn config_schema() -> Vec<u8> {
        schema_cbor()
    }
}

impl exports::greentic::component::component_runtime::Guest for StubComponent {
    fn run(
        _input: Vec<u8>,
        state: Vec<u8>,
    ) -> exports::greentic::component::component_runtime::RunResult {
        exports::greentic::component::component_runtime::RunResult {
            output: Vec::new(),
            new_state: state,
        }
    }
}

impl exports::greentic::component::component_qa::Guest for StubComponent {
    fn qa_spec(mode: exports::greentic::component::component_qa::QaMode) -> Vec<u8> {
        let mode_doc = match mode {
            exports::greentic::component::component_qa::QaMode::Default => QaModeDoc::Default,
            exports::greentic::component::component_qa::QaMode::Setup => QaModeDoc::Setup,
            exports::greentic::component::component_qa::QaMode::Upgrade => QaModeDoc::Upgrade,
            exports::greentic::component::component_qa::QaMode::Remove => QaModeDoc::Remove,
        };
        qa_spec_cbor(mode_doc)
    }

    fn apply_answers(
        _mode: exports::greentic::component::component_qa::QaMode,
        current_config: Vec<u8>,
        _answers: Vec<u8>,
    ) -> Vec<u8> {
        current_config
    }
}

impl exports::greentic::component::component_i18n::Guest for StubComponent {
    fn i18n_keys() -> Vec<String> {
        vec![
            format!("{}.qa.title", COMPONENT_ID),
            format!("{}.qa.description", COMPONENT_ID),
        ]
    }
}

export!(StubComponent);
