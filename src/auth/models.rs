use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NewUser {
    pub username: String,
    pub password: String,
}
