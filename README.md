# battered

[![crates.io](https://img.shields.io/crates/v/battered?logo=rust)](https://crates.io/crates/battered)

<img height="128" alt="battered Icon" src="https://raw.githubusercontent.com/t4k1t/battered/main/assets/icon/battered-icon.svg" align="left">

Regularly polls battery levels and reacts to crossing configurable thresholds.

For example, it could send a notification to call attention to the battery discharging, call a script to start battery saving mode on crossing the next threshold, and send another - persistent - notification when the battery level gets critical.

-----

**Table of Contents**

- [Usage](#usage)
- [Installation](#installation)
- [Configuration](#configuration)
- [Logging](#logging)
- [License](#license)

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

## Configuration

battered looks for a configuration file in the following places:
1. `$XDG_CONFIG_HOME/battered/config.toml`
2. `$HOME/.config/battered/config.toml`
3. `/.config/battered/config.toml` if `$HOME` is not set

The `summary` and `body` fields of the `[action.notify]` table support optional placeholders which will be replaced with calculated values. The following placeholders are available:

| Placeholder | Description |
| --- | --- |
| `$percentage` | Current battery level in percent |

Example config:
```
interval = 60                        # battery level check interval in seconds; optional; defaults to 120; integer

[[action]]
percentage = 0.25                    # run action below this threshold; required; decimal
command = "./powersave.sh enable"    # CLI command to run; optional; string
[action.notify]                      # Notification settings; optional; table
summary = "Battery low!"             # Notification summary; required within action.notify table; string
body = "Battery below $percentage%!" # Notification body; optional; string
urgency = "Critical"                 # Notfication urgency; optional; defaults to `Normal`; enum[ Low | Normal | Critical ]
icon = "battery-caution"             # Notification icon; optional; defaults to "battery-discharging"; string
timeout = 0                          # Notification timeout in ms; optional; defaults to desktop default; integer; `0` means no timeout
```

## Logging

Logging is configured via the `RUST_LOG` env variable. The provided systemd unit example sets the log level to `WARN` per default.
