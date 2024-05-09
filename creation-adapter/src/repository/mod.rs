use std::marker::PhantomData;

use crate::persistence::mysql::Db;

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
}
