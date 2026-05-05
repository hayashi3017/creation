use async_trait::async_trait;
use creation_service::{
    model::entity::{
        CreateDiagramEntityMembershipSchema, CreateEntitySchema,
        DeleteDiagramEntityMembershipsSchema, DeleteEntitySchema, Entity, GetEntitiesSchema,
        LoadActiveEntitiesByDiagramIdsSchema, LoadActiveEntityIdsSchema,
        LoadEntitiesByDiagramIdsSchema, LoadEntitiesByWorldSchema, LoadSeedEntitiesSchema,
        SeedEntity, UpdateEntitySchema,
    },
    repository::entity::{
        CreateDiagramEntityMembershipRepositoryError, CreateEntityRepositoryError,
        DeleteDiagramEntityMembershipsRepositoryError, DeleteEntityRepositoryError,
        EntityRepository, GetEntitiesRepositoryError,
        LoadActiveEntitiesByDiagramIdsRepositoryError, LoadActiveEntityIdsRepositoryError,
        LoadEntitiesByDiagramIdsRepositoryError, LoadEntitiesByWorldRepositoryError,
        LoadSeedEntitiesRepositoryError, ProvidesEntityRepository, UpdateEntityRepositoryError,
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
                    e.entity_id,
                    de.diagram_id,
                    e.world_id,
                    e.kind,
                    e.name,
                    e.description,
                    e.created_at,
                    e.updated_at,
                    e.deleted_at
                FROM diagram_entity AS de
                INNER JOIN diagram AS d
                    ON d.diagram_id = de.diagram_id
                    AND d.deleted_at IS NULL
                INNER JOIN entity AS e
                    ON e.entity_id = de.entity_id
                    AND e.world_id = d.world_id
                    AND e.deleted_at IS NULL
                WHERE
                    de.deleted_at IS NULL
                    AND de.diagram_id = $1
                    AND d.world_id = $2
                ORDER BY e.entity_id
            "#,
        )
        .bind(body.diagram_id as i64)
        .bind(body.world_id as i64)
        .fetch_all(&self.pool.0)
        .await
        .map_err(GetEntitiesRepositoryError::Db)?;

        Ok(entities.into_iter().map(map_entity_table).collect())
    }

    async fn create_entity(
        &self,
        body: CreateEntitySchema,
    ) -> Result<usize, CreateEntityRepositoryError> {
        let entity_id = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                create_entity_with(tx.as_mut(), body)
                    .await
                    .map_err(CreateEntityRepositoryError::Db)?
            } else {
                return Err(CreateEntityRepositoryError::Db(closed_transaction_error()));
            }
        } else {
            create_entity_with(&self.pool.0, body)
                .await
                .map_err(CreateEntityRepositoryError::Db)?
        };

        entity_id.ok_or(CreateEntityRepositoryError::NotFound)
    }

    async fn create_diagram_entity_membership(
        &self,
        body: CreateDiagramEntityMembershipSchema,
    ) -> Result<(), CreateDiagramEntityMembershipRepositoryError> {
        let created = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                create_diagram_entity_membership_with(tx.as_mut(), body)
                    .await
                    .map_err(CreateDiagramEntityMembershipRepositoryError::Db)?
            } else {
                return Err(CreateDiagramEntityMembershipRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            create_diagram_entity_membership_with(&self.pool.0, body)
                .await
                .map_err(CreateDiagramEntityMembershipRepositoryError::Db)?
        };

        if created {
            Ok(())
        } else {
            Err(CreateDiagramEntityMembershipRepositoryError::NotFound)
        }
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

    async fn delete_diagram_entity_memberships(
        &self,
        body: DeleteDiagramEntityMembershipsSchema,
    ) -> Result<Vec<usize>, DeleteDiagramEntityMembershipsRepositoryError> {
        let entity_ids = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                delete_diagram_entity_memberships_with(tx.as_mut(), body)
                    .await
                    .map_err(DeleteDiagramEntityMembershipsRepositoryError::Db)?
            } else {
                return Err(DeleteDiagramEntityMembershipsRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            delete_diagram_entity_memberships_with(&self.pool.0, body)
                .await
                .map_err(DeleteDiagramEntityMembershipsRepositoryError::Db)?
        };

        Ok(entity_ids)
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

    async fn load_entities_by_diagram_ids(
        &self,
        body: LoadEntitiesByDiagramIdsSchema,
    ) -> Result<Vec<Entity>, LoadEntitiesByDiagramIdsRepositoryError> {
        let entities = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                load_entities_by_diagram_ids_with(tx.as_mut(), body)
                    .await
                    .map_err(LoadEntitiesByDiagramIdsRepositoryError::Db)?
            } else {
                return Err(LoadEntitiesByDiagramIdsRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            load_entities_by_diagram_ids_with(&self.pool.0, body)
                .await
                .map_err(LoadEntitiesByDiagramIdsRepositoryError::Db)?
        };

        Ok(entities)
    }

    async fn load_entities_by_world(
        &self,
        body: LoadEntitiesByWorldSchema,
    ) -> Result<Vec<Entity>, LoadEntitiesByWorldRepositoryError> {
        let entities = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                load_entities_by_world_with(tx.as_mut(), body)
                    .await
                    .map_err(LoadEntitiesByWorldRepositoryError::Db)?
            } else {
                return Err(LoadEntitiesByWorldRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            load_entities_by_world_with(&self.pool.0, body)
                .await
                .map_err(LoadEntitiesByWorldRepositoryError::Db)?
        };

        Ok(entities)
    }
}

async fn create_entity_with<'e, E>(
    executor: E,
    body: CreateEntitySchema,
) -> Result<Option<usize>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let entity_id = sqlx::query_scalar::<_, i64>(
        r#"
            INSERT INTO entity
                (world_id, kind, name, description)
            SELECT world_id, $2, $3, $4
            FROM world
            WHERE
                world_id = $1
                AND deleted_at IS NULL
            RETURNING entity_id
        "#,
    )
    .bind(body.world_id as i64)
    .bind(body.kind)
    .bind(body.name)
    .bind(body.description)
    .fetch_optional(executor)
    .await?;

    Ok(entity_id.map(|entity_id| entity_id as usize))
}

async fn create_diagram_entity_membership_with<'e, E>(
    executor: E,
    body: CreateDiagramEntityMembershipSchema,
) -> Result<bool, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let inserted = sqlx::query_scalar::<_, i64>(
        r#"
            INSERT INTO diagram_entity
                (diagram_id, entity_id)
            SELECT d.diagram_id, e.entity_id
            FROM diagram AS d
            INNER JOIN entity AS e
                ON e.entity_id = $2
                AND e.world_id = d.world_id
                AND e.deleted_at IS NULL
            WHERE
                d.diagram_id = $1
                AND d.deleted_at IS NULL
            ON CONFLICT (diagram_id, entity_id)
            DO UPDATE SET deleted_at = NULL
            RETURNING entity_id
        "#,
    )
    .bind(body.diagram_id as i64)
    .bind(body.entity_id as i64)
    .fetch_optional(executor)
    .await?;

    Ok(inserted.is_some())
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
                entity_id = $4
                AND world_id = $5
                AND deleted_at IS NULL
        "#,
    )
    .bind(body.kind)
    .bind(body.name)
    .bind(body.description)
    .bind(body.entity_id as i64)
    .bind(body.world_id as i64)
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
            WITH related_diagram AS (
                SELECT de.diagram_id
                FROM diagram_entity AS de
                WHERE
                    de.entity_id = $1
                    AND de.deleted_at IS NULL
                ORDER BY de.diagram_id
                LIMIT 1
            ),
            soft_deleted_entity AS (
                UPDATE entity
                SET
                    deleted_at = now(),
                    updated_at = now()
                WHERE
                    entity_id = $1
                    AND world_id = $2
                    AND deleted_at IS NULL
                RETURNING entity_id
            ),
            soft_deleted_membership AS (
                UPDATE diagram_entity
                SET deleted_at = now()
                WHERE
                    entity_id IN (SELECT entity_id FROM soft_deleted_entity)
                    AND deleted_at IS NULL
            )
            SELECT related_diagram.diagram_id
            FROM related_diagram
            WHERE EXISTS (SELECT 1 FROM soft_deleted_entity)
        "#,
    )
    .bind(body.entity_id as i64)
    .bind(body.world_id as i64)
    .fetch_optional(executor)
    .await?;

    Ok(diagram_id.map(|diagram_id| diagram_id as usize))
}

async fn delete_diagram_entity_memberships_with<'e, E>(
    executor: E,
    body: DeleteDiagramEntityMembershipsSchema,
) -> Result<Vec<usize>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let entity_ids = sqlx::query_scalar::<_, i64>(
        r#"
            UPDATE diagram_entity
            SET deleted_at = now()
            WHERE
                diagram_id = $1
                AND deleted_at IS NULL
            RETURNING entity_id
        "#,
    )
    .bind(body.diagram_id as i64)
    .fetch_all(executor)
    .await?;

    Ok(entity_ids.into_iter().map(|id| id as usize).collect())
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
            SELECT e.entity_id, de.diagram_id
            FROM entity AS e
            INNER JOIN diagram_entity AS de
                ON de.entity_id = e.entity_id
                AND de.deleted_at IS NULL
            WHERE e.entity_id = ANY($1)
            ORDER BY e.entity_id, de.diagram_id
        "#,
    )
    .bind(entity_ids)
    .fetch_all(executor)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(entity_id, diagram_id)| SeedEntity {
            entity_id: entity_id as usize,
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
            SELECT e.entity_id
            FROM diagram_entity AS de
            INNER JOIN entity AS e
                ON e.entity_id = de.entity_id
                AND e.deleted_at IS NULL
            WHERE
                de.diagram_id = $1
                AND de.deleted_at IS NULL
            ORDER BY e.entity_id
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
            SELECT e.entity_id, de.diagram_id
            FROM diagram_entity AS de
            INNER JOIN diagram AS d
                ON d.diagram_id = de.diagram_id
                AND d.deleted_at IS NULL
            INNER JOIN entity AS e
                ON e.entity_id = de.entity_id
                AND e.world_id = d.world_id
                AND e.deleted_at IS NULL
            WHERE
                de.diagram_id = ANY($1)
                AND de.deleted_at IS NULL
            ORDER BY de.diagram_id, e.entity_id
        "#,
    )
    .bind(diagram_ids)
    .fetch_all(executor)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(entity_id, diagram_id)| SeedEntity {
            entity_id: entity_id as usize,
            diagram_id: diagram_id as usize,
        })
        .collect())
}

async fn load_entities_by_diagram_ids_with<'e, E>(
    executor: E,
    body: LoadEntitiesByDiagramIdsSchema,
) -> Result<Vec<Entity>, sqlx::Error>
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

    let entities = sqlx::query_as::<_, EntityTable>(
        r#"
            SELECT
                e.entity_id,
                de.diagram_id,
                e.world_id,
                e.kind,
                e.name,
                e.description,
                e.created_at,
                e.updated_at,
                e.deleted_at
            FROM diagram_entity AS de
            INNER JOIN diagram AS d
                ON d.diagram_id = de.diagram_id
                AND d.deleted_at IS NULL
            INNER JOIN entity AS e
                ON e.entity_id = de.entity_id
                AND e.world_id = d.world_id
                AND e.deleted_at IS NULL
            WHERE
                de.diagram_id = ANY($1)
                AND de.deleted_at IS NULL
            ORDER BY de.diagram_id, e.entity_id
        "#,
    )
    .bind(diagram_ids)
    .fetch_all(executor)
    .await?;

    Ok(entities.into_iter().map(map_entity_table).collect())
}

async fn load_entities_by_world_with<'e, E>(
    executor: E,
    body: LoadEntitiesByWorldSchema,
) -> Result<Vec<Entity>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let entities = sqlx::query_as::<_, EntityTable>(
        r#"
            SELECT
                e.entity_id,
                0::BIGINT AS diagram_id,
                e.world_id,
                e.kind,
                e.name,
                e.description,
                e.created_at,
                e.updated_at,
                e.deleted_at
            FROM entity AS e
            WHERE
                e.world_id = $1
                AND e.deleted_at IS NULL
            ORDER BY e.entity_id
        "#,
    )
    .bind(body.world_id as i64)
    .fetch_all(executor)
    .await?;

    Ok(entities.into_iter().map(map_entity_table).collect())
}

fn map_entity_table(entity: EntityTable) -> Entity {
    Entity {
        entity_id: entity.entity_id as usize,
        diagram_id: entity.diagram_id as usize,
        kind: entity.kind,
        name: entity.name,
        description: entity.description,
    }
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
