mod config;
mod template;

#[macro_use]
extern crate log;
extern crate starship_battery;
use anyhow::{Context, Result};
use config::{xdg_config_home, Action, Config, OnAcAction};
use notify_rust::{Notification, Urgency};
use starship_battery::{Batteries, Battery, State};
use template::{FormatObject, Template};

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::thread;

trait CommandRunner {
    fn run(&mut self) -> Result<()>;
    fn exceeds_threshold(&self, value: &f32) -> bool;
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

    fn exceeds_threshold(&self, value: &f32) -> bool {
        value < &self.percentage
    }
}

impl CommandRunner for OnAcAction {
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

    fn exceeds_threshold(&self, value: &f32) -> bool {
        value >= &self.percentage
    }
}

trait DesktopNotification {
    fn show(&mut self, format_obj: &FormatObject);
    fn has_notify(&self) -> bool;
    fn fill_template<T: Template>(&self, input_string: String, format_obj: &T) -> String;
}

impl DesktopNotification for Action {
    fn show(&mut self, format_obj: &FormatObject) {
        if let Some(n) = &self.notify {
            let templated_summary = &self.fill_template(n.summary.clone(), format_obj);
            let mut body = n.body.clone().unwrap_or(String::from(""));
            body = self.fill_template(body, format_obj);
            Notification::new()
                .summary(templated_summary)
                .body(body.as_str())
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

    fn fill_template<T: Template>(&self, input_string: String, format_obj: &T) -> String {
        let mut result = input_string;
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
    }
}

impl DesktopNotification for OnAcAction {
    fn show(&mut self, format_obj: &FormatObject) {
        if let Some(n) = &self.notify {
            let templated_summary = &self.fill_template(n.summary.clone(), format_obj);
            let mut body = n.body.clone().unwrap_or(String::from(""));
            body = self.fill_template(body, format_obj);
            Notification::new()
                .summary(templated_summary)
                .body(body.as_str())
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

    fn fill_template<T: Template>(&self, input_string: String, format_obj: &T) -> String {
        let mut result = input_string;
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
    }
}

fn get_version_from_env() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn main() -> Result<()> {
    env_logger::init();

    // Handle CLI args
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-V") {
        println!("battered {}", get_version_from_env());
        return Ok(());
    }

    // Config
    let config_path = xdg_config_home().join("battered/config.toml");
    let config = get_config(&config_path).with_context(|| "Failed to read config")?;
    let mut actions = config.action;
    actions.sort_by(|a, b| {
        a.percentage
            .partial_cmp(&b.percentage)
            .expect("Failed to sort actions by percentage")
    }); // Sort by percentage

    // Set up battery manager
    let manager = starship_battery::Manager::new()?;
    let mut batteries = manager.batteries()?;
    debug!("Looking for serial number: {:?}", config.serial_number);
    let mut battery = pick_battery(&mut batteries, config.serial_number.as_deref())?;

    // Check and act on battery levels
    let mut last_action_index: usize = usize::MAX;
    loop {
        manager.refresh(&mut battery)?;
        let charge_value = battery.state_of_charge().value;
        let percentage = (charge_value * 100.0).floor();
        let state = battery.state();
        let mut on_ac = config.on_ac.clone();
        info!("Charge: {:.2}", charge_value);
        info!("State:  {}", state);

        let format_obj = FormatObject {
            percentage: &percentage,
        };
        if state == State::Charging {
            if last_action_index != usize::MAX {
                last_action_index = usize::MAX; // Reset state
                if let Some(on_ac) = &mut on_ac {
                    match trigger_action(on_ac, &format_obj) {
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
                }
            }
            thread::sleep(config.interval);
            continue; // If the battery is charging there is nothing else to do
        }
        match_actions(
            &mut actions,
            &charge_value,
            &mut last_action_index,
            &format_obj,
        )
        .with_context(|| "Failed")?;
        thread::sleep(config.interval);
    }
}

fn pick_battery(
    batteries: &mut Batteries,
    serial_number: Option<&str>,
) -> Result<Battery, anyhow::Error> {
    let mut selected_battery: Option<Battery> = None;
    match serial_number {
        Some(serial) => {
            for battery in batteries {
                let battery_ref = battery.with_context(|| "Failed to access battery")?;
                let battery_serial_number = battery_ref
                    .serial_number()
                    .with_context(|| "Failed to get serial number from battery")?
                    .trim();
                if battery_serial_number == serial {
                    selected_battery = Some(battery_ref);
                    break;
                }
            }
            match selected_battery {
                Some(battery) => Ok(battery),
                None => Err(anyhow::Error::msg(format!(
                    "Failed to find battery with serial number '{}'",
                    serial
                ))),
            }
        }
        None => Ok(batteries
            .next()
            .with_context(|| "Failed to access battery information")??),
    }
}

fn match_actions<T: CommandRunner + DesktopNotification>(
    actions: &mut [T],
    charge_value: &f32,
    last_action_index: &mut usize,
    format_obj: &FormatObject,
) -> Result<(), anyhow::Error> {
    for (i, action) in (actions).iter_mut().enumerate() {
        if action.exceeds_threshold(charge_value) {
            if i == *last_action_index {
                break; // Action was already taken last iteration, nothing else to do
            }
            *last_action_index = i;
            match trigger_action(action, format_obj) {
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
        fn fill_template<T: Template>(&self, _input_string: String, _format_obj: &T) -> String {
            String::from("")
        }
    }

    impl CommandRunner for MockAction {
        fn run(&mut self) -> Result<()> {
            self.run_call_count += 1;
            Ok(())
        }
        fn exceeds_threshold(&self, value: &f32) -> bool {
            value < &self.percentage
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
    fn test_threshold_without_notification() {
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
    fn test_threshold_with_notification() {
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
    fn test_threshold_below_threshold_fn() {
        let action = Action {
            percentage: 0.5,
            command: None,
            notify: None,
        };
        let charge_value_below = 0.3; // Value below percentage threshold
        let charge_value_above = 0.8; // Value below percentage threshold

        let below_result = action.exceeds_threshold(&charge_value_below);
        assert_eq!(below_result, true);

        let above_result = action.exceeds_threshold(&charge_value_above);
        assert_eq!(above_result, false);
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
        let mut last_action_index: usize = 0;
        let format_obj = FormatObject { percentage: &70.0 };
        let result = match_actions(
            &mut actions,
            &charge_value,
            &mut last_action_index,
            &format_obj,
        );
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
        let mut last_action_index = usize::MAX;
        let format_obj = FormatObject { percentage: &30.0 };
        let result = match_actions(
            &mut actions,
            &charge_value,
            &mut last_action_index,
            &format_obj,
        );

        let result_action = actions[0];
        assert!(result.is_ok());
        assert_eq!(result_action.run_call_count, 1);
    }

    #[test]
    fn test_successful_action() {
        let mut action = Action {
            percentage: 0.5,
            notify: None,
            command: Some(vec![String::from("true")]),
        };
        let format_obj = FormatObject { percentage: &50.0 };
        let result = trigger_action(&mut action, &format_obj);
        assert!(result.is_ok());
    }

    #[test]
    fn test_no_action() {
        let mut action = Action {
            percentage: 0.5,
            notify: None,
            command: None,
        };
        let format_obj = FormatObject { percentage: &50.0 };
        let result = trigger_action(&mut action, &format_obj);
        assert!(result.is_ok());
    }

    #[test]
    fn test_failing_action() {
        let mut action = Action {
            percentage: 0.5,
            notify: None,
            command: Some(vec![String::from("false")]),
        };
        let format_obj = FormatObject { percentage: &50.0 };
        let result = trigger_action(&mut action, &format_obj);
        assert!(result.is_err());
    }

    #[test]
    fn test_successful_on_ac_action() {
        let mut action = OnAcAction {
            percentage: 0.0,
            notify: None,
            command: Some(vec![String::from("true")]),
        };
        let format_obj = FormatObject { percentage: &50.0 };
        let result = trigger_action(&mut action, &format_obj);
        assert!(result.is_ok());
    }

    #[test]
    fn test_no_on_ac_action() {
        let mut action = OnAcAction {
            percentage: 0.1,
            notify: None,
            command: None,
        };
        let format_obj = FormatObject { percentage: &50.0 };
        let result = trigger_action(&mut action, &format_obj);
        assert!(result.is_ok());
    }

    #[test]
    fn test_failing_on_ac_action() {
        let mut action = OnAcAction {
            percentage: 0.0,
            notify: None,
            command: Some(vec![String::from("false")]),
        };
        let format_obj = FormatObject { percentage: &50.0 };
        let result = trigger_action(&mut action, &format_obj);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_replaces_percentage() {
        let summary = String::from("Percentage is $percentage%!");
        let body = String::from("$percentage is also in the body");
        let action_w_notify = Action {
            percentage: 0.5,
            command: None,
            notify: Some(Notify {
                summary: summary.clone(),
                body: Some(body.clone()),
                urgency: Urgency::Low,
                icon: String::from(""),
                timeout: Timeout::Default,
            }),
        };
        let format_obj = FormatObject { percentage: &42.0 };
        let summary_result = action_w_notify.fill_template(summary, &format_obj);
        assert_eq!(summary_result, "Percentage is 42%!");
        let body_result = action_w_notify.fill_template(body, &format_obj);
        assert_eq!(body_result, "42 is also in the body");
    }

    #[test]
    fn test_template_replaces_percentage_for_on_ac_action() {
        let summary = String::from("Percentage is $percentage%!");
        let body = String::from("$percentage is also in the body");
        let action_w_notify = OnAcAction {
            percentage: 0.21,
            command: None,
            notify: Some(Notify {
                summary: summary.clone(),
                body: Some(body.clone()),
                urgency: Urgency::Low,
                icon: String::from(""),
                timeout: Timeout::Default,
            }),
        };
        let format_obj = FormatObject { percentage: &42.0 };
        let summary_result = action_w_notify.fill_template(summary, &format_obj);
        assert_eq!(summary_result, "Percentage is 42%!");
        let body_result = action_w_notify.fill_template(body, &format_obj);
        assert_eq!(body_result, "42 is also in the body");
    }

    #[test]
    fn test_template_replaces_nothing() {
        let summary = String::from("No percentage to replace here!");
        let action_w_notify = Action {
            percentage: 0.5,
            command: None,
            notify: Some(Notify {
                summary: summary.clone(),
                body: None,
                urgency: Urgency::Low,
                icon: String::from(""),
                timeout: Timeout::Default,
            }),
        };
        let format_obj = FormatObject { percentage: &42.0 };
        let result = action_w_notify.fill_template(summary, &format_obj);
        assert_eq!(result, "No percentage to replace here!");
    }

    #[test]
    fn test_template_does_not_replace_unknown() {
        let summary = String::from("No $value to replace here!");
        let action_w_notify = Action {
            percentage: 0.5,
            command: None,
            notify: Some(Notify {
                summary: summary.clone(),
                body: None,
                urgency: Urgency::Low,
                icon: String::from(""),
                timeout: Timeout::Default,
            }),
        };
        let format_obj = FormatObject { percentage: &42.0 };
        let result = action_w_notify.fill_template(summary, &format_obj);
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

    #[test]
    fn test_pick_battery_by_serial_not_found() {
        let manager = starship_battery::Manager::new().unwrap();
        let mut batteries = manager.batteries().unwrap();
        let result = pick_battery(&mut batteries, Some("not-a-serial-number"));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Failed to find battery with serial number 'not-a-serial-number'"
        );
    }
}
