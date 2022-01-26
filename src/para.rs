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
use log::debug;
use std::{collections::BTreeMap, convert::TryInto, result::Result, str::FromStr};
use subxt::sp_runtime::AccountId32;

#[derive(Debug)]
pub struct ParaRecords<'a> {
    current_session_index: u32,
    config_stashes: Vec<(AccountId32, u32)>,
    pub records: &'a mut BTreeMap<String, bool>,
}

impl<'a> ParaRecords<'a> {
    pub fn new(records: &'a mut BTreeMap<String, bool>) -> Self {
        Self {
            current_session_index: 0,
            config_stashes: vec![],
            records,
        }
    }

    pub fn set_session(&mut self, new_session_index: u32) {
        self.current_session_index = new_session_index;
    }

    pub fn reset_config_stashes(
        &mut self,
        active_validators: Vec<AccountId32>,
    ) -> Result<(), ScoutyError> {
        let config = CONFIG.clone();

        let mut config_stashes: Vec<(AccountId32, u32)> = vec![];

        // Find stash indices
        for stash_str in config.stashes.iter() {
            let stash = AccountId32::from_str(stash_str)?;
            if let Some(index) = active_validators.iter().position(|x| x == &stash) {
                config_stashes.push((stash, index.try_into().unwrap()));
            };
        }

        self.config_stashes = config_stashes;

        Ok(())
    }

    pub fn insert_record(&mut self, new_session_index: u32, active_validator_indices: Vec<u32>) {
        if self.current_session_index != new_session_index {
            for (stash, index) in self.config_stashes.iter_mut() {
                // Add new entry
                let key = format!("{}:{}", new_session_index, stash);
                let is_para_validator: bool = active_validator_indices.contains(index);
                self.records.insert(key, is_para_validator);
                // Remove oldest entry if exists
                if self.current_session_index != 0 {
                    let session_index = self.current_session_index - 7;
                    let key = format!("{}:{}", session_index, stash);
                    self.records.remove(&key);
                }
            }
            self.set_session(new_session_index);
        }
        debug!("records {:?}", self.records);
    }

    pub fn is_para_validator(&self, stash: &AccountId32) -> bool {
        let key = format!("{}:{}", self.current_session_index, stash);
        match self.records.get(&key) {
            Some(record) => *record,
            None => false,
        }
    }

    pub fn previous_six_sessions_total(&self, stash: &AccountId32) -> u32 {
        let mut total: u32 = 0;
        for n in 1..=6 {
            let session_index = self.current_session_index - n;
            let key = format!("{}:{}", session_index, stash);
            total += match self.records.get(&key) {
                Some(true) => 1,
                Some(false) => 0,
                None => 0,
            };
        }
        return total;
    }
}
