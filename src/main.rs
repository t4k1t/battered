mod config;

#[macro_use]
extern crate log;
extern crate starship_battery;
use config::Config;
use notify_rust::{Notification, Timeout, Urgency};
use starship_battery::State;

use std::io;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[derive(Debug, Eq, PartialEq)]
enum Level {
    Charged,
    Low,
    Critical,
}

fn main() -> starship_battery::Result<()> {
    let config_path = xdg_config_home().join("batterynotify/config.toml");
    env_logger::init();

    let config_values = match std::fs::read_to_string(&config_path) {
        Ok(config) => config,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                warn!(
                    "Config file not found at '{}'; falling back to defaults",
                    config_path.display()
                );
                "[general]".to_string()
            } else {
                panic!(
                    "Failed to read config at '{}'; {}",
                    config_path.display(),
                    e
                );
            }
        }
    };
    let config: Config = toml::from_str(&config_values).unwrap();
    let interval = Duration::from_secs(config.general.interval);
    let threshold_low = config.general.threshold_low;
    let threshold_critical = config.general.threshold_critical;

    let manager = starship_battery::Manager::new()?;
    let mut first_battery = match manager.batteries()?.next() {
        Some(Ok(first_battery)) => first_battery,
        Some(Err(e)) => {
            error!("Unable to access battery information");
            return Err(e);
        }
        None => {
            error!("Unable to find any batteries");
            return Err(io::Error::from(io::ErrorKind::NotFound).into());
        }
    };
    let mut level = Level::Charged;

    loop {
        let charge = first_battery.state_of_charge();
        let state = first_battery.state();
        info!("Charge: {:?}", charge);
        info!("State:  {:?}", state);
        if state != State::Charging && charge.value < threshold_critical {
            if level != Level::Critical {
                level = Level::Critical;
                let mut notification = Notification::new();
                notification
                    .summary("Battery low!")
                    .body(format!("Battery below {}%", (charge.value * 100.0).trunc()).as_str())
                    .icon("battery-caution")
                    .urgency(Urgency::Critical)
                    .timeout(Timeout::Never);
                notification.show().ok();
            };
        } else if state != State::Charging && charge.value < threshold_low {
            if level != Level::Low {
                level = Level::Low;
                let mut notification = Notification::new();
                notification
                    .summary("Battery discharging")
                    .body(format!("Battery below {}%", (charge.value * 100.0).trunc()).as_str())
                    .icon("battery-low")
                    .urgency(Urgency::Normal);
                notification.show().ok();
            };
        } else {
            level = Level::Charged;
        };
        thread::sleep(interval);
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
