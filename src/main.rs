mod config;

extern crate battery;
use battery::State;
use config::Config;
use notify_rust::{Notification, Timeout, Urgency};

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

fn main() -> battery::Result<()> {
    let config_path = xdg_config_home().join("batterynotify/config.toml");
    let config: Config = toml::from_str(&std::fs::read_to_string(config_path).unwrap()).unwrap();
    let interval = Duration::from_secs(config.general.interval);
    let threshold_low = config.general.threshold_low;
    let threshold_critical = config.general.threshold_critical;

    let manager = battery::Manager::new()?;
    let mut my_battery = match manager.batteries()?.next() {
        Some(Ok(my_battery)) => my_battery,
        Some(Err(e)) => {
            eprintln!("Unable to access battery information");
            return Err(e);
        }
        None => {
            eprintln!("Unable to find any batteries");
            return Err(io::Error::from(io::ErrorKind::NotFound).into());
        }
    };
    let mut level = Level::Charged;

    loop {
        let charge = my_battery.state_of_charge();
        let state = my_battery.state();
        println!("{:?}", charge);
        println!("{:?}", state);
        if state != State::Charging && charge.value < threshold_critical {
            if level != Level::Critical {
                level = Level::Critical;
                Notification::new()
                    .summary("Battery low!")
                    .body(format!("Battery percentage down to {:04}%", charge.value).as_str())
                    .icon("battery-caution")
                    .urgency(Urgency::Critical)
                    .timeout(Timeout::Never)
                    .show()
                    .unwrap();
            };
        } else if state != State::Charging && charge.value < threshold_low {
            if level != Level::Low {
                level = Level::Low;
                Notification::new()
                    .summary("Battery discharging")
                    .body(format!("Battery percentage down to {:04}%", charge.value).as_str())
                    .icon("battery-low")
                    .show()
                    .unwrap();
            };
        } else {
            level = Level::Charged;
        };
        thread::sleep(interval);
        manager.refresh(&mut my_battery)?;
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
