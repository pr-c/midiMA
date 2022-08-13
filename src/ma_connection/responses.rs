use super::objects::ItemGroup;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct SessionIdResponse {
    pub realtime: bool,
    pub session: i32,
    pub forceLogin: Option<bool>,
    pub worldIndex: Option<i32>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone)]
pub struct LoginRequestResponse {
    pub realtime: bool,
    pub responseType: String,
    pub result: bool,
    pub prompt: Option<String>,
    pub promptcolor: Option<String>,
    pub worldIndex: Option<i32>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ConnectResponse {
    pub status: String,
    pub appType: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct PlaybacksResponse {
    pub realtime: bool,
    pub responseType: String,
    pub responseSubType: i32,
    pub iPage: i32,
    pub itemGroups: Vec<ItemGroup>,
    pub worldIndex: Option<i32>,
}
