mod config;
mod template;

#[macro_use]
extern crate log;
extern crate starship_battery;
use anyhow::{Context, Result};
use config::{xdg_config_home, Action, Config};
use notify_rust::{Notification, Urgency};
use starship_battery::State;
use template::{FormatObject, Template};

use std::path::PathBuf;
use std::process::Command;
use std::thread;

trait CommandRunner {
    fn run(&mut self) -> Result<()>;
    fn below_threshold(&self, value: f32) -> bool;
}

impl CommandRunner for Action {
    fn run(&mut self) -> Result<()> {
        let command = self.command.as_ref();
        match command {
            Some(cmd) => {
                let status = Command::new(&cmd[0])
                    .args(&cmd[1..])
                    .status()
                    .with_context(|| format!("Failed to execute '{}'", cmd.join(" ")))?;
                if !status.success() {
                    return Err(anyhow::anyhow!("Command failed: {}", status));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    fn below_threshold(&self, value: f32) -> bool {
        value < self.percentage
    }
}

trait DesktopNotification {
    fn show(&mut self, format_obj: &FormatObject);
    fn has_notify(&self) -> bool;
    fn fill_template<T: Template>(&self, format_obj: &T) -> String;
}

impl DesktopNotification for Action {
    fn show(&mut self, format_obj: &FormatObject) {
        if let Some(n) = &self.notify {
            let templated_summary = &self.fill_template(format_obj);
            Notification::new()
                .summary(templated_summary)
                .body(n.body.as_ref().unwrap_or(&"".to_string()).as_str())
                .icon(n.icon.as_str())
                .urgency(n.urgency)
                .timeout(n.timeout)
                .show()
                .ok();
        }
    }

    fn has_notify(&self) -> bool {
        self.notify.is_some()
    }

    fn fill_template<T: Template>(&self, format_obj: &T) -> String {
        if let Some(n) = &self.notify {
            let mut result = n.summary.clone();
            let format_string = format_obj.to_template();

            // Replace template vars with templated values from FormatObject
            for line in format_string.lines() {
                let parts: Vec<&str> = line.split(": ").collect();
                if parts.len() == 2 {
                    let placeholder = format!("${}", parts[0]);
                    result = result.replace(&placeholder, parts[1]);
                }
            }
            result
        } else {
            String::from("")
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();

    // Config
    let config_path = xdg_config_home().join("battered/config.toml");
    let config = get_config(&config_path).with_context(|| "Failed to read config")?;
    let mut actions = config.action;
    actions.sort_by(|a, b| {
        a.percentage
            .partial_cmp(&b.percentage)
            .expect("Failed to sort actions by percentage")
    }); // Sort by percentage

    // Set up battery
    let manager = starship_battery::Manager::new()?;
    let mut first_battery = manager
        .batteries()?
        .next()
        .with_context(|| "Failed to access battery information")??;

    // Check and act on battery levels
    let mut last_action_index: usize = usize::MAX;
    loop {
        manager.refresh(&mut first_battery)?;
        let charge_value = first_battery.state_of_charge().value;
        let state = first_battery.state();
        info!("Charge: {:.2}", charge_value);
        info!("State:  {}", state);

        if state == State::Charging {
            last_action_index = usize::MAX; // Reset state
            thread::sleep(config.interval);
            continue; // If the battery is charging there is nothing to do
        }
        match_actions(&mut actions, charge_value, last_action_index).with_context(|| "Failed")?;
        thread::sleep(config.interval);
    }
}

fn match_actions<T: CommandRunner + DesktopNotification>(
    actions: &mut [T],
    charge_value: f32,
    last_action_index: usize,
) -> Result<(), anyhow::Error> {
    for (i, action) in (actions).iter_mut().enumerate() {
        if action.below_threshold(charge_value) {
            if i == last_action_index {
                break; // Action was already taken last iteration, nothing else to do
            }
            let percentage = (charge_value * 100.0).floor();
            let format_obj = FormatObject {
                percentage: &percentage,
            };
            match trigger_action(action, &format_obj) {
                Ok(_) => (),
                Err(e) => {
                    // Show notification about failed action
                    Notification::new()
                        .summary("Battered action failed")
                        .body(e.to_string().as_str())
                        .urgency(Urgency::Critical)
                        .show()
                        .ok();
                    return Err(e);
                }
            };
            break;
        };
    }
    Ok(())
}

fn trigger_action<A: CommandRunner + DesktopNotification>(
    action: &mut A,
    format_obj: &FormatObject,
) -> Result<()> {
    if action.has_notify() {
        action.show(format_obj); // Show notification
    }
    action.run() // Run command
}

fn get_config(config_path: &PathBuf) -> Result<Config, anyhow::Error> {
    let config_values = match std::fs::read_to_string(config_path) {
        Ok(config_values) => config_values,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                warn!(
                    "Config file not found at '{}'; falling back to defaults",
                    config_path.display()
                );
                String::new()
            } else {
                return Err(anyhow::Error::from(e));
            }
        }
    };
    let config: Config = toml::from_str(&config_values)
        .with_context(|| format!("Failed to parse config at '{}'", config_path.display()))?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::Notify;
    use notify_rust::Timeout;

    #[derive(Copy, Clone)]
    struct MockNotify {}

    #[derive(Copy, Clone)]
    struct MockAction {
        show_call_count: usize,
        run_call_count: usize,
        notify: Option<MockNotify>,
        percentage: f32,
    }

    impl DesktopNotification for MockAction {
        fn show(&mut self, _format_obj: &FormatObject) {
            self.show_call_count += 1;
        }
        fn has_notify(&self) -> bool {
            self.notify.is_some()
        }
        fn fill_template<T: Template>(&self, _format_obj: &T) -> String {
            String::from("")
        }
    }

    impl CommandRunner for MockAction {
        fn run(&mut self) -> Result<()> {
            self.run_call_count += 1;
            Ok(())
        }
        fn below_threshold(&self, value: f32) -> bool {
            value < self.percentage
        }
    }

    #[test]
    fn test_has_notify() {
        let action_w_notify = Action {
            percentage: 0.5,
            command: None,
            notify: Some(Notify {
                summary: String::from(""),
                body: None,
                urgency: Urgency::Low,
                icon: String::from(""),
                timeout: Timeout::Default,
            }),
        };
        let has_notify = action_w_notify.has_notify();
        assert_eq!(has_notify, true);
    }

    #[test]
    fn test_has_no_notify() {
        let action_w_notify = Action {
            percentage: 0.5,
            command: None,
            notify: None,
        };
        let has_notify = action_w_notify.has_notify();
        assert_eq!(has_notify, false);
    }

    #[test]
    fn test_handle_threshold_without_notification() {
        let mut action = MockAction {
            show_call_count: 0,
            run_call_count: 0,
            percentage: 0.5,
            notify: None,
        };
        let format_obj = FormatObject { percentage: &50.0 };
        let result = trigger_action(&mut action, &format_obj);
        assert!(result.is_ok());
        assert_eq!(action.show_call_count, 0);
        assert_eq!(action.run_call_count, 1);
    }

    #[test]
    fn test_handle_threshold_with_notification() {
        let mock_notify = MockNotify {};
        let mut action = MockAction {
            run_call_count: 0,
            show_call_count: 0,
            percentage: 0.5,
            notify: Some(mock_notify),
        };

        let format_obj = FormatObject { percentage: &50.0 };
        let result = trigger_action(&mut action, &format_obj);
        assert!(result.is_ok());
        assert_eq!(action.show_call_count, 1);
        assert_eq!(action.run_call_count, 1);
    }

    #[test]
    fn test_threshold_action_above_threshold() {
        let mock_notify = MockNotify {};
        let action = MockAction {
            run_call_count: 0,
            show_call_count: 0,
            percentage: 0.5,
            notify: Some(mock_notify),
        };
        let charge_value = 0.7; // Value above percentage threshold

        let mut actions = vec![action];
        let result = match_actions(&mut actions, charge_value, 0);
        assert!(result.is_ok());
        assert_eq!(action.show_call_count, 0);
        assert_eq!(action.run_call_count, 0);
    }

    #[test]
    fn test_threshold_action_below_threshold() {
        let action = MockAction {
            run_call_count: 0,
            show_call_count: 0,
            percentage: 0.5,
            notify: None,
        };
        let charge_value = 0.3; // Value below percentage threshold

        let mut actions = vec![action]; // Creates a copy
        let result = match_actions(&mut actions, charge_value, usize::MAX);

        let result_action = actions[0];
        assert!(result.is_ok());
        assert_eq!(result_action.run_call_count, 1);
    }

    #[test]
    fn test_template_replaces_percentage() {
        let action_w_notify = Action {
            percentage: 0.5,
            command: None,
            notify: Some(Notify {
                summary: String::from("Percentage is $percentage%!"),
                body: None,
                urgency: Urgency::Low,
                icon: String::from(""),
                timeout: Timeout::Default,
            }),
        };
        let format_obj = FormatObject { percentage: &42.0 };
        let result = action_w_notify.fill_template(&format_obj);
        assert_eq!(result, "Percentage is 42%!");
    }

    #[test]
    fn test_template_replaces_nothing() {
        let action_w_notify = Action {
            percentage: 0.5,
            command: None,
            notify: Some(Notify {
                summary: String::from("No percentage to replace here!"),
                body: None,
                urgency: Urgency::Low,
                icon: String::from(""),
                timeout: Timeout::Default,
            }),
        };
        let format_obj = FormatObject { percentage: &42.0 };
        let result = action_w_notify.fill_template(&format_obj);
        assert_eq!(result, "No percentage to replace here!");
    }

    #[test]
    fn test_template_does_not_replace_unknown() {
        let action_w_notify = Action {
            percentage: 0.5,
            command: None,
            notify: Some(Notify {
                summary: String::from("No $value to replace here!"),
                body: None,
                urgency: Urgency::Low,
                icon: String::from(""),
                timeout: Timeout::Default,
            }),
        };
        let format_obj = FormatObject { percentage: &42.0 };
        let result = action_w_notify.fill_template(&format_obj);
        assert_eq!(result, "No $value to replace here!");
    }

    #[test]
    fn test_get_config_from_invalid_path() {
        let result = get_config(&PathBuf::from("/dev/null"));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Failed to parse config at '/dev/null'"
        );
    }
}
