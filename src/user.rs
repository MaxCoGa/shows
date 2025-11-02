use atomic_counter::{AtomicCounter, ConsistentCounter};
use bcrypt::{hash, DEFAULT_COST};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

// --- User Struct and Database ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    #[serde(skip)]
    pub password_hash: String,
}

pub static USER_ID_COUNTER: Lazy<ConsistentCounter> = Lazy::new(|| ConsistentCounter::new(2));
pub static USERS: Lazy<DashMap<String, User>> = Lazy::new(|| {
    let map = DashMap::new();
    let password_hash = hash("password", DEFAULT_COST).unwrap();
    map.insert(
        "admin".to_string(),
        User {
            id: 1,
            username: "admin".to_string(),
            password_hash,
        },
    );
    map
});

// --- New User Creation ---

#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub enum UserError {
    UsernameTaken,
}

pub fn create_user(new_user: NewUser) -> Result<(), UserError> {
    if USERS.contains_key(&new_user.username) {
        return Err(UserError::UsernameTaken);
    }

    let password_hash = hash(&new_user.password, DEFAULT_COST).unwrap();
    let user = User {
        id: USER_ID_COUNTER.inc() as u64,
        username: new_user.username.clone(),
        password_hash,
    };

    USERS.insert(new_user.username, user);

    Ok(())
}

pub fn find_user_by_id(user_id: u64) -> Option<User> {
    USERS
        .iter()
        .find(|entry| entry.value().id == user_id)
        .map(|entry| entry.value().clone())
}

// --- User Deletion ---

#[derive(Debug)]
pub enum UserDeleteError {
    UserNotFound,
}

pub fn delete_user(user_id: u64) -> Result<(), UserDeleteError> {
    let username = USERS
        .iter()
        .find(|entry| entry.value().id == user_id)
        .map(|entry| entry.key().clone());

    if let Some(username) = username {
        USERS.remove(&username);
        Ok(())
    } else {
        Err(UserDeleteError::UserNotFound)
    }
}
