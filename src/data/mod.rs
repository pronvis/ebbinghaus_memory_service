use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUserRequest {
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUserResponse {
    pub user_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateMemoryRequest {
    pub user_id: i32,
    pub topic: Option<String>,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateMemoryResponse {
    pub memory_id: i32,
}
