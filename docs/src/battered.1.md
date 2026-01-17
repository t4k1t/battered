% battered(1) | General Commands Manual
% Thomas Kager
% June 2025

# NAME

battered - Make the most of your battery life with custom actions and desktop notifications

# SYNOPSIS

**battered** [OPTIONS]

# DESCRIPTION

Make the most of your battery life with custom actions and informative desktop notifications.

# OPTIONS

**-h**, **\--help**
: Print help information.

**-V**, **\--version**
: Print version information.

# CONFIGURATION FILE

**battered** reads its configuration from a file in one of the following locations (checked in order):

1. _$XDG_CONFIG_HOME/battered/config.toml_
2. _$HOME/.config/battered/config.toml_
3. _/.config/battered/config.toml_ (if HOME is not set)

A minimal configuration file must contain at least one `[[action]]` entry. See battered(5) for details on configuration options.

# ENVIRONMENT

**battered** can be configured using environment variables.

**RUST_LOG**
: Logging is configured via the RUST_LOG environment variable. Possible values are "error", "warn", "info", "debug", "trace", or "off" (and these values are case-insensitive). Defaults to "error".

# BUGS

Issue reports or feature requests can be filed at https://github.com/t4k1t/battered/issues
