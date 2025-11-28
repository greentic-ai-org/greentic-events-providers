use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Shared HTTP endpoint configuration for webhook-style providers.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpEndpointConfig {
    /// Base path where the host exposes the webhook (e.g. "/webhook").
    pub base_path: String,
    /// Individual routes served under the base path.
    pub routes: Vec<WebhookRoute>,
}

/// Per-route configuration for webhook providers.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebhookRoute {
    /// Route path relative to base (e.g. "/stripe").
    pub path: String,
    /// Optional reference to a secret containing a signing key.
    pub secret_ref: Option<String>,
    /// Topic prefix to apply when emitting events (e.g. "webhook.stripe").
    pub topic_prefix: String,
}

/// Timer/scheduler configuration shared by timer providers.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SchedulerConfig {
    pub schedules: Vec<Schedule>,
}

/// Definition of a single schedule.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Schedule {
    /// Logical name of the schedule (e.g. "daily-report").
    pub name: String,
    /// Cron or interval expression supplied by deployer/host.
    pub cron: String,
    /// Topic to emit when the schedule fires (e.g. "timer.daily.report").
    pub topic: String,
    /// JSON payload that will be emitted with the event.
    pub payload: Value,
}
