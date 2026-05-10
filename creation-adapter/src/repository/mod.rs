use std::marker::PhantomData;

use sqlx::{Pool, Postgres, Transaction as SqlxTransaction};

use crate::persistence::postgres::Db;

use self::transaction::SharedTransaction;

pub mod diagram;
pub mod entity;
pub mod person;
pub mod relationship;
pub mod transaction;
pub mod tree_path;
pub mod user;
pub mod world;

macro_rules! impl_minimal_cake_bindings {
    (
        model = $model:ty,
        repository_trait = $repository_trait:path,
        provides_repository_trait = $provides_repository_trait:path,
        repository_getter = $repository_getter:ident,
        service_trait = $service_trait:path,
        provides_service_trait = $provides_service_trait:path,
        service_getter = $service_getter:ident,
    ) => {
        impl $repository_trait for $crate::repository::RepositoryImpl<$model> {}
        impl $service_trait for $crate::repository::RepositoryImpl<$model> {}

        impl $provides_repository_trait for $crate::repository::RepositoryImpl<$model> {
            type T = Self;
            fn $repository_getter(&self) -> &Self::T {
                self
            }
        }
        impl $provides_service_trait for $crate::repository::RepositoryImpl<$model> {
            type T = Self;
            fn $service_getter(&self) -> &Self::T {
                self
            }
        }
    };
    (
        model = $model:ty,
        repository_trait = $repository_trait:path,
        provides_repository_trait = $provides_repository_trait:path,
        repository_getter = $repository_getter:ident,
        service_trait = $service_trait:path,
        provides_service_trait = $provides_service_trait:path,
        service_getter = $service_getter:ident,
        usecase_trait = $usecase_trait:path,
        provides_usecase_trait = $provides_usecase_trait:path,
        usecase_getter = $usecase_getter:ident $(,)?
    ) => {
        impl $repository_trait for $crate::repository::RepositoryImpl<$model> {}
        impl $service_trait for $crate::repository::RepositoryImpl<$model> {}
        impl $usecase_trait for $crate::repository::RepositoryImpl<$model> {}

        impl $provides_repository_trait for $crate::repository::RepositoryImpl<$model> {
            type T = Self;
            fn $repository_getter(&self) -> &Self::T {
                self
            }
        }
        impl $provides_service_trait for $crate::repository::RepositoryImpl<$model> {
            type T = Self;
            fn $service_getter(&self) -> &Self::T {
                self
            }
        }
        impl $provides_usecase_trait for $crate::repository::RepositoryImpl<$model> {
            type T = Self;
            fn $usecase_getter(&self) -> &Self::T {
                self
            }
        }
    };
}

pub(crate) use impl_minimal_cake_bindings;

#[derive(Clone)]
pub struct RepositoryImpl<T> {
    pub pool: Db,
    pub tx: Option<SharedTransaction>,
    pub _marker: PhantomData<T>,
}

impl<T> RepositoryImpl<T> {
    pub async fn new() -> Self {
        Self::from_db(Db::new().await)
    }
    pub async fn new_test(pool: Pool<Postgres>) -> Self {
        Self::from_db(Db::new_test(pool).await)
    }

    pub fn from_db(pool: Db) -> Self {
        RepositoryImpl::<T> {
            pool,
            tx: None,
            _marker: PhantomData::<T>,
        }
    }

    pub fn from_db_with_transaction(pool: Db, tx: SharedTransaction) -> Self {
        RepositoryImpl::<T> {
            pool,
            tx: Some(tx),
            _marker: PhantomData::<T>,
        }
    }

    pub async fn begin(&self) -> Result<SqlxTransaction<'static, Postgres>, sqlx::Error> {
        self.pool.0.begin().await
    }

    pub async fn commit(tx: SqlxTransaction<'static, Postgres>) -> Result<(), sqlx::Error> {
        tx.commit().await
    }
}

#[cfg(test)]
mod tests {
    use super::impl_minimal_cake_bindings;

    struct DummyModel;

    trait DummyRepository {}
    trait DummyService {}
    trait DummyUsecase {}

    trait DummyProvidesRepository {
        type T: DummyRepository;
        fn dummy_repository(&self) -> &Self::T;
    }

    trait DummyProvidesService {
        type T: DummyService;
        fn dummy_service(&self) -> &Self::T;
    }

    trait DummyProvidesUsecase {
        type T: DummyUsecase;
        fn dummy_usecase(&self) -> &Self::T;
    }

    impl_minimal_cake_bindings!(
        model = DummyModel,
        repository_trait = DummyRepository,
        provides_repository_trait = DummyProvidesRepository,
        repository_getter = dummy_repository,
        service_trait = DummyService,
        provides_service_trait = DummyProvidesService,
        service_getter = dummy_service,
        usecase_trait = DummyUsecase,
        provides_usecase_trait = DummyProvidesUsecase,
        usecase_getter = dummy_usecase,
    );

    fn assert_all_traits<T>()
    where
        T: DummyRepository
            + DummyService
            + DummyUsecase
            + DummyProvidesRepository<T = T>
            + DummyProvidesService<T = T>
            + DummyProvidesUsecase<T = T>,
    {
    }

    #[test]
    fn impl_minimal_cake_bindings_wires_traits_and_getters() {
        type Target = crate::repository::RepositoryImpl<DummyModel>;

        assert_all_traits::<Target>();

        let repo_getter: fn(&Target) -> &Target =
            <Target as DummyProvidesRepository>::dummy_repository;
        let service_getter: fn(&Target) -> &Target =
            <Target as DummyProvidesService>::dummy_service;
        let usecase_getter: fn(&Target) -> &Target =
            <Target as DummyProvidesUsecase>::dummy_usecase;

        let _ = (repo_getter, service_getter, usecase_getter);
    }
}
