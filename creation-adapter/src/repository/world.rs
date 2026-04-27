use async_trait::async_trait;
use creation_service::{
    model::world::{
        CreateWorldSchema, DeleteWorldSchema, GetWorldSchema, GetWorldsSchema, UpdateWorldSchema,
        World,
    },
    repository::world::{
        CreateWorldRepositoryError, DeleteWorldRepositoryError, GetWorldRepositoryError,
        GetWorldsRepositoryError, ProvidesWorldRepository, UpdateWorldRepositoryError,
        UsesWorldRepository, WorldRepository,
    },
    service::world::{ProvidesWorldService, WorldService},
};
use creation_usecase::usecase::world::{ProvidesWorldUsecase, WorldUsecase};
use sqlx::{Executor, Postgres};

use crate::{
    model::world::WorldTable,
    repository::{transaction::closed_transaction_error, RepositoryImpl},
};

#[async_trait]
impl UsesWorldRepository for RepositoryImpl<WorldTable> {
    async fn get_worlds(
        &self,
        _body: GetWorldsSchema,
    ) -> Result<Vec<World>, GetWorldsRepositoryError> {
        let worlds = sqlx::query_as::<_, WorldTable>(
            r#"
                SELECT world_id, name, description, created_at, updated_at, deleted_at
                FROM world
                WHERE deleted_at IS NULL
                ORDER BY updated_at DESC, world_id DESC
            "#,
        )
        .fetch_all(&self.pool.0)
        .await
        .map_err(GetWorldsRepositoryError::Db)?;

        Ok(worlds.into_iter().map(map_world_table).collect())
    }

    async fn get_world(
        &self,
        body: GetWorldSchema,
    ) -> Result<Option<World>, GetWorldRepositoryError> {
        let world = sqlx::query_as::<_, WorldTable>(
            r#"
                SELECT world_id, name, description, created_at, updated_at, deleted_at
                FROM world
                WHERE world_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(body.world_id as i64)
        .fetch_optional(&self.pool.0)
        .await
        .map_err(GetWorldRepositoryError::Db)?;

        Ok(world.map(map_world_table))
    }

    async fn create_world(
        &self,
        body: CreateWorldSchema,
    ) -> Result<World, CreateWorldRepositoryError> {
        let world = sqlx::query_as::<_, WorldTable>(
            r#"
                INSERT INTO world (name, description)
                VALUES ($1, $2)
                RETURNING world_id, name, description, created_at, updated_at, deleted_at
            "#,
        )
        .bind(body.name)
        .bind(body.description)
        .fetch_one(&self.pool.0)
        .await
        .map_err(CreateWorldRepositoryError::Db)?;

        Ok(map_world_table(world))
    }

    async fn update_world(
        &self,
        body: UpdateWorldSchema,
    ) -> Result<World, UpdateWorldRepositoryError> {
        let world = sqlx::query_as::<_, WorldTable>(
            r#"
                UPDATE world
                SET
                    name = $1,
                    description = $2,
                    updated_at = now()
                WHERE
                    world_id = $3
                    AND deleted_at IS NULL
                RETURNING world_id, name, description, created_at, updated_at, deleted_at
            "#,
        )
        .bind(body.name)
        .bind(body.description)
        .bind(body.world_id as i64)
        .fetch_optional(&self.pool.0)
        .await
        .map_err(UpdateWorldRepositoryError::Db)?;

        world
            .map(map_world_table)
            .ok_or(UpdateWorldRepositoryError::NotFound)
    }

    async fn delete_world(
        &self,
        body: DeleteWorldSchema,
    ) -> Result<(), DeleteWorldRepositoryError> {
        let deleted = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                delete_world_with(tx.as_mut(), body)
                    .await
                    .map_err(DeleteWorldRepositoryError::Db)?
            } else {
                return Err(DeleteWorldRepositoryError::Db(closed_transaction_error()));
            }
        } else {
            delete_world_with(&self.pool.0, body)
                .await
                .map_err(DeleteWorldRepositoryError::Db)?
        };

        if deleted {
            Ok(())
        } else {
            Err(DeleteWorldRepositoryError::NotFound)
        }
    }
}

async fn delete_world_with<'e, E>(executor: E, body: DeleteWorldSchema) -> Result<bool, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let deleted_world_id = sqlx::query_scalar::<_, i64>(
        r#"
            WITH active_world AS (
                SELECT world_id
                FROM world
                WHERE
                    world_id = $1
                    AND deleted_at IS NULL
                FOR UPDATE
            ),
            deleted_tree_path AS (
                DELETE FROM tree_path
                WHERE
                    ancestor_id IN (SELECT entity_id FROM entity WHERE world_id IN (SELECT world_id FROM active_world))
                    OR descendant_id IN (SELECT entity_id FROM entity WHERE world_id IN (SELECT world_id FROM active_world))
            ),
            deleted_relationship AS (
                UPDATE relationship
                SET
                    deleted_at = now(),
                    updated_at = now()
                WHERE
                    diagram_id IN (SELECT diagram_id FROM diagram WHERE world_id IN (SELECT world_id FROM active_world))
                    AND deleted_at IS NULL
            ),
            deleted_person AS (
                UPDATE person
                SET
                    deleted_at = now(),
                    updated_at = now()
                WHERE
                    entity_id IN (SELECT entity_id FROM entity WHERE world_id IN (SELECT world_id FROM active_world))
                    AND deleted_at IS NULL
            ),
            deleted_diagram_entity AS (
                UPDATE diagram_entity
                SET deleted_at = now()
                WHERE
                    diagram_id IN (SELECT diagram_id FROM diagram WHERE world_id IN (SELECT world_id FROM active_world))
                    AND deleted_at IS NULL
            ),
            deleted_entity AS (
                UPDATE entity
                SET
                    deleted_at = now(),
                    updated_at = now()
                WHERE
                    world_id IN (SELECT world_id FROM active_world)
                    AND deleted_at IS NULL
            ),
            deleted_diagram AS (
                UPDATE diagram
                SET
                    deleted_at = now(),
                    updated_at = now()
                WHERE
                    world_id IN (SELECT world_id FROM active_world)
                    AND deleted_at IS NULL
            ),
            deleted_world AS (
                UPDATE world
                SET
                    deleted_at = now(),
                    updated_at = now()
                WHERE world_id IN (SELECT world_id FROM active_world)
                RETURNING world_id
            )
            SELECT world_id FROM deleted_world
        "#,
    )
    .bind(body.world_id as i64)
    .fetch_optional(executor)
    .await?;

    Ok(deleted_world_id.is_some())
}

fn map_world_table(world: WorldTable) -> World {
    World {
        world_id: world.world_id as usize,
        name: world.name,
        description: world.description,
        created_at: world.created_at,
        updated_at: world.updated_at,
    }
}

impl WorldRepository for RepositoryImpl<WorldTable> {}
impl WorldService for RepositoryImpl<WorldTable> {}
impl WorldUsecase for RepositoryImpl<WorldTable> {}

impl ProvidesWorldRepository for RepositoryImpl<WorldTable> {
    type T = Self;

    fn world_repository(&self) -> &Self::T {
        self
    }
}

impl ProvidesWorldService for RepositoryImpl<WorldTable> {
    type T = Self;

    fn world_service(&self) -> &Self::T {
        self
    }
}

impl ProvidesWorldUsecase for RepositoryImpl<WorldTable> {
    type T = Self;

    fn world_usecase(&self) -> &Self::T {
        self
    }
}
