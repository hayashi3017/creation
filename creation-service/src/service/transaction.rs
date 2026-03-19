use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BeginTransactionError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait TransactionContext: Send {
    async fn commit(self) -> Result<(), TransactionError>;
}

#[async_trait]
pub trait ProvidesTransactionManager: Send + Sync + 'static {
    type T: TransactionContext;

    async fn begin_transaction(&self) -> Result<Self::T, BeginTransactionError>;
}
