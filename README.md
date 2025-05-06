# battered

[![crates.io](https://img.shields.io/crates/v/battered?logo=rust)](https://crates.io/crates/battered)

<img height="64" alt="battered Icon" src="https://raw.githubusercontent.com/t4k1t/battered/main/assets/icon/battered-icon.svg" align="left">

Make the most of your laptop's battery life with custom actions and informative desktop notifications.

Written in Rust, `battered` uses minimal system resources.

-----

**Table of Contents**

- [Features](#features)
- [Usage](#usage)
- [Installation](#installation)
- [Configuration](#configuration)
- [Logging](#logging)
- [License](#license)

## Features

- Unlimited custom actions
- Customizable desktop notifications with placeholder values
- Optional action and notification on connecting power supply
- Configurable poll interval

## Usage

First, make sure you've [configured](#configuration) some actions. Then simply run `battered`:

```bash
battered
```

## Installation

`battered` is available on [crates.io](https://crates.io/crates/battered) and can be installed from there:

```bash
cargo install battered
```

If you're using Arch Linux you can also install it from the AUR using your favorite AUR helper (e.g. `paru`):

```bash
paru -Syu battered
```

## Configuration

battered looks for a configuration file in the following places:
1. `$XDG_CONFIG_HOME/battered/config.toml`
2. `$HOME/.config/battered/config.toml`
3. `/.config/battered/config.toml` if `$HOME` is not set

The `summary` and `body` fields of the `[action.notify]` table support optional placeholders which will be replaced with calculated values. The following placeholders are available:

| Placeholder | Description |
| --- | --- |
| `$percentage` | Current battery level in percent |

By default `battered` will monitor the first battery it finds. Use the `serial_number` config value to pick a specific battery instead.
One way to find the serial number is through sysfs. E.g. find the serial number of `BAT0`:
```bash
cat /sys/class/power_supply/BAT0/serial_number
```

Example config:
```toml
interval = 60                        # Battery level check interval in seconds; optional; defaults to 120; integer
serial_number = "31415"              # Serial number of battery; optional; defaults to first battery; string

[[action]]
percentage = 0.25                    # Run action below this threshold; required; decimal
command = "./powersave.sh enable"    # CLI command to run; optional; string
[action.notify]                      # Notification settings; optional; table
summary = "Battery low!"             # Notification summary; required within action.notify table; string
body = "Battery below $percentage%!" # Notification body; optional; string
urgency = "Critical"                 # Notfication urgency; optional; defaults to `Normal`; enum[ Low | Normal | Critical ]
icon = "battery-caution"             # Notification icon; optional; defaults to "battery-discharging"; string
timeout = 0                          # Notification timeout in ms; optional; defaults to desktop default; integer; `0` means no timeout

# There can be as many `action` entries as desired, and order doesn't matter
[[action]]
percentage = 0.95
[action.notify]
summary = "Battery discharging"

# Special action to run after connecting to AC
# Options are the same as for regular actions
[on_ac]
percentage = 0.10                  # Only run if battery level above this threshold; optional; decimal
command = "./powersave.sh disable"
[on_ac.notify]
summary = "Battery charging"
urgency = "Low"
icon = "battery-good-charging"
timeout = 300
```

## Logging

Logging is configured via the `RUST_LOG` env variable. The provided systemd unit example sets the log level to `WARN` by default.

## License

`battered` is distributed under the terms of the [MIT](https://spdx.org/licenses/MIT.html) license.
