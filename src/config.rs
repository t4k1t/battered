use notify_rust::{Timeout, Urgency};
use serde::de::Error as SerdeError;
use serde::{Deserialize, Deserializer};
use serde_with::{serde_as, DurationSeconds};
use shell_words::split as shell_split;
use std::path::PathBuf;
use std::time::Duration;

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "default_interval")]
    pub interval: Duration,
    pub action: Vec<Action>,
}

#[derive(Debug, Deserialize)]
pub struct Action {
    #[serde(deserialize_with = "deserialize_float_percentage")]
    pub percentage: f32,
    #[serde(default, deserialize_with = "deserialize_command")]
    pub command: Option<Vec<String>>,
    pub notify: Option<Notify>,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct Notify {
    pub summary: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default = "default_urgency", deserialize_with = "deserialize_urgency")]
    pub urgency: Urgency,
    #[serde(default = "default_icon")]
    pub icon: String,
    #[serde(default, deserialize_with = "deserialize_timeout")]
    pub timeout: Timeout,
}

fn default_urgency() -> Urgency {
    Urgency::Normal
}

fn default_icon() -> String {
    "battery-caution".to_string()
}

fn default_interval() -> Duration {
    Duration::from_secs(60)
}

fn deserialize_float_percentage<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Deserialize float
    let value: f32 = Deserialize::deserialize(deserializer)?;
    // Check valid range
    if !(0.0..=1.0).contains(&value) {
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
fn deserialize_urgency<'de, D>(deserializer: D) -> Result<Urgency, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize the string
    let value: String = String::deserialize(deserializer)?;
    // Attempt to parse the notification urgency
    match Urgency::try_from(value.as_str()) {
        Ok(urgency) => Ok(urgency),
        Err(e) => Err(D::Error::custom(format!(
            "Failed to parse notification urgency: {}",
            e
        ))),
    }
}

fn deserialize_timeout<'de, D>(deserializer: D) -> Result<Timeout, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize the integer
    let value: i32 = i32::deserialize(deserializer)?;
    Ok(Timeout::from(value))
}

// Taken from i3status-rust
pub fn xdg_config_home() -> PathBuf {
    // In the unlikely event that $HOME is not set, it doesn't really matter
    // what we fall back on, so use /.config.
    let config_path = std::env::var("XDG_CONFIG_HOME").unwrap_or(format!(
        "{}/.config",
        std::env::var("HOME").unwrap_or_else(|_| "".to_string())
    ));
    PathBuf::from(&config_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;
    use toml;

    static ENV_VAR_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_valid_config() {
        let toml_str = r#"
        interval = 120

        [[action]]
        percentage = 0.84
        command = "./powersave.sh profile laptop-battery-powersave"
        [action.notify]
        summary = "Battery discharging"
        urgency = "Low"
        icon = "battery-discharging"
        timeout = 10000
        action_critical = "./powersave.sh suspend"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.interval, Duration::from_secs(120));
        assert_eq!(config.action[0].percentage, 0.84);
        assert_eq!(
            config.action[0].command,
            Some(vec![
                "./powersave.sh".to_string(),
                "profile".to_string(),
                "laptop-battery-powersave".to_string()
            ])
        );
        let notify = config.action[0].notify.as_ref().unwrap();
        assert_eq!(notify.summary, "Battery discharging");
        assert_eq!(notify.urgency, Urgency::Low);
        assert_eq!(notify.icon, "battery-discharging");
        assert_eq!(notify.timeout, Timeout::Milliseconds(10000));
    }

    #[test]
    fn test_default_values() {
        let toml_str = r#"

        # At least one actions is required
        [[action]]
        percentage = 0.99
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.interval, Duration::from_secs(60));
    }

    #[test]
    fn test_default_timeout_never_value() {
        let toml_str = r#"

        # At least one actions is required
        [[action]]
        percentage = 0.99
        [action.notify]
        summary = "Never Gonna Give You Up"
        timeout = 0
        "#; // `0` means no timeout

        let config: Config = toml::from_str(toml_str).unwrap();
        let notify = config.action[0].notify.as_ref().unwrap();
        assert_eq!(notify.timeout, Timeout::Never);
    }

    #[test]
    fn test_missing_actions() {
        let toml_str = r#"
        "#;

        let result: Result<Config, toml::de::Error> = toml::from_str(toml_str);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message(), "missing field `action`");
    }

    #[test]
    fn test_invalid_interval_type() {
        let toml_str = r#"
        # Interval has to be valid Duration
        interval = "not_a_number"

        [[action]]
        percentage = 0.99
        "#;

        let result: Result<Config, toml::de::Error> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_percentage_values() {
        let test_values = vec![
            r#"
            [[action]]
            percentage = -0.2
            "#,
            r#"
            [[action]]
            percentage = 42
            "#,
            r#"
            [[action]]
            percentage = "0.5"
            "#,
        ]; // Thresholds have to be positive floating point numbers between 0 and 1

        for (i, value) in test_values.iter().enumerate() {
            let result: Result<Config, toml::de::Error> = toml::from_str(value);
            assert!(result.is_err());
            if i == 0 || i == 1 {
                assert_eq!(
                    result.unwrap_err().message(),
                    "value must be between 0 and 1"
                );
            } else {
                assert!(result.unwrap_err().message().starts_with("invalid type"));
            };
        }
    }

    #[test]
    fn test_invalid_urgency_values() {
        let string_test_values = vec![
            r#"
            [[action]]
            percentage = 0.9
            [action.notify]
            urgency = ""
            "#,
            r#"
            [[action]]
            percentage = 0.9
            [action.notify]
            urgency = "Whatever"
            "#,
        ];
        let other_test_values = vec![
            r#"
            [[action]]
            percentage = 0.9
            [action.notify]
            urgency = 1
            "#,
            r#"
            [[action]]
            percentage = 0.9
            [action.notify]
            urgency = ["Critical", "Normal"]
            "#,
        ];

        for value in string_test_values {
            let result: Result<Config, toml::de::Error> = toml::from_str(value);
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .message()
                .starts_with("Failed to parse notification urgency"));
        }
        for value in other_test_values {
            let result: Result<Config, toml::de::Error> = toml::from_str(value);
            assert!(result.is_err());
            assert!(result.unwrap_err().message().starts_with("invalid type"));
        }
    }

    #[test]
    fn test_invalid_timeout_values() {
        let string_test_values = vec![
            r#"
            [[action]]
            percentage = 0.9
            [action.notify]
            timeout = "500"
            "#,
            r#"
            [[action]]
            percentage = 0.9
            [action.notify]
            # Timeout has to be i32
            timeout = 600.0
            "#,
        ];

        for value in string_test_values {
            let result: Result<Config, toml::de::Error> = toml::from_str(value);
            assert!(result.is_err());
            assert!(result.unwrap_err().message().starts_with("invalid type"));
        }

        let toml_str = r#"
        # Interval has to be valid Duration
        [[action]]
        percentage = 0.99
        [action.notify]
        summary = ""
        timeout = -5
        "#; // Negative timeout is silently converted to `Timeout::Default` by notify-rust
        let config: Config = toml::from_str(toml_str).unwrap();
        let notify = config.action[0].notify.as_ref().unwrap();
        assert_eq!(notify.timeout, Timeout::Default);
    }

    #[test]
    fn test_unparsable_command_value() {
        let toml_str = r#"
        [[action]]
        percentage = 0.99
        command = "notify-send 'Oops, I am missing my closing single quote!"
        "#; // Unbalanced quotes cannot be parsed correctly

        let result: Result<Config, toml::de::Error> = toml::from_str(toml_str);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message(),
            "Failed to split command: missing closing quote"
        );
    }

    #[test]
    fn test_xdg_config_home() {
        let _lock = ENV_VAR_MUTEX.lock().unwrap();
        env::set_var("XDG_CONFIG_HOME", "/home/battered/.config");
        let config_home = xdg_config_home();
        assert_eq!(config_home, PathBuf::from("/home/battered/.config"));
    }

    #[test]
    fn test_xdg_config_home_from_home_var() {
        let _lock = ENV_VAR_MUTEX.lock().unwrap();
        env::remove_var("XDG_CONFIG_HOME");
        env::set_var("HOME", "/home/battered");
        let config_home = xdg_config_home();
        assert_eq!(config_home, PathBuf::from("/home/battered/.config"));
    }

    #[test]
    fn test_xdg_config_home_from_nothing() {
        let _lock = ENV_VAR_MUTEX.lock().unwrap();
        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("HOME");
        let config_home = xdg_config_home();
        assert_eq!(config_home, PathBuf::from("/.config"));
    }
}
