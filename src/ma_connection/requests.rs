use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub requestType: String,
    pub username: String,
    pub password: String,
    pub session: i32,
    pub maxRequests: i32,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct SessionIdRequest {
    pub session: i32,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct PlaybacksRequst {
    pub requestType: String,
    pub startIndex: Vec<u32>,
    pub itemsCount: Vec<u32>,
    pub pageIndex: i32,
    pub itemsType: Vec<u32>,
    pub view: i32,
    pub execButtonViewMode: i32,
    pub buttonsViewMode: i32,
    pub session: i32
}
