use serde::de::Error as SerdeError;
use serde::{Deserialize, Deserializer};
use serde_with::{serde_as, DurationSeconds};
use shell_words::split as shell_split;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_general_config")]
    pub general: GeneralConfig,
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    #[serde(
        default = "default_threshold_low",
        deserialize_with = "deserialize_float_percentage"
    )]
    pub threshold_low: f32,
    #[serde(
        default = "default_threshold_critical",
        deserialize_with = "deserialize_float_percentage"
    )]
    pub threshold_critical: f32,
    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "default_interval")]
    pub interval: Duration,
    #[serde(default, deserialize_with = "deserialize_command")]
    pub action_low: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_command")]
    pub action_critical: Option<Vec<String>>,
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

fn default_general_config() -> GeneralConfig {
    GeneralConfig {
        threshold_low: default_threshold_low(),
        threshold_critical: default_threshold_critical(),
        interval: default_interval(),
        action_low: None,
        action_critical: None,
    }
}

fn deserialize_float_percentage<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Deserialize float
    let value: f32 = Deserialize::deserialize(deserializer)?;
    // Check valid range
    if value < 0.0 || value > 1.0 {
        return Err(D::Error::custom("value must be between 0 and 1"));
    }
    Ok(value)
}

fn deserialize_command<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize the string
    let value: String = String::deserialize(deserializer)?;
    // Attempt to split the command
    match shell_split(&value) {
        Ok(command) => Ok(Some(command)),
        Err(e) => Err(D::Error::custom(format!("Failed to split command: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml;

    #[test]
    fn test_valid_config() {
        let toml_str = r#"
        [general]
        interval = 120
        threshold_low = 0.9
        threshold_critical = 0.1
        action_low = "./powersave.sh profile laptop-battery-powersave"
        action_critical = "./powersave.sh suspend"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.general.interval, Duration::from_secs(120));
        assert_eq!(config.general.threshold_low, 0.9);
        assert_eq!(config.general.threshold_critical, 0.1);
        assert_eq!(
            config.general.action_low,
            Some(vec![
                "./powersave.sh".to_string(),
                "profile".to_string(),
                "laptop-battery-powersave".to_string()
            ])
        );
        assert_eq!(
            config.general.action_critical,
            Some(vec!["./powersave.sh".to_string(), "suspend".to_string(),])
        );
    }

    #[test]
    fn test_default_values_for_general_section() {
        let toml_str = r#"
        [general]
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.general.interval, Duration::from_secs(60));
        assert_eq!(config.general.threshold_low, 0.8);
        assert_eq!(config.general.threshold_critical, 0.25);
        assert_eq!(config.general.action_low, None);
        assert_eq!(config.general.action_critical, None);
    }

    #[test]
    fn test_empty_config_loads_default_values() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config.general.interval, Duration::from_secs(60));
        assert_eq!(config.general.threshold_low, 0.8);
        assert_eq!(config.general.threshold_critical, 0.25);
    }

    #[test]
    fn test_invalid_interval_type() {
        let toml_str = r#"
        [general]
        interval = "not_a_number"
        "#; // Interval has to be valid Duration

        let result: Result<Config, toml::de::Error> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_threshold_low_type() {
        let toml_str = r#"
        [general]
        threshold_low = -0.2
        "#; // Thresholds have to be positive floating point numbers

        let result: Result<Config, toml::de::Error> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_threshold_low_value() {
        let toml_str = r#"
        [general]
        threshold_low = 3.14
        "#; // Thresholds have to be between 0 and 1

        let result: Result<Config, toml::de::Error> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_threshold_critical_value() {
        let toml_str = r#"
        [general]
        threshold_critical = 1/2
        "#; // Thresholds have to be positive floating point numbers

        let result: Result<Config, toml::de::Error> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_action_low_value() {
        let toml_str = r#"
        [general]
        action_low = 42
        "#; // Invalid action_low value

        let result: Result<Config, toml::de::Error> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_unparsable_action_low_value() {
        let toml_str = r#"
        [general]
        action_low = "notify-send 'Oops, I am missing my closing single quote!"
        "#; // Unbalanced quotes cannot be parsed correctly

        let result: Result<Config, toml::de::Error> = toml::from_str(toml_str);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message(),
            "Failed to split command: missing closing quote"
        );
    }
}
