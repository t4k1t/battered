mod config;

#[macro_use]
extern crate log;
extern crate starship_battery;
use config::Config;
use notify_rust::{Notification, Timeout, Urgency};
use starship_battery::State;

use std::path::PathBuf;
use std::process::Command;
use std::thread;

#[derive(Debug, Eq, PartialEq)]
enum Level {
    Charged,
    Low,
    Critical,
}

fn get_config(config_path: &PathBuf) -> Config {
    let config_values = match std::fs::read_to_string(&config_path) {
        Ok(config_values) => config_values,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                warn!(
                    "Config file not found at '{}'; falling back to defaults",
                    config_path.display()
                );
                String::new()
            } else {
                panic!(
                    "Failed to read config at '{}'; {}",
                    config_path.display(),
                    e
                );
            }
        }
    };
    toml::from_str(&config_values)
        .expect(format!("Failed to parse config at '{}'", config_path.display()).as_str())
}

fn main() -> starship_battery::Result<()> {
    env_logger::init();
    let config_path = xdg_config_home().join("battered/config.toml");
    let config = get_config(&config_path);

    let action_low = &config.general.action_low.unwrap_or_default();
    let action_critical = &config.general.action_critical.unwrap_or_default();

    let manager = starship_battery::Manager::new()?;
    let mut first_battery = match manager.batteries()?.next() {
        Some(Ok(first_battery)) => first_battery,
        Some(Err(e)) => {
            panic!("Unable to access battery information: {}", e);
        }
        _ => {
            panic!("Unable to find any batteries");
        }
    };
    let mut level = Level::Charged;

    loop {
        let charge_value = first_battery.state_of_charge().value;
        let state = first_battery.state();
        info!("Charge: {:.2}", charge_value);
        info!("State:  {}", state);
        if state != State::Charging && charge_value < config.general.threshold_critical {
            if level != Level::Critical {
                level = Level::Critical;
                let mut notification = Notification::new();
                // Send critical level notification
                notification
                    .summary("Battery low!")
                    .body(format!("Battery below {}%", (charge_value * 100.0).trunc()).as_str())
                    .icon("battery-caution")
                    .urgency(Urgency::Critical)
                    .timeout(Timeout::Never);
                notification.show().ok();
                // Run critical level custom action
                if !action_critical.is_empty() {
                    Command::new(&action_critical[0])
                        .args(&action_critical[1..])
                        .status()
                        .unwrap_or_else(|error_code| {
                            panic!(
                                "Failed to execute '{}': {}",
                                action_critical.join(" "),
                                error_code
                            )
                        });
                };
            };
        } else if state != State::Charging && charge_value < config.general.threshold_low {
            if level != Level::Low {
                level = Level::Low;
                let mut notification = Notification::new();
                // Send low level notification
                notification
                    .summary("Battery discharging")
                    .body(format!("Battery below {}%", (charge_value * 100.0).trunc()).as_str())
                    .icon("battery-low")
                    .urgency(Urgency::Normal);
                notification.show().ok();
                // Run low level custom action
                if !action_low.is_empty() {
                    Command::new(&action_low[0])
                        .args(&action_low[1..])
                        .status()
                        .unwrap_or_else(|error_code| {
                            panic!(
                                "Failed to execute '{}': {}",
                                action_low.join(" "),
                                error_code
                            )
                        });
                };
            };
        } else {
            level = Level::Charged;
        };
        thread::sleep(config.general.interval);
        manager.refresh(&mut first_battery)?;
    }
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
