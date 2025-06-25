% battered(5) | File Formats Manual
% Thomas Kager
% June 2025

# NAME

battered - **battered** configuration file

# DESCRIPTION

_battered_(1) obtains configuration data from the following sources in the following order:

1. _$XDG_CONFIG_HOME/battered/config.toml_
2. _$HOME/.config/battered/config.toml_
3. _/.config/battered/config.toml_ - if HOME is not set

# PLACEHOLDER VALUES

The _summary_ and _body_ fields of the _[action.notify]_ table support optional placeholders which will be replaced with calculated values. The following placeholders are available:

**percentage**
: Current battery level in percent.

# GENERAL SETTINGS

**interval** <seconds>
: Battery level check interval in seconds. Defaults to 120.

**serial_number** <battery-serial-number>
: Specifies which battery to monitor, if device has multiple batteries. If this is not set, **battered** will pick the first battery it finds.

**\[\[action\]\]** <array-of-tables>
: At least on action has to be configured. See ACTIONS for more details.

**\[on_ac\]** <table>
: Optional. See ON_AC for a description.

# ACTIONS

Actions are the main way to configure the behavior of **battered**. They specify what to do on dropping battery levels. There is no limit to how many actions can be defined. It does not matter in which order actions are defined within the config file, they will automatically get picked up based on the percentage.

**percentage**: <percent>
: Once battery level drops below this percentage (expressed as decimal between 0 and 1), this action is executed. This setting is required.

**command**: <command>
: Shell command to run on execution of this action. Optional.

**\[action.notify\]**: <sub-table>
: Optional. See NOTIFY for more details.

# NOTIFY

The **notify** sub-table makes it easy to configure desktop notifications for actions. It can be omitted entirely if showing a desktop notification is not desired.

**summary** <text>
: Summary, or title, of the desktop notification. Optional.

**body** <text>
: The body of the desktop notification. Optional.

**urgency**
: Urgency of the desktop notification. Possible values are "Low", "Normal", "Critical". Optional.

**icon**
: Choose an icon for the desktop notification. Optional.

**timeout** <seconds>
: Set a timeout for the desktop notification. Setting this to 0 means the notification will never time out. Optional.

# ON_AC

The **\[on_ac\]** action is a special, optional, action which runs once the monitored battery is connected to a power supply. It takes the same settings as an action - the only difference is that here the percentage is optional.

# MINIMAL CONFIGURATION

A minimal config file might look something like this:

```
[[action]]
percentage = 0.5
command = "./powersave.sh enable"

[on_ac]
percentage = 0.10
command = "./powersave.sh disable"
[on_ac.notify]
summary = "Connected to power supply"
```
