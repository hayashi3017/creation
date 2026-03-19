use async_trait::async_trait;
use creation_service::{
    model::entity::{
        CreateEntitySchema, DeleteEntitySchema, Entity, GetEntitiesSchema, UpdateEntitySchema,
    },
    repository::entity::{
        CreateEntityRepositoryError, DeleteEntityRepositoryError, EntityRepository,
        GetEntitiesRepositoryError, ProvidesEntityRepository, UpdateEntityRepositoryError,
        UsesEntityRepository,
    },
    service::entity::{EntityService, ProvidesEntityService},
};
use creation_usecase::usecase::entity::{EntityUsecase, ProvidesEntityUsecase};
use sqlx::{Executor, Postgres};

use crate::{
    model::entity::EntityTable,
    repository::{transaction::closed_transaction_error, RepositoryImpl},
};

use super::impl_minimal_cake_bindings;

#[async_trait]
impl UsesEntityRepository for RepositoryImpl<EntityTable> {
    async fn get_entities(
        &self,
        body: GetEntitiesSchema,
    ) -> Result<Vec<Entity>, GetEntitiesRepositoryError> {
        let entities = sqlx::query_as::<_, EntityTable>(
            r#"
                SELECT
                    id,
                    diagram_id,
                    kind,
                    name,
                    description,
                    created_at,
                    updated_at,
                    deleted_at
                FROM entity
                WHERE
                    deleted_at IS NULL
                    AND diagram_id = $1
                ORDER BY id
            "#,
        )
        .bind(body.diagram_id as i64)
        .fetch_all(&self.pool.0)
        .await
        .map_err(GetEntitiesRepositoryError::Db)?;

        let ret: Vec<Entity> = entities
            .iter()
            .map(|entity| Entity {
                id: entity.id as usize,
                diagram_id: entity.diagram_id as usize,
                kind: entity.kind.clone(),
                name: entity.name.clone(),
                description: entity.description.clone(),
            })
            .collect();

        Ok(ret)
    }

    async fn create_entity(
        &self,
        body: CreateEntitySchema,
    ) -> Result<usize, CreateEntityRepositoryError> {
        if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                return create_entity_with(tx.as_mut(), body)
                    .await
                    .map_err(CreateEntityRepositoryError::Db);
            }

            return Err(CreateEntityRepositoryError::Db(closed_transaction_error()));
        }

        create_entity_with(&self.pool.0, body)
            .await
            .map_err(CreateEntityRepositoryError::Db)
    }

    async fn update_entity(
        &self,
        body: UpdateEntitySchema,
    ) -> Result<(), UpdateEntityRepositoryError> {
        let result = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                update_entity_with(tx.as_mut(), body)
                    .await
                    .map_err(UpdateEntityRepositoryError::Db)?
            } else {
                return Err(UpdateEntityRepositoryError::Db(closed_transaction_error()));
            }
        } else {
            update_entity_with(&self.pool.0, body)
                .await
                .map_err(UpdateEntityRepositoryError::Db)?
        };

        if result == 0 {
            return Err(UpdateEntityRepositoryError::NotFound);
        }

        Ok(())
    }

    async fn delete_entity(
        &self,
        body: DeleteEntitySchema,
    ) -> Result<(), DeleteEntityRepositoryError> {
        let result = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                delete_entity_with(tx.as_mut(), body)
                    .await
                    .map_err(DeleteEntityRepositoryError::Db)?
            } else {
                return Err(DeleteEntityRepositoryError::Db(closed_transaction_error()));
            }
        } else {
            delete_entity_with(&self.pool.0, body)
                .await
                .map_err(DeleteEntityRepositoryError::Db)?
        };

        if result == 0 {
            return Err(DeleteEntityRepositoryError::NotFound);
        }

        Ok(())
    }
}

async fn create_entity_with<'e, E>(
    executor: E,
    body: CreateEntitySchema,
) -> Result<usize, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let entity_id = sqlx::query_scalar::<_, i64>(
        r#"
            INSERT INTO entity
                (diagram_id, kind, name, description)
            VALUES ($1, $2, $3, $4)
            RETURNING id
        "#,
    )
    .bind(body.diagram_id as i64)
    .bind(body.kind)
    .bind(body.name)
    .bind(body.description)
    .fetch_one(executor)
    .await?;

    Ok(entity_id as usize)
}

async fn update_entity_with<'e, E>(
    executor: E,
    body: UpdateEntitySchema,
) -> Result<u64, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let result = sqlx::query(
        r#"
            UPDATE entity
            SET
                diagram_id = $1,
                kind = $2,
                name = $3,
                description = $4,
                updated_at = now()
            WHERE
                id = $5
                AND deleted_at IS NULL
        "#,
    )
    .bind(body.diagram_id as i64)
    .bind(body.kind)
    .bind(body.name)
    .bind(body.description)
    .bind(body.id as i64)
    .execute(executor)
    .await?;

    Ok(result.rows_affected())
}

async fn delete_entity_with<'e, E>(
    executor: E,
    body: DeleteEntitySchema,
) -> Result<u64, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let result = sqlx::query(
        r#"
            UPDATE entity
            SET
                deleted_at = now(),
                updated_at = now()
            WHERE
                id = $1
                AND deleted_at IS NULL
        "#,
    )
    .bind(body.id as i64)
    .execute(executor)
    .await?;

    Ok(result.rows_affected())
}

impl_minimal_cake_bindings!(
    model = EntityTable,
    repository_trait = EntityRepository,
    provides_repository_trait = ProvidesEntityRepository,
    repository_getter = entity_repository,
    service_trait = EntityService,
    provides_service_trait = ProvidesEntityService,
    service_getter = entity_service,
    usecase_trait = EntityUsecase,
    provides_usecase_trait = ProvidesEntityUsecase,
    usecase_getter = entity_usecase,
);
