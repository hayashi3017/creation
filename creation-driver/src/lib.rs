use config::Config;
use creation_adapter::{
    model::{
        diagram::DiagramTable, entity::EntityTable, person::PersonTable,
        relationship::RelationshipTable, tree_path::TreePathTable, user::UserTable,
    },
    persistence::postgres::Db,
    repository::{
        transaction::{new_shared_transaction, SharedTransaction},
        RepositoryImpl,
    },
};
use creation_service::{
    repository::{
        diagram::ProvidesDiagramRepository, entity::ProvidesEntityRepository,
        person::ProvidesPersonRepository, relationship::ProvidesRelationshipRepository,
        tree_path::ProvidesTreePathRepository, user::ProvidesUserRepository,
    },
    service::{
        diagram::{DiagramService, ProvidesDiagramService},
        entity::{EntityService, ProvidesEntityService},
        person::{PersonService, ProvidesPersonService},
        relationship::{ProvidesRelationshipService, RelationshipService},
        transaction::{
            BeginTransactionError, ProvidesTransactionManager, TransactionContext, TransactionError,
        },
        tree_path::{ProvidesTreePathService, TreePathService},
        user::{ProvidesUserService, UserService},
    },
};
use creation_usecase::usecase::{
    diagram::{DiagramUsecase, ProvidesDiagramUsecase},
    entity::{EntityUsecase, ProvidesEntityUsecase},
    family_tree::{FamilyTreeUsecase, ProvidesFamilyTreeUsecase},
    person::{PersonUsecase, ProvidesPersonUsecase},
    relationship::{ProvidesRelationshipUsecase, RelationshipUsecase},
    user::{ProvidesUserUsecase, UserUsecase},
};
use sqlx::{Pool, Postgres};

pub mod config;
mod handler;
mod jwt_auth;
pub mod middleware;
mod openapi;
mod response;
pub mod route;
pub mod utils;

pub struct AppState {
    pub driver: AppModule,
    pub env: Config,
}

#[derive(Clone)]
pub struct AppModule {
    pub db: Db,
    pub tx: Option<SharedTransaction>,
    pub user_repository: RepositoryImpl<UserTable>,
    pub diagram_repository: RepositoryImpl<DiagramTable>,
    pub entity_repository: RepositoryImpl<EntityTable>,
    pub person_repository: RepositoryImpl<PersonTable>,
    pub relationship_repository: RepositoryImpl<RelationshipTable>,
    pub tree_path_repository: RepositoryImpl<TreePathTable>,
}

impl AppModule {
    pub async fn new() -> Self {
        let db = Db::new().await;

        AppModule {
            db: db.clone(),
            tx: None,
            user_repository: RepositoryImpl::<UserTable>::from_db(db.clone()),
            diagram_repository: RepositoryImpl::<DiagramTable>::from_db(db.clone()),
            entity_repository: RepositoryImpl::<EntityTable>::from_db(db.clone()),
            person_repository: RepositoryImpl::<PersonTable>::from_db(db.clone()),
            relationship_repository: RepositoryImpl::<RelationshipTable>::from_db(db.clone()),
            tree_path_repository: RepositoryImpl::<TreePathTable>::from_db(db),
        }
    }

    pub async fn new_test(pool: Pool<Postgres>) -> Self {
        let db = Db::new_test(pool).await;

        AppModule {
            db: db.clone(),
            tx: None,
            user_repository: RepositoryImpl::<UserTable>::from_db(db.clone()),
            diagram_repository: RepositoryImpl::<DiagramTable>::from_db(db.clone()),
            entity_repository: RepositoryImpl::<EntityTable>::from_db(db.clone()),
            person_repository: RepositoryImpl::<PersonTable>::from_db(db.clone()),
            relationship_repository: RepositoryImpl::<RelationshipTable>::from_db(db.clone()),
            tree_path_repository: RepositoryImpl::<TreePathTable>::from_db(db),
        }
    }
}

impl ProvidesUserRepository for AppModule {
    type T = RepositoryImpl<UserTable>;

    fn user_repository(&self) -> &Self::T {
        &self.user_repository
    }
}

impl ProvidesDiagramRepository for AppModule {
    type T = RepositoryImpl<DiagramTable>;

    fn diagram_repository(&self) -> &Self::T {
        &self.diagram_repository
    }
}

impl ProvidesEntityRepository for AppModule {
    type T = RepositoryImpl<EntityTable>;

    fn entity_repository(&self) -> &Self::T {
        &self.entity_repository
    }
}

impl ProvidesPersonRepository for AppModule {
    type T = RepositoryImpl<PersonTable>;

    fn person_repository(&self) -> &Self::T {
        &self.person_repository
    }
}

impl ProvidesRelationshipRepository for AppModule {
    type T = RepositoryImpl<RelationshipTable>;

    fn relationship_repository(&self) -> &Self::T {
        &self.relationship_repository
    }
}

impl ProvidesTreePathRepository for AppModule {
    type T = RepositoryImpl<TreePathTable>;

    fn tree_path_repository(&self) -> &Self::T {
        &self.tree_path_repository
    }
}

#[async_trait::async_trait]
impl ProvidesTransactionManager for AppModule {
    type T = Self;

    async fn begin_transaction(&self) -> Result<Self::T, BeginTransactionError> {
        let shared_tx =
            new_shared_transaction(self.db.0.begin().await.map_err(BeginTransactionError::Db)?);

        Ok(AppModule {
            db: self.db.clone(),
            tx: Some(shared_tx.clone()),
            user_repository: RepositoryImpl::<UserTable>::from_db_with_transaction(
                self.db.clone(),
                shared_tx.clone(),
            ),
            diagram_repository: RepositoryImpl::<DiagramTable>::from_db_with_transaction(
                self.db.clone(),
                shared_tx.clone(),
            ),
            entity_repository: RepositoryImpl::<EntityTable>::from_db_with_transaction(
                self.db.clone(),
                shared_tx.clone(),
            ),
            person_repository: RepositoryImpl::<PersonTable>::from_db_with_transaction(
                self.db.clone(),
                shared_tx.clone(),
            ),
            relationship_repository: RepositoryImpl::<RelationshipTable>::from_db_with_transaction(
                self.db.clone(),
                shared_tx.clone(),
            ),
            tree_path_repository: RepositoryImpl::<TreePathTable>::from_db_with_transaction(
                self.db.clone(),
                shared_tx,
            ),
        })
    }
}

#[async_trait::async_trait]
impl TransactionContext for AppModule {
    async fn commit(self) -> Result<(), TransactionError> {
        let Some(shared_tx) = self.tx else {
            return Ok(());
        };

        let mut tx = shared_tx.lock().await;
        let Some(tx) = tx.take() else {
            return Ok(());
        };

        tx.commit().await.map_err(TransactionError::Db)
    }
}

impl UserService for AppModule {}
impl DiagramService for AppModule {}
impl EntityService for AppModule {}
impl PersonService for AppModule {}
impl RelationshipService for AppModule {}
impl TreePathService for AppModule {}

impl ProvidesUserService for AppModule {
    type T = Self;

    fn user_service(&self) -> &Self::T {
        self
    }
}

impl ProvidesDiagramService for AppModule {
    type T = Self;

    fn diagram_service(&self) -> &Self::T {
        self
    }
}

impl ProvidesEntityService for AppModule {
    type T = Self;

    fn entity_service(&self) -> &Self::T {
        self
    }
}

impl ProvidesPersonService for AppModule {
    type T = Self;

    fn person_service(&self) -> &Self::T {
        self
    }
}

impl ProvidesRelationshipService for AppModule {
    type T = Self;

    fn relationship_service(&self) -> &Self::T {
        self
    }
}

impl ProvidesTreePathService for AppModule {
    type T = Self;

    fn tree_path_service(&self) -> &Self::T {
        self
    }
}

impl UserUsecase for AppModule {}
impl DiagramUsecase for AppModule {}
impl EntityUsecase for AppModule {}
impl FamilyTreeUsecase for AppModule {}
impl PersonUsecase for AppModule {}
impl RelationshipUsecase for AppModule {}

impl ProvidesUserUsecase for AppModule {
    type T = Self;

    fn user_usecase(&self) -> &Self::T {
        self
    }
}

impl ProvidesDiagramUsecase for AppModule {
    type T = Self;

    fn diagram_usecase(&self) -> &Self::T {
        self
    }
}

impl ProvidesEntityUsecase for AppModule {
    type T = Self;

    fn entity_usecase(&self) -> &Self::T {
        self
    }
}

impl ProvidesFamilyTreeUsecase for AppModule {
    type T = Self;

    fn family_tree_usecase(&self) -> &Self::T {
        self
    }
}

impl ProvidesPersonUsecase for AppModule {
    type T = Self;

    fn person_usecase(&self) -> &Self::T {
        self
    }
}

impl ProvidesRelationshipUsecase for AppModule {
    type T = Self;

    fn relationship_usecase(&self) -> &Self::T {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::AppModule;
    use creation_service::{
        repository::{
            diagram::ProvidesDiagramRepository, entity::ProvidesEntityRepository,
            person::ProvidesPersonRepository, relationship::ProvidesRelationshipRepository,
            user::ProvidesUserRepository,
        },
        service::{
            diagram::ProvidesDiagramService, entity::ProvidesEntityService,
            person::ProvidesPersonService, relationship::ProvidesRelationshipService,
            user::ProvidesUserService,
        },
    };
    use creation_usecase::usecase::{
        diagram::UsesDiagramUsecase, entity::UsesEntityUsecase, family_tree::UsesFamilyTreeUsecase,
        person::UsesPersonUsecase, relationship::UsesRelationshipUsecase, user::UsesUserUsecase,
    };

    trait UsesMultipleRepositories:
        ProvidesUserRepository
        + ProvidesDiagramRepository
        + ProvidesEntityRepository
        + ProvidesPersonRepository
        + ProvidesRelationshipRepository
    {
    }
    impl<T> UsesMultipleRepositories for T where
        T: ProvidesUserRepository
            + ProvidesDiagramRepository
            + ProvidesEntityRepository
            + ProvidesPersonRepository
            + ProvidesRelationshipRepository
    {
    }

    trait UsesMultipleServices:
        ProvidesUserService
        + ProvidesDiagramService
        + ProvidesEntityService
        + ProvidesPersonService
        + ProvidesRelationshipService
    {
    }
    impl<T> UsesMultipleServices for T where
        T: ProvidesUserService
            + ProvidesDiagramService
            + ProvidesEntityService
            + ProvidesPersonService
            + ProvidesRelationshipService
    {
    }

    fn assert_module_supports_multi_dependencies<T>()
    where
        T: UsesMultipleRepositories
            + UsesMultipleServices
            + UsesUserUsecase
            + UsesDiagramUsecase
            + UsesEntityUsecase
            + UsesFamilyTreeUsecase
            + UsesPersonUsecase
            + UsesRelationshipUsecase,
    {
    }

    #[test]
    fn app_module_implements_multi_dependency_traits() {
        assert_module_supports_multi_dependencies::<AppModule>();
    }
}
