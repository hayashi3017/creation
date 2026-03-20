use async_trait::async_trait;
use creation_service::{
    model::entity::{
        CreateEntitySchema, DeleteEntitySchema, Entity, GetEntitiesSchema,
        LoadActiveEntitiesByDiagramIdsSchema, LoadActiveEntityIdsSchema, LoadSeedEntitiesSchema,
        SeedEntity, UpdateEntitySchema,
    },
    repository::entity::{
        CreateEntityRepositoryError, DeleteEntityRepositoryError, EntityRepository,
        GetEntitiesRepositoryError, LoadActiveEntitiesByDiagramIdsRepositoryError,
        LoadActiveEntityIdsRepositoryError, LoadSeedEntitiesRepositoryError,
        ProvidesEntityRepository, UpdateEntityRepositoryError, UsesEntityRepository,
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
    ) -> Result<usize, DeleteEntityRepositoryError> {
        let diagram_id = if let Some(shared_tx) = &self.tx {
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

        match diagram_id {
            Some(diagram_id) => Ok(diagram_id),
            None => Err(DeleteEntityRepositoryError::NotFound),
        }
    }

    async fn load_seed_entities(
        &self,
        body: LoadSeedEntitiesSchema,
    ) -> Result<Vec<SeedEntity>, LoadSeedEntitiesRepositoryError> {
        let seed_entities = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                load_seed_entities_with(tx.as_mut(), body)
                    .await
                    .map_err(LoadSeedEntitiesRepositoryError::Db)?
            } else {
                return Err(LoadSeedEntitiesRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            load_seed_entities_with(&self.pool.0, body)
                .await
                .map_err(LoadSeedEntitiesRepositoryError::Db)?
        };

        Ok(seed_entities)
    }

    async fn load_active_entity_ids(
        &self,
        body: LoadActiveEntityIdsSchema,
    ) -> Result<Vec<usize>, LoadActiveEntityIdsRepositoryError> {
        let entity_ids = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                load_active_entity_ids_with(tx.as_mut(), body)
                    .await
                    .map_err(LoadActiveEntityIdsRepositoryError::Db)?
            } else {
                return Err(LoadActiveEntityIdsRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            load_active_entity_ids_with(&self.pool.0, body)
                .await
                .map_err(LoadActiveEntityIdsRepositoryError::Db)?
        };

        Ok(entity_ids)
    }

    async fn load_active_entities_by_diagram_ids(
        &self,
        body: LoadActiveEntitiesByDiagramIdsSchema,
    ) -> Result<Vec<SeedEntity>, LoadActiveEntitiesByDiagramIdsRepositoryError> {
        let active_entities = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                load_active_entities_by_diagram_ids_with(tx.as_mut(), body)
                    .await
                    .map_err(LoadActiveEntitiesByDiagramIdsRepositoryError::Db)?
            } else {
                return Err(LoadActiveEntitiesByDiagramIdsRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            load_active_entities_by_diagram_ids_with(&self.pool.0, body)
                .await
                .map_err(LoadActiveEntitiesByDiagramIdsRepositoryError::Db)?
        };

        Ok(active_entities)
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
                kind = $1,
                name = $2,
                description = $3,
                updated_at = now()
            WHERE
                id = $4
                AND diagram_id = $5
                AND deleted_at IS NULL
        "#,
    )
    .bind(body.kind)
    .bind(body.name)
    .bind(body.description)
    .bind(body.id as i64)
    .bind(body.diagram_id as i64)
    .execute(executor)
    .await?;

    Ok(result.rows_affected())
}

async fn delete_entity_with<'e, E>(
    executor: E,
    body: DeleteEntitySchema,
) -> Result<Option<usize>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let diagram_id = sqlx::query_scalar::<_, i64>(
        r#"
            UPDATE entity
            SET
                deleted_at = now(),
                updated_at = now()
            WHERE
                id = $1
                AND deleted_at IS NULL
            RETURNING diagram_id
        "#,
    )
    .bind(body.id as i64)
    .fetch_optional(executor)
    .await?;

    Ok(diagram_id.map(|diagram_id| diagram_id as usize))
}

async fn load_seed_entities_with<'e, E>(
    executor: E,
    body: LoadSeedEntitiesSchema,
) -> Result<Vec<SeedEntity>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    if body.entity_ids.is_empty() {
        return Ok(Vec::new());
    }

    let entity_ids = body
        .entity_ids
        .into_iter()
        .map(|entity_id| entity_id as i64)
        .collect::<Vec<_>>();

    let rows = sqlx::query_as::<_, (i64, i64)>(
        r#"
            SELECT id, diagram_id
            FROM entity
            WHERE id = ANY($1)
            ORDER BY id
        "#,
    )
    .bind(entity_ids)
    .fetch_all(executor)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, diagram_id)| SeedEntity {
            id: id as usize,
            diagram_id: diagram_id as usize,
        })
        .collect())
}

async fn load_active_entity_ids_with<'e, E>(
    executor: E,
    body: LoadActiveEntityIdsSchema,
) -> Result<Vec<usize>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let entity_ids = sqlx::query_scalar::<_, i64>(
        r#"
            SELECT id
            FROM entity
            WHERE
                diagram_id = $1
                AND deleted_at IS NULL
            ORDER BY id
        "#,
    )
    .bind(body.diagram_id as i64)
    .fetch_all(executor)
    .await?;

    Ok(entity_ids.into_iter().map(|id| id as usize).collect())
}

async fn load_active_entities_by_diagram_ids_with<'e, E>(
    executor: E,
    body: LoadActiveEntitiesByDiagramIdsSchema,
) -> Result<Vec<SeedEntity>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    if body.diagram_ids.is_empty() {
        return Ok(Vec::new());
    }

    let diagram_ids = body
        .diagram_ids
        .into_iter()
        .map(|diagram_id| diagram_id as i64)
        .collect::<Vec<_>>();

    let rows = sqlx::query_as::<_, (i64, i64)>(
        r#"
            SELECT id, diagram_id
            FROM entity
            WHERE
                diagram_id = ANY($1)
                AND deleted_at IS NULL
            ORDER BY diagram_id, id
        "#,
    )
    .bind(diagram_ids)
    .fetch_all(executor)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, diagram_id)| SeedEntity {
            id: id as usize,
            diagram_id: diagram_id as usize,
        })
        .collect())
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
