use std::marker::PhantomData;

use sqlx::{Pool, Postgres};

use crate::persistence::postgres::Db;

pub mod diagram;
pub mod user;

#[derive(Clone)]
pub struct RepositoryImpl<T> {
    pub pool: Db,
    pub _marker: PhantomData<T>,
}

impl<T> RepositoryImpl<T> {
    pub async fn new() -> Self {
        RepositoryImpl::<T> {
            pool: Db::new().await,
            _marker: PhantomData::<T>,
        }
    }
    pub async fn new_test(pool: Pool<Postgres>) -> Self {
        RepositoryImpl::<T> {
            pool: Db::new_test(pool).await,
            _marker: PhantomData::<T>,
        }
    }
}
