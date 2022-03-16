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
use codec::Decode;
use log::debug;
use sp_consensus_babe::digests::PreDigest;
use std::{collections::BTreeMap, convert::TryInto, result::Result, str::FromStr};
use subxt::{
    rpc::ChainBlock,
    sp_runtime::{traits::Header, AccountId32, Digest, DigestItem},
    DefaultConfig,
};

pub type AuthorityIndex = u32;

pub fn decode_authority_index(
    chain_block: &ChainBlock<DefaultConfig>,
) -> Option<AuthorityIndex> {
    match chain_block.block.header.digest() {
        Digest { logs } => {
            for digests in logs.iter() {
                match digests {
                    DigestItem::PreRuntime(_, data) => {
                        if let Some(pre) = PreDigest::decode(&mut &data[..]).ok() {
                            return Some(pre.authority_index());
                        } else {
                            return None;
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    None
}

#[derive(Debug, Default)]
pub struct AuthorityRecords {
    last_block: u32,
    current_session_index: u32,
    authorities: Vec<AccountId32>,
    pub records: BTreeMap<String, u32>,
}

impl AuthorityRecords {
    pub fn new() -> Self {
        Self {
            last_block: 0,
            current_session_index: 0,
            authorities: vec![],
            records: BTreeMap::new(),
        }
    }

    pub fn set_block(&mut self, new_block: u32) {
        self.last_block = new_block;
    }

    pub fn set_session(&mut self, new_session_index: u32) {
        self.current_session_index = new_session_index;
    }

    pub fn set_authorities(&mut self, new_authorities: Vec<AccountId32>) {
        self.authorities = new_authorities;
    }

    pub fn insert_record(
        &mut self,
        block_number: u32,
        authority: Option<AuthorityIndex>,
    ) -> Result<(), ScoutyError> {
        let config = CONFIG.clone();
        if self.last_block != block_number {
            if let Some(authority_index) = authority {
                // Get author stash from authorities set
                let i: usize = authority_index.try_into().unwrap();
                if let Some(author_stash) = self.authorities.get(i) {
                    for stash_str in config.stashes.iter() {
                        let stash = AccountId32::from_str(stash_str)?;
                        if author_stash == &stash {
                            let key = format!(
                                "{}:{}",
                                self.current_session_index, author_stash
                            );
                            match self.records.get_mut(&key) {
                                Some(record) => {
                                    *record += 1;
                                }
                                None => {
                                    self.records.insert(key, 1);
                                }
                            };
                            // remove oldest entry if exists
                            self.remove(&stash);
                            // skip next entries
                            break;
                        }
                    }
                }
            }
            self.set_block(block_number);
        }
        debug!("records {:?}", self.records);

        Ok(())
    }

    pub fn current_session_total(&self, stash: &AccountId32) -> u32 {
        let key = format!("{}:{}", self.current_session_index, stash);
        match self.records.get(&key) {
            Some(record) => *record,
            None => 0,
        }
    }

    pub fn previous_session_total(&self, stash: &AccountId32) -> u32 {
        let session_index = self.current_session_index - 1;
        let key = format!("{}:{}", session_index, stash);
        match self.records.get(&key) {
            Some(record) => *record,
            None => 0,
        }
    }

    pub fn previous_six_sessions_total(&self, stash: &AccountId32) -> u32 {
        let mut total: u32 = 0;
        for n in 1..=6 {
            let session_index = self.current_session_index - n;
            let key = format!("{}:{}", session_index, stash);
            total += match self.records.get(&key) {
                Some(record) => *record,
                None => 0,
            };
        }
        return total;
    }

    fn remove(&mut self, stash: &AccountId32) {
        let session_index = self.current_session_index - 7;
        let key = format!("{}:{}", session_index, stash);
        self.records.remove(&key);
    }
}
