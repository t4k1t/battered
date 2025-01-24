# battery-notify

Regularly polls battery levels and sends notifications on crossing certain thresholds.

The idea is to have one notification to warn about the battery discharging and another, persistent, notification when action has to be taken. Both thresholds and the poll interval can be configured.

## Config

battery-notify looks for a configuration file in the following places:
1. `$XDG_CONFIG_HOME/batterynotify/config.toml`
2. `$HOME/.config/batterynotify/config.toml`
3. `/.config/batterynotify/config.toml` if `$HOME` is not set

Example config:
```
[general]
interval = 60              # in seconds
threshold_low = 0.8        # percentage as decimal
threshold_critical = 0.25  # percentage as decimal
action_low = 'tuned-adm profile laptop-battery-powersave'
action_critical = 'systemctl suspend'
```

## Logging

Logging is configured via the `RUST_LOG` env variable. The provided systemd unit example sets the log level to `WARN` per default.
