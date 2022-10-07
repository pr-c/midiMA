use crate::LoginCredentials;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::vec::Vec;
use crate::config::ButtonPosition;
use crate::ma_interface::ButtonValue;

#[derive(PartialEq)]
pub enum RequestType {
    Command,
    Login,
    Playbacks,
    Close,
}

impl FromStr for RequestType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "command" => Ok(RequestType::Command),
            "login" => Ok(RequestType::Login),
            "playbacks" => Ok(RequestType::Playbacks),
            "close" => Ok(RequestType::Close),
            _ => Err(()),
        }
    }
}

impl ToString for RequestType {
    fn to_string(&self) -> std::string::String {
        match self {
            RequestType::Playbacks => String::from("playbacks"),
            RequestType::Close => String::from("close"),
            RequestType::Login => String::from("login"),
            RequestType::Command => String::from("command"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    #[serde(rename = "requestType")]
    pub request_type: String,
    pub username: String,
    pub password: String,
    pub session: i32,
    #[serde(rename = "maxRequests")]
    pub max_requests: i32,
}

impl LoginRequest {
    pub fn new(login_credentials: &LoginCredentials, session: &i32) -> LoginRequest {
        LoginRequest {
            request_type: String::from("login"),
            max_requests: 10,
            username: login_credentials.username.clone(),
            password: login_credentials.password_hash.clone(),
            session: *session,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SessionIdRequest {
    pub session: i32,
}

impl SessionIdRequest {
    pub fn new(id: &i32) -> SessionIdRequest {
        SessionIdRequest { session: *id }
    }
}

impl SessionIdRequest {
    pub fn new_unknown_session() -> SessionIdRequest {
        SessionIdRequest { session: 0 }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PlaybacksRequest {
    #[serde(rename = "requestType")]
    pub request_type: String,
    #[serde(rename = "startIndex")]
    pub start_index: Vec<u32>,
    #[serde(rename = "itemsCount")]
    pub items_count: Vec<u32>,
    #[serde(rename = "pageIndex")]
    pub page_index: u32,
    #[serde(rename = "itemsType")]
    pub items_type: Vec<u32>,
    pub view: i32,
    #[serde(rename = "execButtonViewMode")]
    pub exec_button_view_mode: i32,
    #[serde(rename = "buttonsViewMode")]
    pub buttons_view_mode: i32,
    pub session: i32,
}

#[derive(Serialize, Deserialize)]
pub struct FaderInputRequest {
    #[serde(rename = "requestType")]
    request_type: String,
    #[serde(rename = "execIndex")]
    exec_index: u8,
    #[serde(rename = "pageIndex")]
    page_index: u32,
    #[serde(rename = "faderValue")]
    fader_value: f32,
    #[serde(rename = "type")]
    input_type: u32,
    session: i32,
}

impl FaderInputRequest {
    pub fn new(session: i32, exec_index: u8, page_index: u32, fader_value: f32) -> FaderInputRequest {
        FaderInputRequest {
            request_type: "playbacks_userInput".to_string(),
            exec_index,
            page_index,
            fader_value,
            input_type: 1,
            session,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ButtonInputRequest {
    #[serde(rename = "requestType")]
    pub request_type: String,
    #[serde(rename = "execIndex")]
    pub exec_index: u8,
    #[serde(rename = "pageIndex")]
    pub page_index: u32,
    pub cmdline: String,
    #[serde(rename = "buttonId")]
    pub button_id: u8,
    pub pressed: bool,
    pub released: bool,
    #[serde(rename = "type")]
    pub input_type: u32,
    pub session: i32,
}

impl ButtonInputRequest {
    pub fn new(session: i32, button: &ButtonValue, page_index: u32) -> Self {
        Self {
            request_type: "playbacks_userInput".to_string(),
            exec_index: button.exec_index,
            page_index,
            pressed: button.button_value,
            released: !button.button_value,
            input_type: 0,
            button_id: { match button.position {
                ButtonPosition::Top => 2,
                ButtonPosition::Mid => 1,
                ButtonPosition::Bottom => 0
            } },
            cmdline: "".to_string(),
            session
        }
    }
}