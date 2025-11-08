use crate::database::DbPool;
use bcrypt::{hash, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// --- User Struct ---

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip)]
    pub password_hash: String,
}

// --- New User Creation ---

#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub enum UserError {
    UsernameTaken,
    DatabaseError(sqlx::Error),
}

impl From<sqlx::Error> for UserError {
    fn from(err: sqlx::Error) -> Self {
        UserError::DatabaseError(err)
    }
}

pub async fn create_user(pool: &DbPool, new_user: NewUser) -> Result<(), UserError> {
    let password_hash = hash(&new_user.password, DEFAULT_COST).unwrap();

    sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&new_user.username)
        .bind(&password_hash)
        .execute(pool)
        .await
        .map(|_| ()) // Discard the result, return a unit type on success
        .map_err(|e: sqlx::Error| {
            // Check if the error is a unique constraint violation
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return UserError::UsernameTaken;
                }
            }
            UserError::DatabaseError(e)
        })
}

pub async fn find_user_by_id(pool: &DbPool, user_id: i64) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

pub async fn find_user_by_username(pool: &DbPool, username: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}


// --- User Deletion ---

#[derive(Debug)]
pub enum UserDeleteError {
    UserNotFound,
    DatabaseError(sqlx::Error),
}

impl From<sqlx::Error> for UserDeleteError {
    fn from(err: sqlx::Error) -> Self {
        UserDeleteError::DatabaseError(err)
    }
}

pub async fn delete_user(pool: &DbPool, user_id: i64) -> Result<(), UserDeleteError> {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        Err(UserDeleteError::UserNotFound)
    } else {
        Ok(())
    }
}
