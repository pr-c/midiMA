use serde::{Deserialize, Serialize};

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
