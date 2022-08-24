use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
}

#[derive(Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_threshold_low")]
    pub threshold_low: f32,
    #[serde(default = "default_threshold_critical")]
    pub threshold_critical: f32,
    #[serde(default = "default_interval")]
    pub interval: u64,
}

fn default_threshold_low() -> f32 {
    0.8
}

fn default_threshold_critical() -> f32 {
    0.25
}

fn default_interval() -> u64 {
    60
}
