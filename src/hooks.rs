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

use crate::errors::ScoutyError;
use log::{info, warn};
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::{process::Command, process::Stdio, result::Result};

pub const HOOK_INIT: &'static str = "Scouty initialized";
pub const HOOK_NEW_SESSION: &'static str = "New session";
pub const HOOK_NEW_ERA: &'static str = "New era";
pub const HOOK_VALIDATOR_STARTS_ACTIVE_NEXT_ERA: &'static str =
    "Validator starts active next era";
pub const HOOK_VALIDATOR_STARTS_INACTIVE_NEXT_ERA: &'static str =
    "Validator starts inactive next era";
pub const HOOK_VALIDATOR_SLASHED: &'static str = "Validator has been slashed";
pub const HOOK_VALIDATOR_CHILLED: &'static str = "Validator has been chilled";
pub const HOOK_VALIDATOR_OFFLINE: &'static str = "Validator has been offline";
pub const HOOK_DEMOCRACY_STARTED: &'static str = "Democracy started";

#[derive(Debug, Deserialize, Default)]
pub struct Hook {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub filename_exists: bool,
    #[serde(default)]
    pub stdout: Vec<u8>,
}

impl Hook {
    pub fn try_run(
        name: &str,
        filename: &str,
        args: Vec<String>,
    ) -> Result<Hook, ScoutyError> {
        if Path::new(filename).exists() {
            info!("Run: {} {}", filename, args.join(" "));

            let mut stdout_formatted: Vec<u8> = Vec::new();

            let mut child = Command::new(filename)
                .args(args)
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;

            if let Some(child_stdout) = child.stdout.take() {
                let reader = BufReader::new(child_stdout);

                reader
                    .lines()
                    .filter_map(|line| line.ok())
                    .for_each(|line| {
                        info!("$ {}", line);
                        stdout_formatted
                            .extend(format!("{}\n", line).as_bytes().to_vec());
                    });

                let output = child.wait_with_output()?;

                if output.status.success() {
                    Ok(Hook {
                        name: name.to_string(),
                        filename: filename.to_string(),
                        filename_exists: true,
                        stdout: stdout_formatted,
                    })
                } else {
                    let err = String::from_utf8(output.stderr)?;
                    Err(ScoutyError::Other(format!(
                        "Hook script - {} - filename ({}) executed with error: {:?}",
                        name, filename, err
                    )))
                }
            } else {
                warn!(
                    "Hook script - {} - filename ({}) child stdout could not be captured",
                    name, filename
                );
                Err(ScoutyError::Other(format!(
                    "Hook script - {} - filename ({}) child stdout could not be captured",
                    name, filename
                )))
            }
        } else {
            warn!(
                "Hook script - {} - filename ({}) not defined",
                name, filename
            );
            Ok(Hook {
                name: name.to_string(),
                filename: filename.to_string(),
                filename_exists: false,
                stdout: vec![],
            })
        }
    }

    pub fn exists(name: &str, filename: &str) -> bool {
        if !Path::new(filename).exists() {
            warn!(
                "Hook script - {} - filename ({}) not defined",
                name, filename
            );
            return false;
        }
        return true;
    }
}
