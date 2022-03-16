// The MIT License (MIT)
// Copyright © 2021 Aukbit Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// Load environment variables into a Config struct
//
// Envy is a library for deserializing environment variables into
// typesafe structs
//
// Dotenv loads environment variables from a .env file, if available,
// and mashes those with the actual environment variables provided by
// the operative system.
//
// Set Config struct into a CONFIG lazy_static to avoid multiple processing.
//
use clap::{App, Arg};
use dotenv;
use lazy_static::lazy_static;
use log::info;
use serde::Deserialize;
use std::env;

// Set Config struct into a CONFIG lazy_static to avoid multiple processing
lazy_static! {
    pub static ref CONFIG: Config = get_config();
}

/// provides default value for interval if SCOUTY_INTERVAL env var is not set
fn default_interval() -> u64 {
    21600
}

/// provides default value for error interval if SCOUTY_ERROR_INTERVAL env var is not set
fn default_error_interval() -> u64 {
    30
}

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_interval")]
    pub interval: u64,
    #[serde(default = "default_error_interval")]
    pub error_interval: u64,
    pub substrate_ws_url: String,
    pub stashes: Vec<String>,
    #[serde(default)]
    pub is_debug: bool,
    #[serde(default)]
    pub is_short: bool,
    // hooks configuration
    #[serde(default)]
    pub hook_init_path: String,
    #[serde(default)]
    pub hook_new_session_path: String,
    #[serde(default)]
    pub hook_new_era_path: String,
    #[serde(default)]
    pub hook_validator_starts_active_next_era_path: String,
    #[serde(default)]
    pub hook_validator_starts_inactive_next_era_path: String,
    #[serde(default)]
    pub hook_validator_chilled_path: String,
    #[serde(default)]
    pub hook_validator_slashed_path: String,
    #[serde(default)]
    pub hook_validator_offline_path: String,
    #[serde(default)]
    pub hook_democracy_started_path: String,
    // matrix configuration
    #[serde(default)]
    pub matrix_user: String,
    #[serde(default)]
    pub matrix_bot_user: String,
    #[serde(default)]
    pub matrix_bot_password: String,
    #[serde(default)]
    pub matrix_disabled: bool,
    #[serde(default)]
    pub matrix_bot_display_name_disabled: bool,
    // chain settings exposure
    #[serde(default)]
    pub expose_network: bool,
    #[serde(default)]
    pub expose_nominators: bool,
    #[serde(default)]
    pub expose_authored_blocks: bool,
    #[serde(default)]
    pub expose_all_nominators: bool,
    #[serde(default)]
    pub expose_para_validator: bool,
    #[serde(default)]
    pub expose_era_points: bool,
    #[serde(default)]
    pub expose_all: bool,
}

/// Inject dotenv and env vars into the Config struct
fn get_config() -> Config {
    // Define CLI flags with clap
    let matches = App::new(env!("CARGO_PKG_NAME"))
    .version(env!("CARGO_PKG_VERSION"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .about(env!("CARGO_PKG_DESCRIPTION"))
    .arg(
      Arg::with_name("CHAIN")
          .index(1)
          .possible_values(&["westend", "kusama", "polkadot"])
          .help(
            "Sets the substrate-based chain for which 'scouty' will try to connect",
          )
    )
    .arg(
      Arg::with_name("debug")
        .long("debug")
        .help("Prints debug information verbosely."))
    .arg(
      Arg::with_name("matrix-user")
        .long("matrix-user")
        .takes_value(true)
        .help("Your regular matrix user. e.g. '@your-regular-matrix-account:matrix.org' this user account will receive notifications from your other 'Scouty Bot' matrix account."))
    .arg(
          Arg::with_name("matrix-bot-user")
            .long("matrix-bot-user")
            .takes_value(true)
            .help("Your new 'Scouty Bot' matrix user. e.g. '@your-own-scouty-bot-account:matrix.org' this user account will be your 'Scouty Bot' which will be responsible to send messages/notifications to your private 'Scouty Bot' room."))
    .arg(
      Arg::with_name("matrix-bot-password")
        .long("matrix-bot-password")
        .takes_value(true)
        .help("Password for the 'Scouty Bot' matrix user sign in."))
    .arg(
      Arg::with_name("disable-matrix")
        .long("disable-matrix")
        .help(
          "Disable matrix bot for 'scouty'. (e.g. with this flag active 'scouty' will not send messages/notifications to your private 'Scouty Bot' room) (https://matrix.org/)",
        ),
    )
    .arg(
      Arg::with_name("disable-matrix-bot-display-name")
        .long("disable-matrix-bot-display-name")
        .help(
          "Disable matrix bot display name update for 'scouty'. (e.g. with this flag active 'scouty' will not change the matrix bot user display name)",
        ),
      )
    .arg(
      Arg::with_name("short")
        .long("short")
        .help("Display only essential information (e.g. with this flag active 'scouty' will hide certain sections in a message)"))
    .arg(
      Arg::with_name("error-interval")
        .long("error-interval")
        .takes_value(true)
        .default_value("30")
        .help("Interval value (in minutes) from which 'scouty' will restart again in case of a critical error."))
    .arg(
      Arg::with_name("stashes")
        .short("s")
        .long("stashes")
        .takes_value(true)
        .help(
          "Validator stash addresses for which 'scouty' will take a particular eye. If needed specify more than one (e.g. stash_1,stash_2,stash_3).",
        ),
    )
    .arg(
      Arg::with_name("substrate-ws-url")
        .short("w")
        .long("substrate-ws-url")
        .takes_value(true)
        .help(
          "Substrate websocket endpoint for which 'scouty' will try to connect. (e.g. wss://kusama-rpc.polkadot.io) (NOTE: substrate_ws_url takes precedence than <CHAIN> argument)",
        ),
    )
    .arg(
      Arg::with_name("config-path")
        .short("c")
        .long("config-path")
        .takes_value(true)
        .value_name("FILE")
        .default_value(".env")
        .help(
          "Sets a custom config file path. The config file contains 'scouty' configuration variables.",
        ),
    )
    .arg(
      Arg::with_name("expose-network")
        .long("expose-network")
        .help(
          "Expose the network name, token symbol and token decimal under new positional arguments for each hook.",
        ),
      )
    .arg(
      Arg::with_name("expose-nominators")
        .long("expose-nominators")
        .help(
          "Expose ACTIVE nominator details under new positional arguments for some of the hooks. Note: `scouty` only look after active nominators for each validator stash predefined.",
        ),
      )
    .arg(
        Arg::with_name("expose-authored-blocks")
          .long("expose-authored-blocks")
          .help(
            "Expose the number of blocks authored by each validator stash predefined.",
          ),
        )
    .arg(
      Arg::with_name("expose-all-nominators")
        .long("expose-all-nominators")
        .help(
          "Expose ALL nominator details under new positional arguments for some of the hooks. Note: `scouty` only look after all nominators for each validator stash predefined.",
        ),
      )
    .arg(
      Arg::with_name("expose-para-validator")
        .long("expose-para-validator")
        .help(
          "Expose the para validator details under new positional arguments for some of the hooks.",
        ),
      )
    .arg(
      Arg::with_name("expose-era-points")
        .long("expose-era-points")
        .help(
          "Expose the era points details under new positional arguments for the `_new_era` hook.",
        ),
      )
    .arg(
      Arg::with_name("expose-all")
        .long("expose-all")
        .help(
          "Expose all positional arguments for some of the hooks. Note: Each hook bash script describes which data is available through the positional arguments.",
        ),
      )
    .arg(
        Arg::with_name("hook-init-path")
          .long("hook-init-path")
          .takes_value(true)
          .value_name("FILE")
          .help(
            "Sets the path for the script that is called every time `scouty` starts. Here is a good place for try out new things and test new scripts.",
          ),
      )
    .arg(
      Arg::with_name("hook-new-session-path")
        .long("hook-new-session-path")
        .takes_value(true)
        .value_name("FILE")
        .help(
          "Sets the path for the script that is called every new session.",
        ),
    )
    .arg(
      Arg::with_name("hook-new-era-path")
        .long("hook-new-era-path")
        .takes_value(true)
        .value_name("FILE")
        .help(
          "Sets the path for the script that is called every new era.",
        ),
    )
    .arg(
      Arg::with_name("hook-validator-starts-active-next-era-path")
        .long("hook-validator-starts-active-next-era-path")
        .takes_value(true)
        .value_name("FILE")
        .help(
          "Sets the path for the script that is called on the last session of an era, if the stash is NOT ACTIVE and keys are QUEUED for the next Session/Era.",
        ),
    )
    .arg(
      Arg::with_name("hook-validator-starts-inactive-next-era-path")
        .long("hook-validator-starts-inactive-next-era-path")
        .takes_value(true)
        .value_name("FILE")
        .help(
          "Sets the path for the script that is called on the last session of an era, if the stash is ACTIVE and keys are NOT QUEUED for the next Session/Era.",
        ),
    )
    .arg(
      Arg::with_name("hook-validator-slashed-path")
        .long("hook-validator-slashed-path")
        .takes_value(true)
        .value_name("FILE")
        .help(
          "Sets the path for the script that is called every time a Slash occurred on the network.",
        ),
    )
    .arg(
      Arg::with_name("hook-validator-chilled-path")
        .long("hook-validator-chilled-path")
        .takes_value(true)
        .value_name("FILE")
        .help(
          "Sets the path for the script that is called every time one of the Validator stashes defined is chilled.",
        ),
    )
    .arg(
      Arg::with_name("hook-validator-offline-path")
        .long("hook-validator-offline-path")
        .takes_value(true)
        .value_name("FILE")
        .help(
          "Sets the path for the script that is called every time one of the Validator stashes defined is offline at the end of a session.",
        ),
    )
    .get_matches();

    // Try to load configuration from file first
    let config_path = matches.value_of("config-path").unwrap_or(".env");

    match dotenv::from_filename(&config_path).ok() {
        Some(_) => info!("Loading configuration from {} file", &config_path),
        None => {
            let config_path =
                env::var("SCOUTY_CONFIG_FILENAME").unwrap_or(".env".to_string());
            if let Some(_) = dotenv::from_filename(&config_path).ok() {
                info!("Loading configuration from {} file", &config_path);
            }
        }
    }

    match matches.value_of("CHAIN") {
        Some("westend") => {
            env::set_var(
                "SCOUTY_SUBSTRATE_WS_URL",
                "wss://westend-rpc.polkadot.io:443",
            );
        }
        Some("kusama") => {
            env::set_var(
                "SCOUTY_SUBSTRATE_WS_URL",
                "wss://kusama-rpc.polkadot.io:443",
            );
        }
        Some("polkadot") => {
            env::set_var("SCOUTY_SUBSTRATE_WS_URL", "wss://rpc.polkadot.io:443");
        }
        _ => {
            if env::var("SCOUTY_SUBSTRATE_WS_URL").is_err() {
                env::set_var("SCOUTY_SUBSTRATE_WS_URL", "ws://127.0.0.1:9944");
            };
        }
    }

    if let Some(stashes) = matches.value_of("stashes") {
        env::set_var("SCOUTY_STASHES", stashes);
    }

    if let Some(substrate_ws_url) = matches.value_of("substrate-ws-url") {
        env::set_var("SCOUTY_SUBSTRATE_WS_URL", substrate_ws_url);
    }

    if matches.is_present("debug") {
        env::set_var("SCOUTY_IS_DEBUG", "true");
    }

    if matches.is_present("short") {
        env::set_var("SCOUTY_IS_SHORT", "true");
    }

    if let Some(hook_init_path) = matches.value_of("hook-init-path") {
        env::set_var("SCOUTY_HOOK_INIT_PATH", hook_init_path);
    }

    if let Some(hook_new_session_path) = matches.value_of("hook-new-session-path") {
        env::set_var("SCOUTY_HOOK_NEW_SESSION_PATH", hook_new_session_path);
    }

    if let Some(hook_new_era_path) = matches.value_of("hook-new-era-path") {
        env::set_var("SCOUTY_HOOK_NEW_ERA_PATH", hook_new_era_path);
    }

    if let Some(hook_validator_starts_active_next_era_path) =
        matches.value_of("hook-validator-starts-active-next-era-path")
    {
        env::set_var(
            "SCOUTY_HOOK_VALIDATOR_STARTS_ACTIVE_NEXT_ERA_PATH",
            hook_validator_starts_active_next_era_path,
        );
    }

    if let Some(hook_validator_starts_inactive_next_era_path) =
        matches.value_of("hook-validator-starts-inactive-next-era-path")
    {
        env::set_var(
            "SCOUTY_HOOK_VALIDATOR_STARTS_INACTIVE_NEXT_ERA_PATH",
            hook_validator_starts_inactive_next_era_path,
        );
    }

    if let Some(hook_validator_slashed_path) =
        matches.value_of("hook-validator-slashed-path")
    {
        env::set_var(
            "SCOUTY_HOOK_VALIDATOR_SLASHED_PATH",
            hook_validator_slashed_path,
        );
    }

    if let Some(hook_validator_chilled_path) =
        matches.value_of("hook-validator-chilled-path")
    {
        env::set_var(
            "SCOUTY_HOOK_VALIDATOR_CHILLED_PATH",
            hook_validator_chilled_path,
        );
    }

    if let Some(hook_validator_offline_path) =
        matches.value_of("hook-validator-offline-path")
    {
        env::set_var(
            "SCOUTY_HOOK_VALIDATOR_OFFLINE_PATH",
            hook_validator_offline_path,
        );
    }

    if let Some(hook_democracy_started_path) =
        matches.value_of("hook-democracy-started-path")
    {
        env::set_var(
            "SCOUTY_HOOK_DEMOCRACY_STARTED_PATH",
            hook_democracy_started_path,
        );
    }

    if matches.is_present("expose-all") {
        env::set_var("SCOUTY_EXPOSE_ALL", "true");
    }

    if matches.is_present("expose-network") {
        env::set_var("SCOUTY_EXPOSE_NETWORK", "true");
    }

    if matches.is_present("expose-nominators") {
        env::set_var("SCOUTY_EXPOSE_NOMINATORS", "true");
    }

    if matches.is_present("expose-all-nominators") {
        env::set_var("SCOUTY_EXPOSE_ALL_NOMINATORS", "true");
    }

    if matches.is_present("expose-authored-blocks") {
        env::set_var("SCOUTY_EXPOSE_AUTHORED_BLOCKS", "true");
    }

    if matches.is_present("expose-para-validator") {
        env::set_var("SCOUTY_EXPOSE_PARA_VALIDATOR", "true");
    }

    if matches.is_present("expose-era-points") {
        env::set_var("SCOUTY_EXPOSE_ERA_POINTS", "true");
    }

    if matches.is_present("expose-all") {
        env::set_var("SCOUTY_EXPOSE_ALL", "true");
    }

    if matches.is_present("disable-matrix") {
        env::set_var("SCOUTY_MATRIX_DISABLED", "true");
    }

    if let Some(matrix_user) = matches.value_of("matrix-user") {
        env::set_var("SCOUTY_MATRIX_ACCOUNT", matrix_user);
    }

    if let Some(matrix_bot_user) = matches.value_of("matrix-bot-user") {
        env::set_var("SCOUTY_MATRIX_BOT_USER", matrix_bot_user);
    }

    if let Some(matrix_bot_password) = matches.value_of("matrix-bot-password") {
        env::set_var("SCOUTY_MATRIX_BOT_PASSWORD", matrix_bot_password);
    }

    if let Some(error_interval) = matches.value_of("error-interval") {
        env::set_var("SCOUTY_ERROR_INTERVAL", error_interval);
    }

    match envy::prefixed("SCOUTY_").from_env::<Config>() {
        Ok(config) => config,
        Err(error) => panic!("Configuration error: {:#?}", error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_gets_a_config() {
        let config = get_config();
        assert_ne!(config.substrate_ws_url, "".to_string());
    }

    #[test]
    fn it_gets_a_config_from_the_lazy_static() {
        let config = &CONFIG;
        assert_ne!(config.substrate_ws_url, "".to_string());
    }
}
