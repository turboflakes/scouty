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
use crate::hooks::Hook;
use log::info;
use serde::Deserialize;
use std::{convert::TryInto, result::Result};
use subxt::{sp_runtime::AccountId32, Client, DefaultConfig};

#[derive(Debug)]
pub struct Network {
    pub name: String,
    pub token_symbol: String,
    pub token_decimals: u8,
}

impl Network {
    pub async fn load(client: &Client<DefaultConfig>) -> Result<Network, ScoutyError> {
        let properties = client.properties();

        // Get Network name
        let chain_name = client.rpc().system_chain().await?;

        // Get Token symbol
        let token_symbol: String = if let Some(token_symbol) = properties.get("tokenSymbol") {
            token_symbol.as_str().unwrap_or_default().to_string()
        } else {
            "ND".to_string()
        };

        // Get Token decimals
        let token_decimals: u8 = if let Some(token_decimals) = properties.get("tokenDecimals") {
            token_decimals
                .as_u64()
                .unwrap_or_default()
                .try_into()
                .unwrap()
        } else {
            12
        };

        Ok(Network {
            name: chain_name,
            token_symbol,
            token_decimals,
        })
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct Session {
    pub active_era_index: u32,
    pub current_session_index: u32,
    pub eras_session_index: u32,
    pub queued_session_keys_changed: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct Validator {
    pub stash: AccountId32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub is_queued: bool,
    #[serde(default)]
    pub queued_session_keys: Vec<u8>,
    #[serde(default)]
    pub is_slashed: bool,
    #[serde(default)]
    pub is_chilled: bool,
    #[serde(default)]
    pub hooks: Vec<Hook>,
}

impl Validator {
    pub fn new(stash: AccountId32) -> Validator {
        Validator {
            stash,
            ..Default::default()
        }
    }
}

pub type Validators = Vec<Validator>;

#[derive(Debug, Deserialize, Default)]
pub struct Referendum {
    #[serde(default)]
    pub ref_index: u32,
    #[serde(default)]
    pub vote_threshold: String,
    #[serde(default)]
    pub hook: Hook,
}

#[derive(Debug, Deserialize, Default)]
pub struct Slash {
    #[serde(default)]
    pub who: AccountId32,
    #[serde(default)]
    pub amount_value: u128,
    #[serde(default)]
    pub hook: Hook,
}

#[derive(Debug)]
pub struct RawDataSession {
    pub network: Network,
    pub session: Session,
    pub validators: Validators,
}

#[derive(Debug)]
pub struct RawDataStaking {
    pub network: Network,
    pub validators: Validators,
    pub slash: Slash,
}

#[derive(Debug)]
pub struct RawDataDemocracy {
    pub network: Network,
    pub referendum: Referendum,
}

type Body = Vec<String>;

pub struct Report {
    body: Body,
    is_short: bool,
}

impl Report {
    pub fn new() -> Report {
        let config = CONFIG.clone();
        Report {
            body: Vec::new(),
            is_short: config.is_short,
        }
    }

    pub fn add_raw_text(&mut self, t: String) {
        self.body.push(t);
    }

    pub fn add_text(&mut self, t: String) {
        if !self.is_short {
            self.add_raw_text(t);
        }
    }
    pub fn add_break(&mut self) {
        self.add_raw_text("".into());
    }

    pub fn message(&self) -> String {
        self.body.join("\n")
    }

    pub fn formatted_message(&self) -> String {
        self.body.join("<br/>")
    }

    pub fn log(&self) {
        info!("__START__");
        for t in &self.body {
            info!("{}", t);
        }
        info!("__END__");
    }
}

impl From<RawDataSession> for Report {
    /// Converts a Scouty `RawData` into a [`Report`].
    fn from(data: RawDataSession) -> Report {
        let mut report = Report::new();
        // Scouty package
        report.add_raw_text(format!(
            "🤖 <code>{} v{}</code>",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ));

        // Network info
        report.add_break();
        report.add_raw_text(format!(
            "🔗 <b>{}</b> -> {} {} session ({}) of era {}",
            data.network.name,
            session_flag(data.session.eras_session_index),
            session_ordinal_number(data.session.eras_session_index),
            data.session.current_session_index,
            data.session.active_era_index
        ));

        // Validators info
        for validator in data.validators {
            report.add_break();

            let is_active_desc = if validator.is_active { "🟢" } else { "🔴" };
            report.add_raw_text(format!(
                "{} <b><a href=\"https://{}.subscan.io/validator/{}\">{}</a></b>",
                is_active_desc,
                data.network.name.to_lowercase(),
                validator.stash,
                validator.name,
            ));
            for hook in validator.hooks {
                let exists_desc = if !hook.filename_exists { "❌" } else { "" };
                report.add_text(format!("🪝 <code>{}</code> {}", hook.filename, exists_desc));

                let raw_output = String::from_utf8(hook.stdout).unwrap();
                // filter lines that start by special character '!'
                for line in raw_output.lines().filter(|line| line.starts_with("!")) {
                    report.add_raw_text(format!("‣ {}", line.strip_prefix("!").unwrap()));
                }
            }
        }

        report.add_break();

        report.add_raw_text("___".into());
        report.add_break();

        // Log report
        report.log();

        report
    }
}

impl From<RawDataDemocracy> for Report {
    /// Converts a Scouty `RawData` into a [`Report`].
    fn from(data: RawDataDemocracy) -> Report {
        let mut report = Report::new();
        // Scouty package
        report.add_raw_text(format!(
            "🤖 <code>{} v{}</code>",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ));

        // Network info
        report.add_break();
        report.add_raw_text(format!(
            "🔗 <b>{}</b> -> 🗳️ Referendum {} ({}) has begun.",
            data.network.name, data.referendum.ref_index, data.referendum.vote_threshold,
        ));

        report.add_break();
        report.add_raw_text(format!(
            "Vote here -> <a href=\"https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F{}.api.onfinality.io%2Fpublic-ws#/democracy\">Polkadot.js</a>",
            data.network.name.to_lowercase()
        ));
        report.add_raw_text(format!(
            "Or here -> <a href=\"https://{}.polkassembly.io/referendum/{}\">Polkassembly</a>",
            data.network.name.to_lowercase(),
            data.referendum.ref_index
        ));
        report.add_raw_text(format!(
            "Or here -> <a href=\"https://commonwealth.im/{}/proposal/referendum/{}\">Commonwealth</a>",
            data.network.name.to_lowercase(),
            data.referendum.ref_index
        ));

        // Hook
        report.add_break();
        let exists_desc = if !data.referendum.hook.filename_exists {
            "❌"
        } else {
            ""
        };
        report.add_text(format!(
            "🪝 <code>{}</code> {}",
            data.referendum.hook.filename, exists_desc
        ));

        let raw_output = String::from_utf8(data.referendum.hook.stdout).unwrap();
        // filter lines that start by special character '!'
        for line in raw_output.lines().filter(|line| line.starts_with("!")) {
            report.add_raw_text(format!("‣ {}", line.strip_prefix("!").unwrap()));
        }

        report.add_break();

        report.add_raw_text("___".into());
        report.add_break();

        // Log report
        report.log();

        report
    }
}

impl From<RawDataStaking> for Report {
    /// Converts a Scouty `RawData` into a [`Report`].
    fn from(data: RawDataStaking) -> Report {
        let mut report = Report::new();
        // Scouty package
        report.add_raw_text(format!(
            "🤖 <code>{} v{}</code>",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ));

        // Network info
        report.add_break();
        report.add_raw_text(format!(
            "🔗 <b>{}</b> -> <a href=\"https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F{}.api.onfinality.io%2Fpublic-ws#/staking/slashes\">🏴‍☠️ Slash occurred!</a>",
            data.network.name,
            data.network.name.to_lowercase(),
        ));

        let slashed_amount = format!(
            "{:.4} {}",
            (data.slash.amount_value) as f64 / 10f64.powi(data.network.token_decimals.into()),
            data.network.token_symbol
        );

        // Validators info
        for validator in data.validators {
            if validator.is_slashed {
                report.add_break();
                report.add_raw_text(format!(
                    "🤬 <b><a href=\"https://{}.subscan.io/validator/{}\">{}</a></b>",
                    data.network.name.to_lowercase(),
                    validator.stash,
                    validator.name,
                ));

                report.add_raw_text(format!("😱 Slashed amount -> 💸 <b>{}</b>", slashed_amount,));
            }
        }

        // Hook
        report.add_break();
        let exists_desc = if !data.slash.hook.filename_exists {
            "❌"
        } else {
            ""
        };
        report.add_text(format!(
            "🪝 <code>{}</code> {}",
            data.slash.hook.filename, exists_desc
        ));

        let raw_output = String::from_utf8(data.slash.hook.stdout).unwrap();
        // filter lines that start by special character '!'
        for line in raw_output.lines().filter(|line| line.starts_with("!")) {
            report.add_raw_text(format!("‣ {}", line.strip_prefix("!").unwrap()));
        }

        report.add_break();

        report.add_raw_text("___".into());
        report.add_break();

        // Log report
        report.log();

        report
    }
}

fn session_flag(index: u32) -> String {
    match index {
        1 => "🏁".to_string(),
        6 => "🏳️".to_string(),
        _ => "🚩".to_string(),
    }
}

fn session_ordinal_number(index: u32) -> String {
    match index {
        1 => "1st".to_string(),
        2 => "2nd".to_string(),
        3 => "3rd".to_string(),
        6 => "<b>last</b>".to_string(),
        _ => format!("{}th", index),
    }
}
