# battered

[![crates.io](https://img.shields.io/crates/v/battered?logo=rust)](https://crates.io/crates/battered)

<img height="128" alt="battered Icon" src="https://raw.githubusercontent.com/t4k1t/battered/main/assets/icon/battered-icon.svg" align="left">

Regularly polls battery levels and sends notifications on crossing certain thresholds.

The idea is to have one notification to warn about the battery discharging and another, persistent, notification when action has to be taken. Both thresholds and the poll interval can be configured.

-----

**Table of Contents**

- [Usage](#usage)
- [Installation](#installation)
- [Configuration](#configuration)
- [Logging](#logging)
- [License](#license)

## Usage

Simply run `battered`:

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

Example config:
```
[general]
interval = 60              # in seconds
threshold_low = 0.8        # percentage as decimal
threshold_critical = 0.25  # percentage as decimal
action_low = "tuned-adm profile laptop-battery-powersave"
action_critical = "systemctl suspend"
```

## Logging

Logging is configured via the `RUST_LOG` env variable. The provided systemd unit example sets the log level to `WARN` per default.
