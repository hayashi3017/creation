use thiserror::Error;

pub type Result<T> = anyhow::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("hashing password")]
    HashingPassword(argon2::password_hash::Error),
    #[error("duplicate user")]
    DubpicateUser,
    #[error(transparent)]
    Any(#[from] anyhow::Error),
}

// TODO: 検証
// ref: https://github.com/http-rs/surf/issues/335#issuecomment-1025118151
impl From<argon2::password_hash::Error> for AppError {
    fn from(value: argon2::password_hash::Error) -> Self {
        AppError::HashingPassword(value)
    }
}
