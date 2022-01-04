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
use log::info;
use serde::Deserialize;
use subxt::sp_runtime::AccountId32;

#[derive(Debug)]
pub struct Network {
    pub name: String,
    pub active_era_index: u32,
    pub current_session_index: u32,
    pub eras_session_index: u32,
    pub queued_session_keys_changed: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct Hook {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub stdout: Vec<u8>,
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
    pub hooks: Vec<Hook>,
}

impl Validator {
    pub fn new(stash: AccountId32) -> Validator {
        Validator {
            stash,
            ..Default::default()
            // pub name: String,
            // is_active: false,
            // is_queued: false,
            // hooks: Vec::new(),
        }
    }
}

pub type Validators = Vec<Validator>;

#[derive(Debug)]
pub struct RawData {
    pub network: Network,
    pub validators: Validators,
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

impl From<RawData> for Report {
    /// Converts a Skipper `RawData` into a [`Report`].
    fn from(data: RawData) -> Report {
        let mut report = Report::new();
        // Skipper package
        report.add_raw_text(format!(
            "🤖 <code>{} v{}</code>",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ));

        // Network info
        report.add_break();
        report.add_raw_text(format!(
            "{} <b>{}</b> -> {} session ({}) of era {}",
            session_flag(data.network.eras_session_index),
            data.network.name,
            session_ordinal_number(data.network.eras_session_index),
            data.network.current_session_index,
            data.network.active_era_index
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
                report.add_text(format!("🪝 <code>{}</code>", hook.filename));

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
