use serde::Deserialize;
use serde_with::{serde_as, DurationSeconds};
use std::time::Duration;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_general_config")]
    pub general: GeneralConfig,
}

#[serde_as]
#[derive(Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_threshold_low")]
    pub threshold_low: f32,
    #[serde(default = "default_threshold_critical")]
    pub threshold_critical: f32,
    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "default_interval")]
    pub interval: Duration,
    #[serde(default = "default_action")]
    pub action_low: Option<String>,
    #[serde(default = "default_action")]
    pub action_critical: Option<String>,
}

fn default_threshold_low() -> f32 {
    0.8
}

fn default_threshold_critical() -> f32 {
    0.25
}

fn default_interval() -> Duration {
    Duration::from_secs(60)
}

fn default_action() -> Option<String> {
    None
}

fn default_general_config() -> GeneralConfig {
    GeneralConfig {
        threshold_low: default_threshold_low(),
        threshold_critical: default_threshold_critical(),
        interval: default_interval(),
        action_low: None,
        action_critical: None,
    }
}
