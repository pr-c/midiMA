use super::objects::ItemGroup;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ResponseWithExplicitType {
    #[serde(rename = "responseType")]
    pub response_type: String,
}

//Server sometimes sends SessionIdResponse with worldIndex as a duplicate entry which makes it invalid json
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct SessionIdResponse {
    pub realtime: bool,
    pub session: i32,
    #[serde(rename = "forceLogin")]
    pub force_login: Option<bool>,
    #[serde(rename = "worldIndex")]
    pub world_index: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LoginRequestResponse {
    pub realtime: bool,
    #[serde(rename = "responseType")]
    pub response_type: String,
    pub result: bool,
    pub prompt: Option<String>,
    #[serde(rename = "promptcolor")]
    pub prompt_color: Option<String>,
    #[serde(rename = "worldIndex")]
    pub world_index: Option<i32>,
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

#[derive(Serialize, Deserialize)]
pub struct RealtimeResponse {
    pub realtime: bool,
}
