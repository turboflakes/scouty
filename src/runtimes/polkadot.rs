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

use crate::authority::{AuthorityIndex, AuthorityRecords};
use crate::config::CONFIG;
use crate::errors::ScoutyError;
use crate::hooks::{
    Hook, HOOK_INIT, HOOK_NEW_ERA, HOOK_NEW_SESSION, HOOK_REFERENDA_SUBMITTED,
    HOOK_VALIDATOR_CHILLED, HOOK_VALIDATOR_OFFLINE, HOOK_VALIDATOR_SLASHED,
    HOOK_VALIDATOR_STARTS_ACTIVE_NEXT_ERA, HOOK_VALIDATOR_STARTS_INACTIVE_NEXT_ERA,
};
use crate::identity::Identity;
use crate::para::ParaRecords;
use crate::report::{
    Init, Network, Points, RawData, Referendum, Report, Section, Session, Slash,
    Validator, Validators,
};
use crate::scouty::{convert_account_id, get_account_id_from_storage_key, Scouty};
use crate::stats;
use async_recursion::async_recursion;
use codec::{Decode, Encode};
use log::{debug, info};
use std::{collections::BTreeMap, convert::TryInto, result::Result, str::FromStr};
use subxt::{
    config::substrate::{Digest, DigestItem},
    ext::sp_core::{hexdisplay::HexDisplay, H256},
    utils::AccountId32,
};

#[subxt::subxt(
    runtime_metadata_path = "metadata/polkadot_metadata.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
mod node_runtime {}

use node_runtime::{
    im_online::events::SomeOffline,
    referenda::events::Submitted,
    runtime_types::{
        bounded_collections::bounded_vec::BoundedVec, pallet_identity::types::Data,
        polkadot_primitives::v4::ValidatorIndex, sp_consensus_babe::digests::PreDigest,
    },
    session::events::NewSession,
    staking::events::Chilled,
    staking::events::Slashed,
};

const ERAS_PER_DAY: u32 = 4;

pub async fn init_and_subscribe_on_chain_events(
    scouty: &Scouty,
) -> Result<(), ScoutyError> {
    let api = scouty.client().clone();

    // Initialize authority records
    let mut authority_records = AuthorityRecords::new();
    init_authority_records(&scouty, &mut authority_records).await?;

    // Initialize para records
    let mut para_records = ParaRecords::new();
    init_para_records(&scouty, &mut para_records).await?;

    // Start by calling init hook
    try_init_hook(&scouty, &authority_records, &para_records).await?;
    //
    info!("Subscribe on-chain finalized blocks");
    let mut blocks_sub = api.blocks().subscribe_finalized().await?;
    while let Some(block) = blocks_sub.next().await {
        let block = block?;
        let block_number = block.header().number;
        let events = block.events().await?;
        let block_hash = events.block_hash();

        if let Some(authority_index) =
            get_authority_index(&scouty, Some(block_hash)).await?
        {
            // Event --> session::NewSession
            let event = events.find_first::<NewSession>()?;
            try_run_session_hooks(
                &scouty,
                event,
                &mut authority_records,
                block_number,
                authority_index,
                &mut para_records,
            )
            .await?;

            // Event --> staking::Slashed
            let event = events.find_first::<Slashed>()?;
            try_run_staking_slashed_hook(&scouty, event).await?;

            // Event --> staking::Chilled
            let event = events.find_first::<Chilled>()?;
            try_run_staking_chilled_hook(&scouty, event).await?;

            // Event --> im_online::SomeOffline
            let event = events.find_first::<SomeOffline>()?;
            try_run_im_online_some_offline_hook(&scouty, event).await?;

            // Event --> referenda::Submitted
            let event = events.find_first::<Submitted>()?;
            try_run_referenda_submitted_hook(&scouty, event).await?;

            // Track authority record
            authority_records.insert_record(block_number, Some(authority_index))?;
        }
    }
    // If subscription has closed for some reason await and subscribe again
    Err(ScoutyError::SubscriptionFinished)
}

async fn try_init_hook(
    scouty: &Scouty,
    authority_records: &AuthorityRecords,
    para_records: &ParaRecords,
) -> Result<(), ScoutyError> {
    let api = scouty.client().clone();
    let config = CONFIG.clone();

    // Get the current block number being processed
    let block_number_addr = node_runtime::storage().system().number();
    let block_number = api
        .storage()
        .at_latest()
        .await?
        .fetch(&block_number_addr)
        .await?
        .unwrap();

    // timestamp of current block
    let now_addr = node_runtime::storage().timestamp().now();
    let now = api
        .storage()
        .at_latest()
        .await?
        .fetch(&now_addr)
        .await?
        .unwrap();

    let init = Init { block_number, now };

    // Collect session data
    let current_session_index_addr = node_runtime::storage().session().current_index();
    let current_session_index = match api
        .storage()
        .at_latest()
        .await?
        .fetch(&current_session_index_addr)
        .await?
    {
        Some(index) => index,
        None => return Err("Current session index not defined".into()),
    };

    let session = collect_session_data(&scouty, current_session_index).await?;

    let network = Network::load(scouty.rpc()).await?;
    debug!("network {:?}", network);

    // Sync all nominators
    let all_nominators_map = if config.expose_all_nominators || config.expose_all {
        get_nominators(&scouty).await?
    } else {
        BTreeMap::new()
    };

    // Fetch era reward points from previous era
    let era_reward_points_addr = node_runtime::storage()
        .staking()
        .eras_reward_points(&(session.active_era_index - 1));
    let era_reward_points = api
        .storage()
        .at_latest()
        .await?
        .fetch(&era_reward_points_addr)
        .await?
        .unwrap();

    // Collect previusly era reward
    let era_reward_addr = node_runtime::storage()
        .staking()
        .eras_validator_reward(&(session.active_era_index - 1));
    let era_reward = api
        .storage()
        .at_latest()
        .await?
        .fetch(&era_reward_addr)
        .await?
        .unwrap();

    // Collect session active validators
    let validators_addr = node_runtime::storage().session().validators();
    let active_validators = api
        .storage()
        .at_latest()
        .await?
        .fetch(&validators_addr)
        .await?
        .unwrap();

    // Collect validators info based on config stashes
    let mut validators = collect_validators_data(&scouty).await?;

    // Try to run hooks for each stash
    for v in validators.iter_mut() {
        // Try HOOK_INIT
        let mut args = vec![
            convert_account_id(v.stash.clone()).to_string(),
            v.name.to_string(),
            format!("0x{:?}", HexDisplay::from(&v.queued_session_keys)),
            v.is_active.to_string(),
            v.is_queued.to_string(),
            session.active_era_index.to_string(),
            session.current_session_index.to_string(),
            session.eras_session_index.to_string(),
            block_number.to_string(),
        ];

        if config.expose_network || config.expose_all {
            args.push(network.name.to_string());
            args.push(network.token_symbol.to_string());
            args.push(network.token_decimals.to_string());
        } else {
            args.push("-".to_string());
            args.push("-".to_string());
            args.push("-".to_string());
        }

        if v.is_active && (config.expose_nominators || config.expose_all) {
            // get active nominators info
            let (
                total_active_stake,
                own_stake,
                active_nominators,
                active_nominators_stake,
            ) = get_active_nominators(&scouty, session.active_era_index, &v.stash)
                .await?;
            // calculate APR
            let apr = calculate_projected_apr(
                &scouty,
                &v.stash,
                network.token_decimals,
                total_active_stake,
                era_reward,
                active_validators.len().try_into().unwrap(),
            )
            .await?;
            //
            args.push(format!("{:.2}", apr * 100.0));
            args.push(total_active_stake.to_string());
            args.push(own_stake.to_string());
            args.push(active_nominators.join(",").to_string());
            args.push(
                active_nominators_stake
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
            );
        } else {
            args.push("-".to_string());
            args.push("-".to_string());
            args.push("-".to_string());
            args.push("-".to_string());
            args.push("-".to_string());
        }

        if v.is_active && (config.expose_authored_blocks || config.expose_all) {
            let current_session_total = authority_records.current_session_total(&v.stash);
            args.push(current_session_total.to_string());
            args.push("-".to_string());
        } else {
            args.push("-".to_string());
            args.push("-".to_string());
        }

        if config.expose_all_nominators || config.expose_all {
            if let Some(all_nominators) = all_nominators_map.get(&v.stash.to_string()) {
                let all_nominators_stashes = all_nominators
                    .iter()
                    .map(|(x, _, _)| x.to_string())
                    .collect::<Vec<String>>()
                    .join(",");

                let total_nominators_stake: u128 =
                    all_nominators.iter().map(|(_, x, _)| x).sum();

                let raw_nominees_stake: u128 =
                    all_nominators.iter().map(|(_, x, y)| x / *y as u128).sum();
                args.push(all_nominators_stashes.to_string());
                args.push(format!("{},{}", total_nominators_stake, raw_nominees_stake));
            } else {
                args.push("-".to_string());
                args.push("-".to_string());
            }
        } else {
            args.push("-".to_string());
            args.push("-".to_string());
        }

        if v.is_active && (config.expose_para_validator || config.expose_all) {
            let is_para_validator = para_records.is_para_validator(&v.stash);
            args.push(is_para_validator.to_string());
            args.push("-".to_string());
        } else {
            args.push("-".to_string());
            args.push("-".to_string());
        }

        if v.is_active && (config.expose_era_points || config.expose_all) {
            let points =
                get_validator_points_info(&v.stash, era_reward_points.clone()).await?;
            args.push(points.validator.to_string());
            args.push(points.era_avg.to_string());
        } else {
            args.push("-".to_string());
            args.push("-".to_string());
        }

        // Try run hook
        let hook = Hook::try_run(HOOK_INIT, &config.hook_init_path, args)?;
        v.hooks.push(hook);
    }

    // Prepare notification report
    debug!("validators {:?}", validators);

    let data = RawData {
        init,
        network,
        session,
        validators,
        section: Section::Init,
        ..Default::default()
    };

    let report = Report::from(data);
    scouty
        .send_message(&report.message(), &report.formatted_message())
        .await?;

    Ok(())
}

async fn try_run_staking_chilled_hook(
    scouty: &Scouty,
    event: Option<Chilled>,
) -> Result<(), ScoutyError> {
    if let Some(event) = event {
        let config = CONFIG.clone();

        // Collect validators info based on config stashes
        let mut validators = collect_validators_data(&scouty).await?;

        let network = Network::load(scouty.rpc()).await?;
        debug!("network {:?}", network);

        // Try to run hooks for each stash
        for v in validators.iter_mut() {
            // Identify if the stash has been chilled
            if event.stash == v.stash {
                v.is_chilled = true;

                // Try HOOK_VALIDATOR_CHILLED
                let mut args = vec![
                    convert_account_id(v.stash.clone()).to_string(),
                    v.name.to_string(),
                    format!("0x{:?}", HexDisplay::from(&v.queued_session_keys)),
                    v.is_active.to_string(),
                    v.is_queued.to_string(),
                ];

                if config.expose_network || config.expose_all {
                    args.push(network.name.to_string());
                    args.push(network.token_symbol.to_string());
                    args.push(network.token_decimals.to_string());
                } else {
                    args.push("-".to_string());
                    args.push("-".to_string());
                    args.push("-".to_string());
                }

                // Try run hook
                let hook = Hook::try_run(
                    HOOK_VALIDATOR_CHILLED,
                    &config.hook_validator_chilled_path,
                    args.clone(),
                )?;
                v.hooks.push(hook);
                break;
            }
        }

        debug!("validators {:?}", validators);

        // NOTE: Only send chilled message if the chilled account is
        // one of the stashes defined in config
        if validators.iter().any(|v| v.is_chilled) {
            // Prepare notification report
            let data = RawData {
                network,
                validators,
                section: Section::Chill,
                ..Default::default()
            };

            let report = Report::from(data);
            scouty
                .send_message(&report.message(), &report.formatted_message())
                .await?;
        }
    }

    Ok(())
}

async fn try_run_im_online_some_offline_hook(
    scouty: &Scouty,
    event: Option<SomeOffline>,
) -> Result<(), ScoutyError> {
    if let Some(event) = event {
        let config = CONFIG.clone();

        // Collect validators info based on config stashes
        let mut validators = collect_validators_data(&scouty).await?;

        let network = Network::load(scouty.rpc()).await?;
        debug!("network {:?}", network);

        // Try to run hooks for each stash
        for v in validators.iter_mut() {
            for (account_id, _exposure) in event.offline.iter() {
                if account_id == &v.stash {
                    v.is_offline = true;

                    // Try HOOK_VALIDATOR_OFFLINE
                    let mut args = vec![
                        convert_account_id(v.stash.clone()).to_string(),
                        v.name.to_string(),
                        format!("0x{:?}", HexDisplay::from(&v.queued_session_keys)),
                        v.is_active.to_string(),
                        v.is_queued.to_string(),
                    ];

                    if config.expose_network || config.expose_all {
                        args.push(network.name.to_string());
                        args.push(network.token_symbol.to_string());
                        args.push(network.token_decimals.to_string());
                    } else {
                        args.push("-".to_string());
                        args.push("-".to_string());
                        args.push("-".to_string());
                    }

                    // Try run hook
                    let hook = Hook::try_run(
                        HOOK_VALIDATOR_OFFLINE,
                        &config.hook_validator_offline_path,
                        args.clone(),
                    )?;
                    v.hooks.push(hook);
                    break;
                }
            }
        }

        debug!("validators {:?}", validators);

        // NOTE: Only send offline message if the offline account is
        // one of the stashes defined in config
        if validators.iter().any(|v| v.is_offline) {
            // Prepare notification report
            let data = RawData {
                network,
                validators,
                section: Section::Offline,
                ..Default::default()
            };

            let report = Report::from(data);
            scouty
                .send_message(&report.message(), &report.formatted_message())
                .await?;
        }
    }

    Ok(())
}

async fn try_run_staking_slashed_hook(
    scouty: &Scouty,
    event: Option<Slashed>,
) -> Result<(), ScoutyError> {
    if let Some(event) = event {
        let config = CONFIG.clone();

        // Collect validators info based on config stashes
        let mut validators = collect_validators_data(&scouty).await?;

        // Try to run hooks for each stash
        for v in validators.iter_mut() {
            if event.staker == v.stash {
                v.is_slashed = true;
            }
        }

        debug!("validators {:?}", validators);

        let network = Network::load(scouty.rpc()).await?;
        debug!("network {:?}", network);

        let mut args = vec![event.staker.to_string(), event.amount.to_string()];

        if config.expose_network || config.expose_all {
            args.push(network.name.to_string());
            args.push(network.token_symbol.to_string());
            args.push(network.token_decimals.to_string());
        } else {
            args.push("-".to_string());
            args.push("-".to_string());
            args.push("-".to_string());
        }

        // Try run hook
        let hook = Hook::try_run(
            HOOK_VALIDATOR_SLASHED,
            &config.hook_validator_slashed_path,
            args.clone(),
        )?;

        // Set slash info
        let slash = Slash {
            who: Some(event.staker),
            amount_value: event.amount,
            hook,
        };

        // Prepare notification report
        let data = RawData {
            network,
            validators,
            slash,
            section: Section::Slash,
            ..Default::default()
        };

        let report = Report::from(data);
        scouty
            .send_message(&report.message(), &report.formatted_message())
            .await?;
    }
    Ok(())
}

async fn try_run_referenda_submitted_hook(
    scouty: &Scouty,
    event: Option<Submitted>,
) -> Result<(), ScoutyError> {
    if let Some(event) = event {
        let config = CONFIG.clone();

        let network = Network::load(scouty.rpc()).await?;
        debug!("network {:?}", network);

        let mut args = vec![event.index.to_string(), event.track.to_string()];

        if config.expose_network || config.expose_all {
            args.push(network.name.to_string());
            args.push(network.token_symbol.to_string());
            args.push(network.token_decimals.to_string());
        } else {
            args.push("-".to_string());
            args.push("-".to_string());
            args.push("-".to_string());
        }

        // Try run hook
        let hook = Hook::try_run(
            HOOK_REFERENDA_SUBMITTED,
            &config.hook_referenda_submitted_path,
            args.clone(),
        )?;

        // Set referendum info
        let referendum = Referendum {
            index: event.index,
            track: event.track,
            hook,
        };

        // Prepare notification report
        let data = RawData {
            network,
            referendum,
            section: Section::Referenda,
            ..Default::default()
        };

        let report = Report::from(data);
        scouty
            .send_message(&report.message(), &report.formatted_message())
            .await?;
    }
    Ok(())
}

async fn try_run_session_hooks(
    scouty: &Scouty,
    event: Option<NewSession>,
    authority_records: &mut AuthorityRecords,
    block_number: u32,
    authority_index: AuthorityIndex,
    para_records: &mut ParaRecords,
) -> Result<(), ScoutyError> {
    if let Some(event) = event {
        let api = scouty.client().clone();
        let config = CONFIG.clone();

        // Collect session data
        let session = collect_session_data(&scouty, event.session_index).await?;

        // Collect session active validators
        let validators_addr = node_runtime::storage().session().validators();
        let active_validators = api
            .storage()
            .at_latest()
            .await?
            .fetch(&validators_addr)
            .await?
            .unwrap();

        // Authority records -->
        // Set a new authority set every new era in authority_records
        if (session.eras_session_index) == 1 {
            // Get current active authorities
            authority_records.set_authorities(active_validators.clone());
        }
        // Set a new session in authority_records
        authority_records.set_session(session.current_session_index);
        // Track authority record with the new session updated
        authority_records.insert_record(block_number, Some(authority_index))?;
        // Authority records <--

        // Para records -->
        // Set a new validator index for config stashes every new era in para_records
        if (session.eras_session_index) == 1 {
            para_records.reset_config_stashes(active_validators.clone())?;
        }
        // Track para record on a new session
        track_para_records(&scouty, session.current_session_index, para_records).await?;
        // Para records <--

        let network = Network::load(scouty.rpc()).await?;
        debug!("network {:?}", network);

        // Sync all nominators
        let all_nominators_map = if config.expose_all_nominators || config.expose_all {
            get_nominators(&scouty).await?
        } else {
            BTreeMap::new()
        };

        // Fetch era reward points from previous era
        let era_reward_points_addr = node_runtime::storage()
            .staking()
            .eras_reward_points(&(session.active_era_index - 1));
        let era_reward_points = api
            .storage()
            .at_latest()
            .await?
            .fetch(&era_reward_points_addr)
            .await?
            .unwrap();

        // Collect previusly era reward
        let era_reward_addr = node_runtime::storage()
            .staking()
            .eras_validator_reward(&(session.active_era_index - 1));
        let era_reward = api
            .storage()
            .at_latest()
            .await?
            .fetch(&era_reward_addr)
            .await?
            .unwrap();

        // Collect validators info based on config stashes
        let mut validators = collect_validators_data(&scouty).await?;

        // Try to run hooks for each stash
        for v in validators.iter_mut() {
            // Try HOOK_NEW_SESSION
            let mut args = vec![
                convert_account_id(v.stash.clone()).to_string(),
                v.name.to_string(),
                format!("0x{:?}", HexDisplay::from(&v.queued_session_keys)),
                v.is_active.to_string(),
                v.is_queued.to_string(),
                session.active_era_index.to_string(),
                session.current_session_index.to_string(),
                session.eras_session_index.to_string(),
                block_number.to_string(),
            ];

            if config.expose_network || config.expose_all {
                args.push(network.name.to_string());
                args.push(network.token_symbol.to_string());
                args.push(network.token_decimals.to_string());
            } else {
                args.push("-".to_string());
                args.push("-".to_string());
                args.push("-".to_string());
            }

            if v.is_active && (config.expose_nominators || config.expose_all) {
                let (total_active_stake, own_stake, nominators, nominators_stake) =
                    get_active_nominators(&scouty, session.active_era_index, &v.stash)
                        .await?;
                // calculate APR
                let apr = calculate_projected_apr(
                    &scouty,
                    &v.stash,
                    network.token_decimals,
                    total_active_stake,
                    era_reward,
                    active_validators.len().try_into().unwrap(),
                )
                .await?;
                args.push(format!("{:.2}", apr * 100.0));
                args.push(total_active_stake.to_string());
                args.push(own_stake.to_string());
                args.push(nominators.join(",").to_string());
                args.push(
                    nominators_stake
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(","),
                );
            } else {
                args.push("-".to_string());
                args.push("-".to_string());
                args.push("-".to_string());
                args.push("-".to_string());
                args.push("-".to_string());
            }

            if v.is_active && (config.expose_authored_blocks || config.expose_all) {
                let previous_session_total =
                    authority_records.previous_session_total(&v.stash);
                let previous_six_sessions_total =
                    authority_records.previous_six_sessions_total(&v.stash);
                args.push(previous_session_total.to_string());
                args.push(previous_six_sessions_total.to_string());
            } else {
                args.push("-".to_string());
                args.push("-".to_string());
            }

            if config.expose_all_nominators || config.expose_all {
                if let Some(all_nominators) = all_nominators_map.get(&v.stash.to_string())
                {
                    let all_nominators_stashes = all_nominators
                        .iter()
                        .map(|(x, _, _)| x.to_string())
                        .collect::<Vec<String>>()
                        .join(",");

                    let total_nominators_stake: u128 =
                        all_nominators.iter().map(|(_, x, _)| x).sum();

                    let raw_nominees_stake: u128 =
                        all_nominators.iter().map(|(_, x, y)| x / *y as u128).sum();
                    args.push(all_nominators_stashes.to_string());
                    args.push(format!(
                        "{},{}",
                        total_nominators_stake, raw_nominees_stake
                    ));
                }
            } else {
                args.push("-".to_string());
                args.push("-".to_string());
            }

            if v.is_active && (config.expose_para_validator || config.expose_all) {
                let is_para_validator = para_records.is_para_validator(&v.stash);
                let previous_six_sessions_total =
                    para_records.previous_six_sessions_total(&v.stash);
                args.push(is_para_validator.to_string());
                args.push(previous_six_sessions_total.to_string());
            } else {
                args.push("-".to_string());
                args.push("-".to_string());
            }

            // Try run hook
            let hook = Hook::try_run(
                HOOK_NEW_SESSION,
                &config.hook_new_session_path,
                args.clone(),
            )?;
            v.hooks.push(hook);

            // Try HOOK_NEW_ERA
            if (session.eras_session_index) == 1 {
                // Expose validator last era points
                if config.expose_era_points || config.expose_all {
                    let points =
                        get_validator_points_info(&v.stash, era_reward_points.clone())
                            .await?;
                    args.push(points.validator.to_string());
                    args.push((points.era_avg as u32).to_string());
                } else {
                    args.push("-".to_string());
                    args.push("-".to_string());
                }

                // Try run hook
                let hook =
                    Hook::try_run(HOOK_NEW_ERA, &config.hook_new_era_path, args.clone())?;
                v.hooks.push(hook);
            }

            if (session.eras_session_index) == 6 && session.queued_session_keys_changed {
                let next_era_index = session.active_era_index + 1;
                let next_session_index = session.current_session_index + 1;
                let mut args = vec![
                    convert_account_id(v.stash.clone()).to_string(),
                    v.name.to_string(),
                    format!("0x{:?}", HexDisplay::from(&v.queued_session_keys)),
                    format!("{}", next_era_index),
                    format!("{}", next_session_index),
                ];

                if config.expose_network || config.expose_all {
                    args.push(network.name.to_string());
                    args.push(network.token_symbol.to_string());
                    args.push(network.token_decimals.to_string());
                } else {
                    args.push("-".to_string());
                    args.push("-".to_string());
                    args.push("-".to_string());
                }

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

        let data = RawData {
            network,
            session,
            validators,
            section: Section::Session,
            ..Default::default()
        };

        let report = Report::from(data);
        scouty
            .send_message(&report.message(), &report.formatted_message())
            .await?;
    }
    Ok(())
}

async fn get_active_nominators(
    scouty: &Scouty,
    era_index: u32,
    stash: &AccountId32,
) -> Result<(u128, u128, Vec<String>, Vec<u128>), ScoutyError> {
    let api = scouty.client().clone();

    let exposure_addr = node_runtime::storage()
        .staking()
        .eras_stakers(&era_index, stash);
    match api
        .storage()
        .at_latest()
        .await?
        .fetch(&exposure_addr)
        .await?
    {
        Some(exposure) => {
            debug!("__exposure: {:?}", exposure);
            let mut nominators: Vec<String> = vec![];
            let mut nominators_stake: Vec<u128> = vec![];
            for other in exposure.others {
                // NOTE: convert nominator account to specific chain format
                nominators.push(convert_account_id(other.who).to_string());
                nominators_stake.push(other.value);
            }

            Ok((exposure.total, exposure.own, nominators, nominators_stake))
        }
        None => Ok((0, 0, vec![], vec![])),
    }
}

async fn get_nominators(
    scouty: &Scouty,
) -> Result<BTreeMap<String, Vec<(String, u128, u32)>>, ScoutyError> {
    let api = scouty.client().clone();
    let config = CONFIG.clone();

    // BTreeMap<String, Vec<(String, u128, u32)>> = validator_stash : [(nominator_stash, nominator_total_stake, number_of_nominations)]
    let mut stashes_nominators: BTreeMap<String, Vec<(String, u128, u32)>> =
        BTreeMap::new();
    for stash_str in config.stashes.iter() {
        let stash = AccountId32::from_str(stash_str).map_err(|e| {
            ScoutyError::Other(format!(
                "Invalid SS58 format account: {:?} error: {e:?}",
                stash_str
            ))
        })?;
        stashes_nominators.insert(stash.to_string(), vec![]);
    }

    info!("Starting syncing all nominators");
    let storage_query = node_runtime::storage().staking().nominators_iter();
    let mut results = api.storage().at_latest().await?.iter(storage_query).await?;
    while let Some(Ok((key, nominations))) = results.next().await {
        let nominator_stash = get_account_id_from_storage_key(key);
        let bonded_addr = node_runtime::storage()
            .staking()
            .bonded(&nominator_stash.clone());
        if let Some(controller) =
            api.storage().at_latest().await?.fetch(&bonded_addr).await?
        {
            let ledger_addr = node_runtime::storage().staking().ledger(&controller);
            let total_nominator_stake = if let Some(ledger) =
                api.storage().at_latest().await?.fetch(&ledger_addr).await?
            {
                ledger.total
            } else {
                0
            };

            for stash_str in config.stashes.iter() {
                let stash = AccountId32::from_str(stash_str).map_err(|e| {
                    ScoutyError::Other(format!(
                        "Invalid SS58 format account: {:?} error: {e:?}",
                        stash_str
                    ))
                })?;
                let BoundedVec(targets) = nominations.targets.clone();
                if targets.contains(&stash) {
                    if let Some(x) = stashes_nominators.get_mut(&stash.to_string()) {
                        // NOTE: convert account to chain specific format
                        x.push((
                            convert_account_id(nominator_stash.clone()).to_string(),
                            total_nominator_stake,
                            targets.len().try_into().unwrap(),
                        ));
                    }
                }
            }
        }
    }
    info!("Finished syncing all nominators");
    Ok(stashes_nominators)
}

async fn collect_session_data(
    scouty: &Scouty,
    _session_index: u32,
) -> Result<Session, ScoutyError> {
    let api = scouty.client().clone();

    // Get Era index
    let active_era_index_addr = node_runtime::storage().staking().active_era();
    let active_era_index = match api
        .storage()
        .at_latest()
        .await?
        .fetch(&active_era_index_addr)
        .await?
    {
        Some(info) => info.index,
        None => return Err("Current era index not defined".into()),
    };

    // Get current session
    let current_session_index_addr = node_runtime::storage().session().current_index();
    let current_session_index = api
        .storage()
        .at_latest()
        .await?
        .fetch(&current_session_index_addr)
        .await?
        .unwrap();

    // Get start session index
    let start_session_index_addr = node_runtime::storage()
        .staking()
        .eras_start_session_index(&active_era_index);
    let start_session_index = api
        .storage()
        .at_latest()
        .await?
        .fetch(&start_session_index_addr)
        .await?
        .unwrap();

    // Eras session index
    let eras_session_index = 1 + current_session_index - start_session_index;

    // Get session keys queued status
    let queued_changed_addr = node_runtime::storage().session().queued_changed();
    let queued_session_keys_changed = api
        .storage()
        .at_latest()
        .await?
        .fetch(&queued_changed_addr)
        .await?
        .unwrap();

    // Set network info
    let session = Session {
        active_era_index,
        current_session_index,
        eras_session_index,
        queued_session_keys_changed,
    };
    debug!("session {:?}", session);

    Ok(session)
}

async fn collect_validators_data(scouty: &Scouty) -> Result<Validators, ScoutyError> {
    let api = scouty.client().clone();
    let config = CONFIG.clone();

    // Verify session active validators
    let validators_addr = node_runtime::storage().session().validators();
    let active_validators = api
        .storage()
        .at_latest()
        .await?
        .fetch(&validators_addr)
        .await?
        .unwrap();

    // Verify session queued keys
    let queued_keys_addr = node_runtime::storage().session().queued_keys();
    let queued_keys = api
        .storage()
        .at_latest()
        .await?
        .fetch(&queued_keys_addr)
        .await?
        .unwrap();
    // let queued_keys = api.storage().session().queued_keys(None).await?;

    let mut validators: Validators = Vec::new();
    for stash_str in config.stashes.iter() {
        let stash = AccountId32::from_str(stash_str).map_err(|e| {
            ScoutyError::Other(format!(
                "Invalid SS58 format account: {:?} error: {e:?}",
                stash_str
            ))
        })?;
        let mut v = Validator::new(stash.clone());

        // Get validator name
        v.name = get_display_name(&scouty, &stash).await?;

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

async fn get_display_name(
    scouty: &Scouty,
    stash: &AccountId32,
) -> Result<String, ScoutyError> {
    if let Some(identity) = get_identity(&scouty, &stash, None).await? {
        return Ok(identity.to_string());
    } else {
        let s = &stash.to_string();
        Ok(format!("{}...{}", &s[..6], &s[s.len() - 6..]))
    }
}

#[async_recursion]
async fn get_identity(
    scouty: &Scouty,
    stash: &AccountId32,
    sub_account_name: Option<String>,
) -> Result<Option<Identity>, ScoutyError> {
    let api = scouty.client().clone();

    let identity_of_addr = node_runtime::storage().identity().identity_of(stash);
    match api
        .storage()
        .at_latest()
        .await?
        .fetch(&identity_of_addr)
        .await?
    {
        Some(identity) => {
            debug!("identity {:?}", identity);
            let parent = parse_identity_data(identity.info.display);
            let identity = match sub_account_name {
                Some(child) => Identity::with_name_and_sub(parent, child),
                None => Identity::with_name(parent),
            };
            Ok(Some(identity))
        }
        None => {
            let super_of_addr = node_runtime::storage().identity().super_of(stash);
            if let Some((parent_account, data)) = api
                .storage()
                .at_latest()
                .await?
                .fetch(&super_of_addr)
                .await?
            {
                let sub_account_name = parse_identity_data(data);
                return get_identity(
                    &scouty,
                    &parent_account,
                    Some(sub_account_name.to_string()),
                )
                .await;
            } else {
                Ok(None)
            }
        }
    }
}

//
fn parse_identity_data(data: Data) -> String {
    match data {
        Data::Raw0(bytes) => str(bytes.to_vec()),
        Data::Raw1(bytes) => str(bytes.to_vec()),
        Data::Raw2(bytes) => str(bytes.to_vec()),
        Data::Raw3(bytes) => str(bytes.to_vec()),
        Data::Raw4(bytes) => str(bytes.to_vec()),
        Data::Raw5(bytes) => str(bytes.to_vec()),
        Data::Raw6(bytes) => str(bytes.to_vec()),
        Data::Raw7(bytes) => str(bytes.to_vec()),
        Data::Raw8(bytes) => str(bytes.to_vec()),
        Data::Raw9(bytes) => str(bytes.to_vec()),
        Data::Raw10(bytes) => str(bytes.to_vec()),
        Data::Raw11(bytes) => str(bytes.to_vec()),
        Data::Raw12(bytes) => str(bytes.to_vec()),
        Data::Raw13(bytes) => str(bytes.to_vec()),
        Data::Raw14(bytes) => str(bytes.to_vec()),
        Data::Raw15(bytes) => str(bytes.to_vec()),
        Data::Raw16(bytes) => str(bytes.to_vec()),
        Data::Raw17(bytes) => str(bytes.to_vec()),
        Data::Raw18(bytes) => str(bytes.to_vec()),
        Data::Raw19(bytes) => str(bytes.to_vec()),
        Data::Raw20(bytes) => str(bytes.to_vec()),
        Data::Raw21(bytes) => str(bytes.to_vec()),
        Data::Raw22(bytes) => str(bytes.to_vec()),
        Data::Raw23(bytes) => str(bytes.to_vec()),
        Data::Raw24(bytes) => str(bytes.to_vec()),
        Data::Raw25(bytes) => str(bytes.to_vec()),
        Data::Raw26(bytes) => str(bytes.to_vec()),
        Data::Raw27(bytes) => str(bytes.to_vec()),
        Data::Raw28(bytes) => str(bytes.to_vec()),
        Data::Raw29(bytes) => str(bytes.to_vec()),
        Data::Raw30(bytes) => str(bytes.to_vec()),
        Data::Raw31(bytes) => str(bytes.to_vec()),
        Data::Raw32(bytes) => str(bytes.to_vec()),
        _ => format!("???"),
    }
}

fn str(bytes: Vec<u8>) -> String {
    format!("{}", String::from_utf8_lossy(&bytes))
}

async fn init_authority_records(
    scouty: &Scouty,
    authority_records: &mut AuthorityRecords,
) -> Result<(), ScoutyError> {
    let api = scouty.client().clone();
    let config = CONFIG.clone();
    // Get current block
    let number_addr = node_runtime::storage().system().number();
    let number = api
        .storage()
        .at_latest()
        .await?
        .fetch(&number_addr)
        .await?
        .unwrap();

    authority_records.set_block(number);

    // Get current session
    let current_session_index_addr = node_runtime::storage().session().current_index();
    let current_session_index = api
        .storage()
        .at_latest()
        .await?
        .fetch(&current_session_index_addr)
        .await?
        .unwrap();

    authority_records.set_session(current_session_index);

    // Get current active authorities
    let validators_addr = node_runtime::storage().session().validators();
    let active_validators = api
        .storage()
        .at_latest()
        .await?
        .fetch(&validators_addr)
        .await?
        .unwrap();

    authority_records.set_authorities(active_validators);

    // Get blocks authored for each stash
    for stash_str in config.stashes.iter() {
        let stash = AccountId32::from_str(stash_str).map_err(|e| {
            ScoutyError::Other(format!(
                "Invalid SS58 format account: {:?} error: {e:?}",
                stash_str
            ))
        })?;
        let key = format!("{}:{}", current_session_index, stash);
        let blocks_addr = node_runtime::storage()
            .im_online()
            .authored_blocks(&current_session_index, &stash);
        let blocks = if let Some(n) =
            api.storage().at_latest().await?.fetch(&blocks_addr).await?
        {
            n
        } else {
            0
        };

        authority_records.records.insert(key, blocks);
    }
    Ok(())
}

async fn init_para_records(
    scouty: &Scouty,
    para_records: &mut ParaRecords,
) -> Result<(), ScoutyError> {
    let api = scouty.client().clone();

    // Get current active authorities
    let validators_addr = node_runtime::storage().session().validators();
    let active_validators = api
        .storage()
        .at_latest()
        .await?
        .fetch(&validators_addr)
        .await?
        .unwrap();

    para_records.reset_config_stashes(active_validators)?;

    // Get current session
    let current_session_index_addr = node_runtime::storage().session().current_index();
    let current_session_index = api
        .storage()
        .at_latest()
        .await?
        .fetch(&current_session_index_addr)
        .await?
        .unwrap();

    track_para_records(&scouty, current_session_index, para_records).await?;

    Ok(())
}

async fn track_para_records(
    scouty: &Scouty,
    new_session_index: u32,
    para_records: &mut ParaRecords,
) -> Result<(), ScoutyError> {
    let api = scouty.client().clone();

    // Get para active validator indices
    let para_validators_addr = node_runtime::storage()
        .paras_shared()
        .active_validator_indices();
    let para_validators = api
        .storage()
        .at_latest()
        .await?
        .fetch(&para_validators_addr)
        .await?
        .unwrap();

    // Parse Vec<ValidatorIndex> to Vec<u32>
    let active_validator_indices: Vec<u32> = para_validators
        .iter()
        .map(|&ValidatorIndex(index)| index)
        .collect();

    // Insert record
    para_records.insert_record(new_session_index, active_validator_indices);

    Ok(())
}

async fn get_validator_points_info(
    stash: &AccountId32,
    era_reward_points: node_runtime::runtime_types::pallet_staking::EraRewardPoints<
        AccountId32,
    >,
) -> Result<Points, ScoutyError> {
    let stash_points = match era_reward_points
        .individual
        .iter()
        .find(|(s, _)| s == stash)
    {
        Some((_, p)) => *p,
        None => 0,
    };

    // Calculate average points
    let mut points: Vec<u32> = era_reward_points
        .individual
        .into_iter()
        .map(|(_, points)| points)
        .collect();

    let points_f64: Vec<f64> = points.iter().map(|points| *points as f64).collect();

    let points = Points {
        validator: stash_points,
        era_avg: stats::mean(&points_f64),
        ci99_9_interval: stats::confidence_interval_99_9(&points_f64),
        outlier_limits: stats::iqr_interval(&mut points),
    };

    Ok(points)
}

async fn calculate_projected_apr(
    scouty: &Scouty,
    stash: &AccountId32,
    token_decimals: u8,
    stash_active_stake: u128,
    era_reward: u128,
    total_active_validators: u32,
) -> Result<f64, ScoutyError> {
    let api = scouty.client().clone();

    // Get validator prefs
    let validator_prefs_addr = node_runtime::storage().staking().validators(stash);
    let validator_prefs = api
        .storage()
        .at_latest()
        .await?
        .fetch(&validator_prefs_addr)
        .await?
        .unwrap();

    let node_runtime::runtime_types::sp_arithmetic::per_things::Perbill(c) =
        validator_prefs.commission;
    let commission = normalize_commission(c);

    let avg_reward_per_validator_per_era =
        from_plancks_to_ksm(token_decimals, era_reward) / total_active_validators as f64;

    let nominators_reward = (1.0 - commission) * avg_reward_per_validator_per_era;
    let nominator_reward_per_ksm = (1.0_f64
        / from_plancks_to_ksm(token_decimals, stash_active_stake))
        * nominators_reward;
    let apr = nominator_reward_per_ksm * ERAS_PER_DAY as f64 * 365.0_f64;
    Ok(apr)
}

// async fn calculate_current_apr(scouty: &Scouty) -> Result<f64, ScoutyError> {
//     let client = scouty.client();
//     let api = scouty.client().clone();

//     // Get validator prefs
//     info!("Starting validators sync");
//     let history_depth: u32 = api.storage().staking().history_depth(None).await?;
//     let prefs = api
//         .storage()
//         .staking()
//         .validators(stash.clone(), None)
//         .await?;

//     let node_runtime::runtime_types::sp_arithmetic::per_things::Perbill(c) = prefs.commission;
//     let commission = normalize_commission(c);

//     let avg_reward_per_validator_per_era =
//         from_plancks_to_ksm(token_decimals, era_reward) / total_active_validators as f64;

//     let nominators_reward = (1.0 - commission) * avg_reward_per_validator_per_era;
//     let nominator_reward_per_ksm =
//         (1.0_f64 / from_plancks_to_ksm(token_decimals, stash_active_stake)) * nominators_reward;
//     let apr = nominator_reward_per_ksm * ERAS_PER_DAY as f64 * 365.0_f64;
//     Ok(apr)
// }

/// Normalize commission perbill between 0 - 1
fn normalize_commission(commission: u32) -> f64 {
    (commission as f64 / 10.0_f64.powi(9)) as f64
}

/// Convert Planks to KSM
fn from_plancks_to_ksm(token_decimals: u8, plancks: u128) -> f64 {
    (plancks as f64 / 10.0_f64.powi(token_decimals.into())) as f64
}

async fn get_authority_index(
    scouty: &Scouty,
    block_hash: Option<H256>,
) -> Result<Option<AuthorityIndex>, ScoutyError> {
    if let Some(header) = scouty.rpc().chain_get_header(block_hash).await? {
        match header.digest {
            Digest { logs } => {
                for digests in logs.iter() {
                    match digests {
                        DigestItem::PreRuntime(_, data) => {
                            if let Some(pre) = PreDigest::decode(&mut &data[..]).ok() {
                                match pre {
                                    PreDigest::Primary(e) => {
                                        return Ok(Some(e.authority_index))
                                    }
                                    PreDigest::SecondaryPlain(e) => {
                                        return Ok(Some(e.authority_index))
                                    }
                                    PreDigest::SecondaryVRF(e) => {
                                        return Ok(Some(e.authority_index))
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }
    Ok(None)
}
