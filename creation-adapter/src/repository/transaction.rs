use std::sync::Arc;

use sqlx::{Postgres, Transaction as SqlxTransaction};
use tokio::sync::Mutex;

pub type SharedTransaction = Arc<Mutex<Option<SqlxTransaction<'static, Postgres>>>>;

pub fn new_shared_transaction(tx: SqlxTransaction<'static, Postgres>) -> SharedTransaction {
    Arc::new(Mutex::new(Some(tx)))
}

pub fn closed_transaction_error() -> sqlx::Error {
    sqlx::Error::Protocol("transaction already closed".to_string())
}
