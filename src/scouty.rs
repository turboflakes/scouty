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

use crate::config::{Config, CONFIG};
use crate::errors::ScoutyError;
use crate::hooks::{
    Hook, HOOK_DEMOCRACY_STARTED, HOOK_INIT, HOOK_NEW_ERA, HOOK_NEW_SESSION,
    HOOK_VALIDATOR_CHILLED, HOOK_VALIDATOR_OFFLINE, HOOK_VALIDATOR_SLASHED,
    HOOK_VALIDATOR_STARTS_ACTIVE_NEXT_ERA, HOOK_VALIDATOR_STARTS_INACTIVE_NEXT_ERA,
};
use crate::matrix::Matrix;
use crate::runtimes::{
    kusama, polkadot,
    support::{ChainPrefix, ChainTokenSymbol, SupportedRuntime},
    westend,
};

use async_std::task;
use log::{error, info, warn};
use std::{convert::TryInto, result::Result, thread, time};
use subxt::{
    ext::sp_core::crypto, storage::StorageKey, utils::AccountId32, OnlineClient,
    PolkadotConfig,
};

pub async fn create_substrate_node_client(
    config: Config,
) -> Result<OnlineClient<PolkadotConfig>, subxt::Error> {
    OnlineClient::<PolkadotConfig>::from_url(config.substrate_ws_url).await
}

pub async fn create_or_await_substrate_node_client(
    config: Config,
) -> (OnlineClient<PolkadotConfig>, SupportedRuntime) {
    loop {
        match create_substrate_node_client(config.clone()).await {
            Ok(client) => {
                let chain = client.rpc().system_chain().await.unwrap_or_default();
                let name = client.rpc().system_name().await.unwrap_or_default();
                let version = client.rpc().system_version().await.unwrap_or_default();
                let properties =
                    client.rpc().system_properties().await.unwrap_or_default();

                // Display SS58 addresses based on the connected chain
                let chain_prefix: ChainPrefix =
                    if let Some(ss58_format) = properties.get("ss58Format") {
                        ss58_format.as_u64().unwrap_or_default().try_into().unwrap()
                    } else {
                        0
                    };

                crypto::set_default_ss58_version(crypto::Ss58AddressFormat::custom(
                    chain_prefix,
                ));

                let chain_token_symbol: ChainTokenSymbol =
                    if let Some(token_symbol) = properties.get("tokenSymbol") {
                        use serde_json::Value::String;
                        match token_symbol {
                            String(token_symbol) => token_symbol.to_string(),
                            _ => unreachable!("Token symbol with wrong type"),
                        }
                    } else {
                        String::from("")
                    };

                info!(
                    "Connected to {} network using {} * Substrate node {} v{}",
                    chain, config.substrate_ws_url, name, version
                );

                break (client, SupportedRuntime::from(chain_token_symbol));
            }
            Err(e) => {
                error!("{}", e);
                info!("Awaiting for connection using {}", config.substrate_ws_url);
                thread::sleep(time::Duration::from_secs(6));
            }
        }
    }
}

pub struct Scouty {
    runtime: SupportedRuntime,
    client: OnlineClient<PolkadotConfig>,
    matrix: Matrix,
}

impl Scouty {
    async fn new() -> Scouty {
        let (client, runtime) =
            create_or_await_substrate_node_client(CONFIG.clone()).await;

        // Initialize matrix client
        let mut matrix: Matrix = Matrix::new();
        matrix.authenticate(runtime).await.unwrap_or_else(|e| {
            error!("{}", e);
            Default::default()
        });

        Scouty {
            runtime,
            client,
            matrix,
        }
    }

    pub fn client(&self) -> &OnlineClient<PolkadotConfig> {
        &self.client
    }

    /// Returns the matrix configuration
    pub fn matrix(&self) -> &Matrix {
        &self.matrix
    }

    pub async fn send_message(
        &self,
        message: &str,
        formatted_message: &str,
    ) -> Result<(), ScoutyError> {
        self.matrix()
            .send_message(message, formatted_message)
            .await?;
        Ok(())
    }

    /// Spawn and restart subscription on error
    pub fn subscribe() {
        spawn_and_restart_subscription_on_error();
    }

    async fn subscribe_on_chain_events(&self) -> Result<(), ScoutyError> {
        let config = CONFIG.clone();

        // Verify if hooks scripts are available
        Hook::exists(HOOK_INIT, &config.hook_init_path);
        Hook::exists(HOOK_NEW_SESSION, &config.hook_new_session_path);
        Hook::exists(HOOK_NEW_ERA, &config.hook_new_era_path);
        Hook::exists(
            HOOK_VALIDATOR_STARTS_ACTIVE_NEXT_ERA,
            &config.hook_validator_starts_active_next_era_path,
        );
        Hook::exists(
            HOOK_VALIDATOR_STARTS_INACTIVE_NEXT_ERA,
            &config.hook_validator_starts_inactive_next_era_path,
        );
        Hook::exists(HOOK_VALIDATOR_SLASHED, &config.hook_validator_slashed_path);
        Hook::exists(HOOK_VALIDATOR_CHILLED, &config.hook_validator_chilled_path);
        Hook::exists(HOOK_VALIDATOR_OFFLINE, &config.hook_validator_offline_path);
        Hook::exists(HOOK_DEMOCRACY_STARTED, &config.hook_democracy_started_path);

        match self.runtime {
            SupportedRuntime::Polkadot => {
                polkadot::init_and_subscribe_on_chain_events(self).await
            }
            SupportedRuntime::Kusama => {
                kusama::init_and_subscribe_on_chain_events(self).await
            }
            SupportedRuntime::Westend => {
                westend::init_and_subscribe_on_chain_events(self).await
            } // _ => unreachable!(),
        }
    }
}

fn spawn_and_restart_subscription_on_error() {
    let t = task::spawn(async {
        let config = CONFIG.clone();
        loop {
            let c: Scouty = Scouty::new().await;
            if let Err(e) = c.subscribe_on_chain_events().await {
                match e {
                    ScoutyError::SubscriptionFinished => warn!("{}", e),
                    ScoutyError::MatrixError(_) => warn!("Matrix message skipped!"),
                    _ => {
                        error!("{}", e);
                        let message =
                            format!("On hold for {} min!", config.error_interval);
                        let formatted_message = format!("<br/>🚨 An error was raised -> <code>scouty</code> on hold for {} min while rescue is on the way 🚁 🚒 🚑 🚓<br/><br/>", config.error_interval);
                        c.send_message(&message, &formatted_message).await.unwrap();
                        thread::sleep(time::Duration::from_secs(
                            60 * config.error_interval,
                        ));
                        continue;
                    }
                }
                thread::sleep(time::Duration::from_secs(1));
            };
        }
    });
    task::block_on(t);
}

pub fn get_account_id_from_storage_key(key: StorageKey) -> AccountId32 {
    let s = &key.0[key.0.len() - 32..];
    let v: [u8; 32] = s.try_into().expect("slice with incorrect length");
    v.into()
}
