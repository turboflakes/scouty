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
use crate::errors::MatrixError;
use crate::runtimes::support::SupportedRuntime;
use async_recursion::async_recursion;
use base64::encode;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, result::Result, thread, time};
use url::form_urlencoded::byte_serialize;

const MATRIX_URL: &str = "https://matrix.org/_matrix/client/r0";

type AccessToken = String;
type RoomID = String;
type EventID = String;

#[derive(Deserialize, Debug, Default)]
struct Room {
    #[serde(default)]
    room_id: RoomID,
    #[serde(default)]
    servers: Vec<String>,
    #[serde(default)]
    room_alias: String,
    #[serde(default)]
    room_alias_name: String,
}

fn define_private_room_alias_name(
    pkg_name: &str,
    chain_name: &str,
    matrix_user: &str,
    matrix_bot_user: &str,
) -> String {
    encode(
        format!(
            "{}/{}/{}/{}",
            pkg_name, chain_name, matrix_user, matrix_bot_user
        )
        .as_bytes(),
    )
}

impl Room {
    fn new_private(chain: SupportedRuntime) -> Room {
        let config = CONFIG.clone();
        let room_alias_name = define_private_room_alias_name(
            env!("CARGO_PKG_NAME"),
            &chain.to_string(),
            &config.matrix_user,
            &config.matrix_bot_user,
        );
        let v: Vec<&str> = config.matrix_bot_user.split(":").collect();
        Room {
            room_alias_name: room_alias_name.to_string(),
            room_alias: format!("#{}:{}", room_alias_name.to_string(), v.last().unwrap()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    r#type: String,
    user: String,
    password: String,
}

#[derive(Deserialize, Debug)]
struct LoginResponse {
    user_id: String,
    access_token: AccessToken,
    home_server: String,
    device_id: String,
    // "well_known": {
    //   "m.homeserver": {
    //       "base_url": "https://matrix-client.matrix.org/"
    //   }
    // }
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateRoomRequest {
    name: String,
    room_alias_name: String,
    topic: String,
    preset: String,
    invite: Vec<String>,
    is_direct: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct SendRoomMessageRequest {
    msgtype: String,
    body: String,
    format: String,
    formatted_body: String,
}

#[derive(Deserialize, Debug)]
struct SendRoomMessageResponse {
    event_id: EventID,
}

#[derive(Deserialize, Debug)]
struct JoinedRoomsResponse {
    joined_rooms: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct ErrorResponse {
    errcode: String,
    error: String,
}

#[derive(Clone, Debug)]
pub struct Matrix {
    pub client: reqwest::Client,
    access_token: Option<String>,
    chain: SupportedRuntime,
    private_room_id: String,
    public_room_id: String,
    disabled: bool,
}

impl Default for Matrix {
    fn default() -> Matrix {
        Matrix {
            client: reqwest::Client::new(),
            access_token: None,
            chain: SupportedRuntime::Westend,
            private_room_id: String::from(""),
            public_room_id: String::from(""),
            disabled: false,
        }
    }
}

impl Matrix {
    pub fn new() -> Matrix {
        let config = CONFIG.clone();
        Matrix {
            disabled: config.matrix_disabled,
            ..Default::default()
        }
    }

    pub async fn login(&mut self) -> Result<(), MatrixError> {
        if self.disabled {
            return Ok(());
        }
        let config = CONFIG.clone();
        if let None = config.matrix_bot_user.find(":") {
            return Err(MatrixError::Other(format!("matrix bot user '{}' does specifed the matrix server e.g. '@your-own-scouty-bot-account:matrix.org'", config.matrix_bot_user)));
        }
        let client = self.client.clone();
        let req = LoginRequest {
            r#type: "m.login.password".to_string(),
            user: config.matrix_bot_user.to_string(),
            password: config.matrix_bot_password.to_string(),
        };

        let res = client
            .post(format!("{}/login", MATRIX_URL))
            .json(&req)
            .send()
            .await?;

        debug!("response {:?}", res);
        match res.status() {
            reqwest::StatusCode::OK => {
                let response = res.json::<LoginResponse>().await?;
                self.access_token = Some(response.access_token);
                info!(
                    "The 'Scouty Bot' user {} has been authenticated at {}",
                    response.user_id, response.home_server
                );
                Ok(())
            }
            _ => {
                let response = res.json::<ErrorResponse>().await?;
                Err(MatrixError::Other(response.error))
            }
        }
    }

    #[allow(dead_code)]
    pub async fn logout(&mut self) -> Result<(), MatrixError> {
        if self.disabled {
            return Ok(());
        }
        match &self.access_token {
            Some(access_token) => {
                let client = self.client.clone();
                let res = client
                    .post(format!(
                        "{}/logout?access_token={}",
                        MATRIX_URL, access_token
                    ))
                    .send()
                    .await?;
                debug!("response {:?}", res);
                match res.status() {
                    reqwest::StatusCode::OK => {
                        self.access_token = None;
                        Ok(())
                    }
                    _ => {
                        let response = res.json::<ErrorResponse>().await?;
                        Err(MatrixError::Other(response.error))
                    }
                }
            }
            None => Err(MatrixError::Other("access_token not defined".to_string())),
        }
    }

    // Login user, get or create private room
    pub async fn authenticate(&mut self, chain: SupportedRuntime) -> Result<(), MatrixError> {
        if self.disabled {
            return Ok(());
        }
        let config = CONFIG.clone();
        // Set chain
        self.chain = chain;
        // Login
        self.login().await?;
        // Get or create user private room
        if let Some(private_room) = self.get_or_create_private_room().await? {
            self.private_room_id = private_room.room_id;
            info!(
                "Messages will be sent to room {} (Private)",
                private_room.room_alias
            );
        }
        // Change Scouty Bot display name
        if !config.matrix_bot_display_name_disabled {
            self.change_bot_display_name().await?;
        }
        Ok(())
    }

    async fn change_bot_display_name(&self) -> Result<(), MatrixError> {
        match &self.access_token {
            Some(access_token) => {
                let config = CONFIG.clone();
                let client = self.client.clone();
                let v: Vec<&str> = config.matrix_user.split(":").collect();
                let username = v.first().unwrap();
                let display_name = format!("Scouty Bot ({})", &username[1..]);
                let mut data = HashMap::new();
                data.insert("displayname", &display_name);
                let user_id_encoded: String =
                    byte_serialize(config.matrix_bot_user.as_bytes()).collect();
                let res = client
                    .put(format!(
                        "{}/profile/{}/displayname?access_token={}",
                        MATRIX_URL, user_id_encoded, access_token
                    ))
                    .json(&data)
                    .send()
                    .await?;

                debug!("response {:?}", res);
                match res.status() {
                    reqwest::StatusCode::OK => {
                        info!("{} * Matrix bot display name changed", &display_name);
                        Ok(())
                    }
                    _ => {
                        let response = res.json::<ErrorResponse>().await?;
                        Err(MatrixError::Other(response.error))
                    }
                }
            }
            None => Err(MatrixError::Other("access_token not defined".to_string())),
        }
    }

    async fn get_room_id_by_room_alias(
        &self,
        room_alias: &str,
    ) -> Result<Option<RoomID>, MatrixError> {
        let client = self.client.clone();
        let room_alias_encoded: String = byte_serialize(room_alias.as_bytes()).collect();
        let res = client
            .get(format!(
                "{}/directory/room/{}",
                MATRIX_URL, room_alias_encoded
            ))
            .send()
            .await?;
        debug!("response {:?}", res);
        match res.status() {
            reqwest::StatusCode::OK => {
                let room = res.json::<Room>().await?;
                debug!("{} * Matrix room alias", room_alias);
                Ok(Some(room.room_id))
            }
            reqwest::StatusCode::NOT_FOUND => Ok(None),
            _ => {
                let response = res.json::<ErrorResponse>().await?;
                Err(MatrixError::Other(response.error))
            }
        }
    }

    async fn create_private_room(&self) -> Result<Option<Room>, MatrixError> {
        match &self.access_token {
            Some(access_token) => {
                let config = CONFIG.clone();
                let client = self.client.clone();
                let room: Room = Room::new_private(self.chain);
                let req = CreateRoomRequest {
                    name: format!("{} Scouty Bot (Private)", self.chain),
                    room_alias_name: room.room_alias_name.to_string(),
                    topic: "Scouty Bot <> Leading nodes every session".to_string(),
                    preset: "trusted_private_chat".to_string(),
                    invite: vec![config.matrix_user],
                    is_direct: true,
                };
                let res = client
                    .post(format!(
                        "{}/createRoom?access_token={}",
                        MATRIX_URL, access_token
                    ))
                    .json(&req)
                    .send()
                    .await?;

                debug!("response {:?}", res);
                match res.status() {
                    reqwest::StatusCode::OK => {
                        let mut r = res.json::<Room>().await?;
                        r.room_alias = room.room_alias;
                        r.room_alias_name = room.room_alias_name;
                        info!("{} * Matrix private room alias created", r.room_alias);
                        Ok(Some(r))
                    }
                    _ => {
                        let response = res.json::<ErrorResponse>().await?;
                        Err(MatrixError::Other(response.error))
                    }
                }
            }
            None => Err(MatrixError::Other("access_token not defined".to_string())),
        }
    }

    async fn get_or_create_private_room(&self) -> Result<Option<Room>, MatrixError> {
        match &self.access_token {
            Some(_) => {
                let mut room: Room = Room::new_private(self.chain);
                match self.get_room_id_by_room_alias(&room.room_alias).await? {
                    Some(room_id) => {
                        room.room_id = room_id;
                        Ok(Some(room))
                    }
                    None => Ok(self.create_private_room().await?),
                }
            }
            None => Err(MatrixError::Other("access_token not defined".to_string())),
        }
    }

    pub async fn send_message(
        &self,
        message: &str,
        formatted_message: &str,
    ) -> Result<(), MatrixError> {
        if self.disabled {
            return Ok(());
        }
        // Send message to private room (private assigned to the matrix_username in config)
        self.dispatch_message(&self.private_room_id, &message, &formatted_message)
            .await?;

        Ok(())
    }

    #[async_recursion]
    async fn dispatch_message(
        &self,
        room_id: &str,
        message: &str,
        formatted_message: &str,
    ) -> Result<Option<EventID>, MatrixError> {
        if self.disabled {
            return Ok(None);
        }
        match &self.access_token {
            Some(access_token) => {
                let client = self.client.clone();
                let req = SendRoomMessageRequest {
                    msgtype: "m.text".to_string(),
                    body: message.to_string(),
                    format: "org.matrix.custom.html".to_string(),
                    formatted_body: formatted_message.to_string(),
                };

                let res = client
                    .post(format!(
                        "{}/rooms/{}/send/m.room.message?access_token={}",
                        MATRIX_URL, room_id, access_token
                    ))
                    .json(&req)
                    .send()
                    .await?;

                debug!("response {:?}", res);
                match res.status() {
                    reqwest::StatusCode::OK => {
                        let response = res.json::<SendRoomMessageResponse>().await?;
                        debug!("{:?} * Matrix messsage dispatched", response);
                        Ok(Some(response.event_id))
                    }
                    reqwest::StatusCode::TOO_MANY_REQUESTS => {
                        let response = res.json::<ErrorResponse>().await?;
                        warn!("Matrix {} -> Wait 5 seconds and try again", response.error);
                        thread::sleep(time::Duration::from_secs(5));
                        return self
                            .dispatch_message(room_id, message, formatted_message)
                            .await;
                    }
                    _ => {
                        let response = res.json::<ErrorResponse>().await?;
                        Err(MatrixError::Other(response.error))
                    }
                }
            }
            None => Err(MatrixError::Other("access_token not defined".to_string())),
        }
    }
}
