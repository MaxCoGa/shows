pub mod handlers;
pub mod middleware;
pub mod models;
pub mod registration;

use crate::user::{User, USERS};
use bcrypt::verify;
use self::models::Credentials;

#[derive(Debug)]
pub enum AuthError {
    UserNotFound,
    InvalidPassword,
}

pub fn authenticate(creds: &Credentials) -> Result<User, AuthError> {
    let user = USERS.get(&creds.username).ok_or(AuthError::UserNotFound)?;

    if verify(&creds.password, &user.password_hash).unwrap_or(false) {
        Ok(user.clone())
    } else {
        Err(AuthError::InvalidPassword)
    }
}
