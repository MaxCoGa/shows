use crate::user::{create_user, NewUser, UserError};

/// # Errors
/// 
/// Returns `UserError` if the username is already taken.
pub fn register_user(new_user: NewUser) -> Result<(), UserError> {
    create_user(new_user)
}
