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

use crate::config::CONFIG;
use crate::errors::SkipperError;
use crate::skipper::{try_call_hook, verify_hook};
use crate::validator::{Validator, Validators};
use codec::Decode;
use log::{debug, info};
use std::{result::Result, str::FromStr};
use subxt::{sp_runtime::AccountId32, Client, DefaultConfig, DefaultExtra, EventSubscription};

#[subxt::subxt(
    runtime_metadata_path = "metadata/polkadot_metadata.scale",
    generated_type_derives = "Clone, Debug"
)]
mod polkadot {}

pub type PolkadotApi = polkadot::RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>>;

pub async fn run_and_subscribe_new_session_events(
    client: &Client<DefaultConfig>,
) -> Result<(), SkipperError> {
    info!("Check Validator on-chain status");
    check_validator_status(client).await?;
    info!("Subscribe 'NewSession' on-chain finalized event");
    let client = client.clone();
    let sub = client.rpc().subscribe_finalized_events().await?;
    let decoder = client.events_decoder();
    let mut sub = EventSubscription::<DefaultConfig>::new(sub, &decoder);
    sub.filter_event::<polkadot::session::events::NewSession>();
    while let Some(result) = sub.next().await {
        if let Ok(raw) = result {
            match polkadot::session::events::NewSession::decode(&mut &raw.data[..]) {
                Ok(event) => {
                    info!("Successfully decoded event {:?}", event);
                    check_validator_status(&client).await?;
                }
                Err(e) => return Err(SkipperError::CodecError(e)),
            }
        }
    }
    // If subscription has closed for some reason await and subscribe again
    Err(SkipperError::SubscriptionFinished)
}

async fn check_validator_status(client: &Client<DefaultConfig>) -> Result<(), SkipperError> {
    let client = client.clone();
    let api = client.to_runtime_api::<PolkadotApi>();
    let config = CONFIG.clone();

    // check hook paths
    verify_hook("New Session", &config.hook_new_session_path);
    verify_hook("Active next Era", &config.hook_active_next_era_path);
    verify_hook("Inactive next Era", &config.hook_inactive_next_era_path);

    let mut validators: Validators = Vec::new();
    for stash_str in config.stashes.iter() {
        let stash = AccountId32::from_str(stash_str)?;
        let v = Validator::new(stash.clone());
        validators.push(v);
    }

    // Verify session queued keys
    let queued_keys = api.storage().session().queued_keys(None).await?;
    for (account_id, _session_keys) in &queued_keys {
        debug!("{}", account_id.to_string());
        for v in validators.iter_mut() {
            if account_id == &v.stash {
                debug!("account_id {} is_queued", account_id.to_string());
                v.is_queued = true
            }
        }
    }

    // Verify session active validators
    let active_validators = api.storage().session().validators(None).await?;
    for v in validators.iter_mut() {
        // Check if validator is in active set
        v.is_active = active_validators.contains(&v.stash);
    }

    if let Some(active_era_info) = api.storage().staking().active_era(None).await? {
        // info!("active_era_info {:?}", active_era_info);
        // if let Some(start) = active_era_info.start {
        //     let now = api.storage().timestamp().now(None).await?;
        //     // Inside the first session of an Era
        //     // 1 hour -> 1*60*60*1000 = 3_600_000 milliseconds
        //     if (now - start) <= 3_600_000 {
        //         info!("FIRST session of the era!");
        //     }
        //     // Inside the last session of an Era
        //     // 5 hours -> 5*60*60*1000 = 18_000_000 milliseconds
        //     if (now - start) >= 18_000_000 {
        //         info!("LAST session of the era!");
        //     }
        // }

        let active_era_index = active_era_info.index;

        if let Some(start_session_index) = api
            .storage()
            .staking()
            .eras_start_session_index(active_era_index, None)
            .await?
        {
            let current_session_index = api.storage().session().current_index(None).await?;
            let era_session_index = current_session_index - start_session_index;

            info!(
                "Era {} -> Session {} ({})",
                active_era_index, current_session_index, era_session_index
            );

            for v in validators.iter() {
                try_call_hook(
                    "New Session",
                    &config.hook_new_session_path,
                    vec![
                        v.stash.to_string(),
                        v.is_active.to_string(),
                        active_era_index.to_string(),
                        current_session_index.to_string(),
                        era_session_index.to_string(),
                    ],
                )?;
            }

            if (era_session_index) == 5 {
                info!("Era {} (last session)", active_era_index);
                let queued_changed = api.storage().session().queued_changed(None).await?;
                if queued_changed {
                    for v in validators.iter() {
                        // If stash is not active and keys are queued for next Era -> trigger hook to get ready and warm up
                        if !v.is_active && v.is_queued {
                            try_call_hook(
                                "Active next Era",
                                &config.hook_active_next_era_path,
                                vec![
                                    v.stash.to_string(),
                                    active_era_index.to_string(),
                                    current_session_index.to_string(),
                                    format!("{}", active_era_index + 1),
                                    format!("{}", current_session_index + 1),
                                ],
                            )?;
                        }
                        // If stash is active and keys are not queued for next Era trigger hook to inform operator
                        else if v.is_active && !v.is_queued {
                            try_call_hook(
                                "Inactive next Era",
                                &config.hook_inactive_next_era_path,
                                vec![
                                    v.stash.to_string(),
                                    active_era_index.to_string(),
                                    current_session_index.to_string(),
                                    format!("{}", active_era_index + 1),
                                    format!("{}", current_session_index + 1),
                                ],
                            )?;
                        }
                    }
                }
            }
        }
    }

    debug!("{:?}", validators);
    Ok(())
}
