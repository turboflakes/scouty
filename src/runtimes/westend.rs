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
use crate::errors::ScoutyError;
use crate::hooks::{
    Hook, HOOK_NEW_ERA, HOOK_NEW_SESSION, HOOK_VALIDATOR_SLASHED,
    HOOK_VALIDATOR_STARTS_ACTIVE_NEXT_ERA, HOOK_VALIDATOR_STARTS_INACTIVE_NEXT_ERA,
};
use crate::report::{
    Network, RawDataSession, RawDataStaking, Report, Session, Slash,
    Validator, Validators,
};
use crate::scouty::Scouty;
use async_recursion::async_recursion;
use codec::{Decode, Encode};
use log::{debug, info};
use std::{result::Result, str::FromStr};
use subxt::{
    sp_core::hexdisplay::HexDisplay, sp_runtime::AccountId32, DefaultConfig, DefaultExtra,
    EventSubscription, RawEvent,
};

#[subxt::subxt(
    runtime_metadata_path = "metadata/westend_metadata.scale",
    generated_type_derives = "Clone, Debug"
)]
mod api {}

pub type KusamaApi = api::RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>>;

pub async fn subscribe_on_chain_events(scouty: &Scouty) -> Result<(), ScoutyError> {
    info!("Subscribe on-chain finalized events");
    let client = scouty.client().clone();
    let sub = client.rpc().subscribe_finalized_events().await?;
    let decoder = client.events_decoder();
    let mut sub = EventSubscription::<DefaultConfig>::new(sub, &decoder);
    // TODO: perhaps we can have a filtered events Vec inside subxt
    // sub.filter_event::<api::session::events::NewSession>();
    while let Some(result) = sub.next().await {
        if let Ok(raw) = result {
            match raw {
                RawEvent {
                    ref pallet,
                    ref variant,
                    data,
                    ..
                } if pallet == "Session" && variant == "NewSession" => {
                    // https://polkadot.js.org/docs/substrate/events#newsessionu32
                    match api::session::events::NewSession::decode(&mut &data[..]) {
                        Ok(event) => {
                            info!("Successfully decoded event {:?}", event);
                            try_run_session_hooks(&scouty, event).await?;
                        }
                        Err(e) => return Err(ScoutyError::CodecError(e)),
                    }
                }
                RawEvent {
                    ref pallet,
                    ref variant,
                    data,
                    ..
                } if pallet == "Staking" && variant == "Slashed" => {
                    // https://polkadot.js.org/docs/substrate/events#slashedaccountid32-u128-2
                    match api::staking::events::Slashed::decode(&mut &data[..]) {
                        Ok(event) => {
                            info!("Successfully decoded event {:?}", event);
                            try_run_staking_slashed_hook(&scouty, event).await?;
                        }
                        Err(e) => return Err(ScoutyError::CodecError(e)),
                    }
                }
                _ => continue,
            }
        }
    }
    // If subscription has closed for some reason await and subscribe again
    Err(ScoutyError::SubscriptionFinished)
}

async fn try_run_staking_slashed_hook(
    scouty: &Scouty,
    event: api::staking::events::Slashed,
) -> Result<(), ScoutyError> {
    let client = scouty.client();
    let _api = client.clone().to_runtime_api::<KusamaApi>();
    let config = CONFIG.clone();

    // Collect validators info based on config stashes
    let mut validators = collect_validators_data(&scouty).await?;

    // Try to run hooks for each stash
    for v in validators.iter_mut() {
        if event.0 == v.stash {
            // Set validator slashed amount
            v.is_slashed = true;
        }
    }

    debug!("validators {:?}", validators);

    // Try run hook
    let hook = Hook::try_run(
        HOOK_VALIDATOR_SLASHED,
        &config.hook_validator_slashed_path,
        vec![event.0.to_string(), event.1.to_string()],
    )?;

    // Set slash info
    let slash = Slash {
        who: event.0,
        amount_value: event.1,
        hook: hook,
    };

    let network = Network::load(&client).await?;
    debug!("network {:?}", network);

    // Prepare notification report
    let data = RawDataStaking {
        network,
        validators,
        slash,
    };

    let report = Report::from(data);
    scouty
        .send_message(&report.message(), &report.formatted_message())
        .await?;

    Ok(())
}

async fn try_run_session_hooks(
    scouty: &Scouty,
    event: api::session::events::NewSession,
) -> Result<(), ScoutyError> {
    let client = scouty.client();
    let api = client.clone().to_runtime_api::<KusamaApi>();
    let config = CONFIG.clone();

    // Get Era index
    let active_era_index = match api.storage().staking().active_era(None).await? {
        Some(info) => info.index,
        None => return Err(ScoutyError::Other("Active era not available".into())),
    };

    // Get current session
    let current_session_index = event.session_index;

    // Get start session index
    let start_session_index = match api
        .storage()
        .staking()
        .eras_start_session_index(active_era_index, None)
        .await?
    {
        Some(index) => index,
        None => {
            return Err(ScoutyError::Other(
                "Start session index not available".into(),
            ))
        }
    };

    // Eras session index
    let eras_session_index = 1 + current_session_index - start_session_index;

    // Get session keys queued status
    let queued_session_keys_changed = api.storage().session().queued_changed(None).await?;

    // Set network info
    let session = Session {
        active_era_index: active_era_index,
        current_session_index: current_session_index,
        eras_session_index: eras_session_index,
        queued_session_keys_changed: queued_session_keys_changed,
    };
    debug!("session {:?}", session);

    // Collect validators info based on config stashes
    let mut validators = collect_validators_data(&scouty).await?;

    // Try to run hooks for each stash
    for v in validators.iter_mut() {
        // Try HOOK_NEW_SESSION
        let mut args = vec![
            v.stash.to_string(),
            v.name.to_string(),
            format!("0x{:?}", HexDisplay::from(&v.queued_session_keys)),
            v.is_active.to_string(),
            v.is_queued.to_string(),
            active_era_index.to_string(),
            current_session_index.to_string(),
            eras_session_index.to_string(),
        ];

        if config.expose_nominators {
            let nominators = get_nominators(&scouty, active_era_index, &v.stash).await?;
            args.push(nominators.join(","));
        }

        // Try run hook
        let hook = Hook::try_run(
            HOOK_NEW_SESSION,
            &config.hook_new_session_path,
            args.clone(),
        )?;
        v.hooks.push(hook);

        // Try HOOK_NEW_ERA
        if (eras_session_index) == 1 {
            // Try run hook
            let hook = Hook::try_run(HOOK_NEW_ERA, &config.hook_new_era_path, args.clone())?;
            v.hooks.push(hook);
        }

        if (eras_session_index) == 6 && queued_session_keys_changed {
            let next_era_index = active_era_index + 1;
            let next_session_index = current_session_index + 1;
            let args = vec![
                v.stash.to_string(),
                v.name.to_string(),
                format!("0x{:?}", HexDisplay::from(&v.queued_session_keys)),
                format!("{}", next_era_index),
                format!("{}", next_session_index),
            ];

            // Try HOOK_VALIDATOR_STARTS_ACTIVE_NEXT_ERA
            // If stash is not active and keys are queued for next Era -> trigger hook to get ready and warm up
            if !v.is_active && v.is_queued {
                // Try run hook
                let hook = Hook::try_run(
                    HOOK_VALIDATOR_STARTS_ACTIVE_NEXT_ERA,
                    &config.hook_validator_starts_active_next_era_path,
                    args.clone(),
                )?;
                v.hooks.push(hook);
            }

            // Try HOOK_VALIDATOR_INACTIVE_NEXT_ERA
            // If stash is active and keys are not queued for next Era trigger hook to inform operator
            if v.is_active && !v.is_queued {
                let args = vec![
                    v.stash.to_string(),
                    v.name.to_string(),
                    format!("0x{:?}", HexDisplay::from(&v.queued_session_keys)),
                    format!("{}", next_era_index),
                    format!("{}", next_session_index),
                ];

                // Try run hook
                let hook = Hook::try_run(
                    HOOK_VALIDATOR_STARTS_INACTIVE_NEXT_ERA,
                    &config.hook_validator_starts_inactive_next_era_path,
                    args.clone(),
                )?;
                v.hooks.push(hook);
            }
        }
    }

    // Prepare notification report
    debug!("validators {:?}", validators);

    let network = Network::load(client).await?;
    debug!("network {:?}", network);

    let data = RawDataSession {
        network,
        session,
        validators,
    };

    let report = Report::from(data);
    scouty
        .send_message(&report.message(), &report.formatted_message())
        .await?;

    Ok(())
}

async fn get_nominators(
    scouty: &Scouty,
    era_index: u32,
    stash: &AccountId32,
) -> Result<Vec<String>, ScoutyError> {
    let client = scouty.client().clone();
    let api = client.to_runtime_api::<KusamaApi>();

    let exposure = api
        .storage()
        .staking()
        .eras_stakers(era_index, stash.clone(), None)
        .await?;

    let mut nominators: Vec<String> = vec![];
    for other in exposure.others {
        nominators.push(other.who.to_string());
    }

    Ok(nominators)
}

async fn collect_validators_data(scouty: &Scouty) -> Result<Validators, ScoutyError> {
    let client = scouty.client().clone();
    let api = client.to_runtime_api::<KusamaApi>();
    let config = CONFIG.clone();

    // Verify session active validators
    let active_validators = api.storage().session().validators(None).await?;

    // Verify session queued keys
    let queued_keys = api.storage().session().queued_keys(None).await?;

    let mut validators: Validators = Vec::new();
    for stash_str in config.stashes.iter() {
        let stash = AccountId32::from_str(stash_str)?;
        let mut v = Validator::new(stash.clone());

        // Get validator name
        v.name = get_display_name(&scouty, &stash, None).await?;

        // Check if validator is in active set
        v.is_active = active_validators.contains(&v.stash);

        // Check if validator session key is queued
        for (account_id, session_keys) in &queued_keys {
            if account_id == &v.stash {
                v.is_queued = true;
                v.queued_session_keys = session_keys.encode();
                break;
            }
        }

        validators.push(v);
    }

    debug!("validators {:?}", validators);
    Ok(validators)
}

#[async_recursion]
async fn get_display_name(
    scouty: &Scouty,
    stash: &AccountId32,
    sub_account_name: Option<String>,
) -> Result<String, ScoutyError> {
    let client = scouty.client();
    let api = client.clone().to_runtime_api::<KusamaApi>();

    match api
        .storage()
        .identity()
        .identity_of(stash.clone(), None)
        .await?
    {
        Some(identity) => {
            debug!("identity {:?}", identity);
            let parent = parse_identity_data(identity.info.display);
            let name = match sub_account_name {
                Some(child) => format!("{}/{}", parent, child),
                None => parent,
            };
            Ok(name)
        }
        None => {
            if let Some((parent_account, data)) = api
                .storage()
                .identity()
                .super_of(stash.clone(), None)
                .await?
            {
                let sub_account_name = parse_identity_data(data);
                return get_display_name(
                    &scouty,
                    &parent_account,
                    Some(sub_account_name.to_string()),
                )
                .await;
            } else {
                let s = &stash.to_string();
                Ok(format!("{}...{}", &s[..6], &s[s.len() - 6..]))
            }
        }
    }
}

fn parse_identity_data(data: api::runtime_types::pallet_identity::types::Data) -> String {
    match data {
        api::runtime_types::pallet_identity::types::Data::Raw0(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw1(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw2(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw3(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw4(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw5(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw6(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw7(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw8(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw9(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw10(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw11(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw12(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw13(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw14(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw15(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw16(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw17(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw18(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw19(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw20(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw21(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw22(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw23(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw24(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw25(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw26(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw27(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw28(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw29(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw30(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw31(bytes) => str(bytes.to_vec()),
        api::runtime_types::pallet_identity::types::Data::Raw32(bytes) => str(bytes.to_vec()),
        _ => format!("???"),
    }
}

fn str(bytes: Vec<u8>) -> String {
    format!("{}", String::from_utf8(bytes).expect("Identity not utf-8"))
}
