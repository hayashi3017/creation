use async_trait::async_trait;
use creation_service::{
    model::relationship::{
        CreateRelationshipSchema, DeleteRelationshipSchema, DeleteRelationshipsForEntitySchema,
        DiagramRelationshipEdge, GetRelationshipsSchema, LoadRelationshipDiagramIdSchema,
        LoadRelationshipEdgesByDiagramIdsSchema, LoadRelationshipEdgesSchema, Relationship,
        RelationshipEdge, RelationshipEndpoints, RelationshipKind, UpdateRelationshipSchema,
        UpdatedRelationshipEndpoints,
    },
    repository::relationship::{
        CreateRelationshipRepositoryError, DeleteRelationshipRepositoryError,
        DeleteRelationshipsForEntityRepositoryError, GetRelationshipsRepositoryError,
        LoadRelationshipDiagramIdRepositoryError, LoadRelationshipEdgesByDiagramIdsRepositoryError,
        LoadRelationshipEdgesRepositoryError, ProvidesRelationshipRepository,
        RelationshipRepository, UpdateRelationshipRepositoryError, UsesRelationshipRepository,
    },
};
use sqlx::{Executor, Postgres};

use crate::{
    model::relationship::RelationshipTable,
    repository::{transaction::closed_transaction_error, RepositoryImpl},
};

#[async_trait]
impl UsesRelationshipRepository for RepositoryImpl<RelationshipTable> {
    async fn get_relationships(
        &self,
        body: GetRelationshipsSchema,
    ) -> Result<Vec<Relationship>, GetRelationshipsRepositoryError> {
        let relationships = sqlx::query_as::<_, RelationshipTable>(
            r#"
                SELECT
                    r.relationship_id,
                    r.diagram_id,
                    r.source_entity_id,
                    r.target_entity_id,
                    r.kind,
                    r.start_date,
                    r.end_date,
                    r.notes,
                    r.created_at,
                    r.updated_at,
                    r.deleted_at
                FROM relationship AS r
                INNER JOIN diagram AS d
                    ON d.diagram_id = r.diagram_id
                    AND d.deleted_at IS NULL
                WHERE
                    r.diagram_id = $1
                    AND r.deleted_at IS NULL
                ORDER BY r.relationship_id
            "#,
        )
        .bind(body.diagram_id as i64)
        .fetch_all(&self.pool.0)
        .await
        .map_err(GetRelationshipsRepositoryError::Db)?;

        Ok(relationships
            .into_iter()
            .map(|relationship| Relationship {
                relationship_id: relationship.relationship_id as usize,
                diagram_id: relationship.diagram_id as usize,
                source_entity_id: relationship.source_entity_id as usize,
                target_entity_id: relationship.target_entity_id as usize,
                kind: relationship.kind,
                start_date: relationship.start_date,
                end_date: relationship.end_date,
                notes: relationship.notes,
            })
            .collect())
    }

    async fn create_relationship(
        &self,
        body: CreateRelationshipSchema,
    ) -> Result<(), CreateRelationshipRepositoryError> {
        let created = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                create_relationship_with(tx.as_mut(), body)
                    .await
                    .map_err(CreateRelationshipRepositoryError::Db)?
            } else {
                return Err(CreateRelationshipRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            create_relationship_with(&self.pool.0, body)
                .await
                .map_err(CreateRelationshipRepositoryError::Db)?
        };

        if created {
            Ok(())
        } else {
            Err(CreateRelationshipRepositoryError::NotFound)
        }
    }

    async fn update_relationship(
        &self,
        body: UpdateRelationshipSchema,
    ) -> Result<UpdatedRelationshipEndpoints, UpdateRelationshipRepositoryError> {
        let endpoints = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                update_relationship_with(tx.as_mut(), body)
                    .await
                    .map_err(UpdateRelationshipRepositoryError::Db)?
            } else {
                return Err(UpdateRelationshipRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            update_relationship_with(&self.pool.0, body)
                .await
                .map_err(UpdateRelationshipRepositoryError::Db)?
        };

        match endpoints {
            Some(endpoints) => Ok(endpoints),
            None => Err(UpdateRelationshipRepositoryError::NotFound),
        }
    }

    async fn delete_relationship(
        &self,
        body: DeleteRelationshipSchema,
    ) -> Result<RelationshipEndpoints, DeleteRelationshipRepositoryError> {
        let endpoints = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                delete_relationship_with(tx.as_mut(), body)
                    .await
                    .map_err(DeleteRelationshipRepositoryError::Db)?
            } else {
                return Err(DeleteRelationshipRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            delete_relationship_with(&self.pool.0, body)
                .await
                .map_err(DeleteRelationshipRepositoryError::Db)?
        };

        match endpoints {
            Some(endpoints) => Ok(endpoints),
            None => Err(DeleteRelationshipRepositoryError::NotFound),
        }
    }

    async fn delete_relationships_for_entity(
        &self,
        body: DeleteRelationshipsForEntitySchema,
    ) -> Result<Vec<usize>, DeleteRelationshipsForEntityRepositoryError> {
        let entity_ids = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                delete_relationships_for_entity_with(tx.as_mut(), body)
                    .await
                    .map_err(DeleteRelationshipsForEntityRepositoryError::Db)?
            } else {
                return Err(DeleteRelationshipsForEntityRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            delete_relationships_for_entity_with(&self.pool.0, body)
                .await
                .map_err(DeleteRelationshipsForEntityRepositoryError::Db)?
        };

        Ok(entity_ids)
    }

    async fn load_relationship_diagram_id(
        &self,
        body: LoadRelationshipDiagramIdSchema,
    ) -> Result<Option<usize>, LoadRelationshipDiagramIdRepositoryError> {
        let diagram_id = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                load_relationship_diagram_id_with(tx.as_mut(), body)
                    .await
                    .map_err(LoadRelationshipDiagramIdRepositoryError::Db)?
            } else {
                return Err(LoadRelationshipDiagramIdRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            load_relationship_diagram_id_with(&self.pool.0, body)
                .await
                .map_err(LoadRelationshipDiagramIdRepositoryError::Db)?
        };

        Ok(diagram_id)
    }

    async fn load_relationship_edges(
        &self,
        body: LoadRelationshipEdgesSchema,
    ) -> Result<Vec<RelationshipEdge>, LoadRelationshipEdgesRepositoryError> {
        let edges = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                load_relationship_edges_with(tx.as_mut(), body)
                    .await
                    .map_err(LoadRelationshipEdgesRepositoryError::Db)?
            } else {
                return Err(LoadRelationshipEdgesRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            load_relationship_edges_with(&self.pool.0, body)
                .await
                .map_err(LoadRelationshipEdgesRepositoryError::Db)?
        };

        Ok(edges)
    }

    async fn load_relationship_edges_by_diagram_ids(
        &self,
        body: LoadRelationshipEdgesByDiagramIdsSchema,
    ) -> Result<Vec<DiagramRelationshipEdge>, LoadRelationshipEdgesByDiagramIdsRepositoryError>
    {
        let edges = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                load_relationship_edges_by_diagram_ids_with(tx.as_mut(), body)
                    .await
                    .map_err(LoadRelationshipEdgesByDiagramIdsRepositoryError::Db)?
            } else {
                return Err(LoadRelationshipEdgesByDiagramIdsRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            load_relationship_edges_by_diagram_ids_with(&self.pool.0, body)
                .await
                .map_err(LoadRelationshipEdgesByDiagramIdsRepositoryError::Db)?
        };

        Ok(edges)
    }
}

async fn create_relationship_with<'e, E>(
    executor: E,
    body: CreateRelationshipSchema,
) -> Result<bool, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let relationship_id = sqlx::query_scalar::<_, i64>(
        r#"
            INSERT INTO relationship
                (diagram_id, source_entity_id, target_entity_id, kind, start_date, end_date, notes)
            SELECT
                d.diagram_id, $2, $3, $4, $5, $6, $7
            FROM diagram AS d
            WHERE
                d.diagram_id = $1
                AND d.deleted_at IS NULL
                AND
                EXISTS (
                    SELECT 1
                    FROM entity AS source
                    WHERE
                        source.entity_id = $2
                        AND source.diagram_id = $1
                        AND source.deleted_at IS NULL
                )
                AND EXISTS (
                    SELECT 1
                    FROM entity AS target
                    WHERE
                        target.entity_id = $3
                        AND target.diagram_id = $1
                        AND target.deleted_at IS NULL
                )
            RETURNING relationship_id
        "#,
    )
    .bind(body.diagram_id as i64)
    .bind(body.source_entity_id as i64)
    .bind(body.target_entity_id as i64)
    .bind(body.kind)
    .bind(body.start_date)
    .bind(body.end_date)
    .bind(body.notes)
    .fetch_optional(executor)
    .await?;

    Ok(relationship_id.is_some())
}

async fn update_relationship_with<'e, E>(
    executor: E,
    body: UpdateRelationshipSchema,
) -> Result<Option<UpdatedRelationshipEndpoints>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as::<_, (i64, i64)>(
        r#"
            WITH previous AS (
                SELECT
                    r.relationship_id,
                    r.diagram_id,
                    r.source_entity_id,
                    r.target_entity_id
                FROM relationship AS r
                INNER JOIN diagram AS d
                    ON d.diagram_id = r.diagram_id
                    AND d.deleted_at IS NULL
                WHERE
                    r.relationship_id = $7
                    AND r.deleted_at IS NULL
            )
            UPDATE relationship AS r
            SET
                source_entity_id = $1,
                target_entity_id = $2,
                kind = $3,
                start_date = $4,
                end_date = $5,
                notes = $6,
                updated_at = now()
            FROM previous
            WHERE
                r.relationship_id = previous.relationship_id
                AND EXISTS (
                    SELECT 1
                    FROM entity AS source
                    WHERE
                        source.entity_id = $1
                        AND source.diagram_id = previous.diagram_id
                        AND source.deleted_at IS NULL
                )
                AND EXISTS (
                    SELECT 1
                    FROM entity AS target
                    WHERE
                        target.entity_id = $2
                        AND target.diagram_id = previous.diagram_id
                        AND target.deleted_at IS NULL
                )
            RETURNING previous.source_entity_id, previous.target_entity_id
        "#,
    )
    .bind(body.source_entity_id as i64)
    .bind(body.target_entity_id as i64)
    .bind(body.kind)
    .bind(body.start_date)
    .bind(body.end_date)
    .bind(body.notes)
    .bind(body.relationship_id as i64)
    .fetch_optional(executor)
    .await
    .map(|endpoints| {
        endpoints.map(|(previous_source_entity_id, previous_target_entity_id)| {
            UpdatedRelationshipEndpoints {
                previous_source_entity_id: previous_source_entity_id as usize,
                previous_target_entity_id: previous_target_entity_id as usize,
            }
        })
    })
}

async fn delete_relationship_with<'e, E>(
    executor: E,
    body: DeleteRelationshipSchema,
) -> Result<Option<RelationshipEndpoints>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as::<_, (i64, i64)>(
        r#"
            WITH active_relationship AS (
                SELECT
                    r.relationship_id,
                    r.source_entity_id,
                    r.target_entity_id
                FROM relationship AS r
                INNER JOIN diagram AS d
                    ON d.diagram_id = r.diagram_id
                    AND d.deleted_at IS NULL
                WHERE
                    r.relationship_id = $1
                    AND r.deleted_at IS NULL
            )
            UPDATE relationship AS r
            SET
                deleted_at = now(),
                updated_at = now()
            FROM active_relationship
            WHERE r.relationship_id = active_relationship.relationship_id
            RETURNING
                active_relationship.source_entity_id,
                active_relationship.target_entity_id
        "#,
    )
    .bind(body.relationship_id as i64)
    .fetch_optional(executor)
    .await
    .map(|endpoints| {
        endpoints.map(
            |(source_entity_id, target_entity_id)| RelationshipEndpoints {
                source_entity_id: source_entity_id as usize,
                target_entity_id: target_entity_id as usize,
            },
        )
    })
}

async fn delete_relationships_for_entity_with<'e, E>(
    executor: E,
    body: DeleteRelationshipsForEntitySchema,
) -> Result<Vec<usize>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let endpoints = sqlx::query_as::<_, (i64, i64)>(
        r#"
            UPDATE relationship
            SET
                deleted_at = now(),
                updated_at = now()
            WHERE
                deleted_at IS NULL
                AND (
                    source_entity_id = $1
                    OR target_entity_id = $1
                )
            RETURNING source_entity_id, target_entity_id
        "#,
    )
    .bind(body.entity_id as i64)
    .fetch_all(executor)
    .await?;

    Ok(endpoints
        .into_iter()
        .flat_map(|(source_entity_id, target_entity_id)| {
            [source_entity_id as usize, target_entity_id as usize]
        })
        .collect())
}

async fn load_relationship_diagram_id_with<'e, E>(
    executor: E,
    body: LoadRelationshipDiagramIdSchema,
) -> Result<Option<usize>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_scalar::<_, i64>(
        r#"
            SELECT diagram_id
            FROM relationship
            WHERE
                relationship_id = $1
                AND deleted_at IS NULL
        "#,
    )
    .bind(body.relationship_id as i64)
    .fetch_optional(executor)
    .await
    .map(|diagram_id| diagram_id.map(|diagram_id| diagram_id as usize))
}

async fn load_relationship_edges_with<'e, E>(
    executor: E,
    body: LoadRelationshipEdgesSchema,
) -> Result<Vec<RelationshipEdge>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let rows = sqlx::query_as::<_, (i64, i64, RelationshipKind)>(
        r#"
            SELECT
                r.source_entity_id,
                r.target_entity_id,
                r.kind
            FROM relationship AS r
            INNER JOIN diagram AS d
                ON d.diagram_id = r.diagram_id
                AND d.deleted_at IS NULL
            INNER JOIN entity AS source
                ON source.entity_id = r.source_entity_id
                AND source.diagram_id = r.diagram_id
                AND source.deleted_at IS NULL
            INNER JOIN entity AS target
                ON target.entity_id = r.target_entity_id
                AND target.diagram_id = r.diagram_id
                AND target.deleted_at IS NULL
            WHERE
                r.diagram_id = $1
                AND r.deleted_at IS NULL
            ORDER BY r.relationship_id
        "#,
    )
    .bind(body.diagram_id as i64)
    .fetch_all(executor)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|(source_entity_id, target_entity_id, kind)| match kind {
            RelationshipKind::Parent => Some(RelationshipEdge {
                ancestor_id: source_entity_id as usize,
                descendant_id: target_entity_id as usize,
            }),
            RelationshipKind::Child => Some(RelationshipEdge {
                ancestor_id: target_entity_id as usize,
                descendant_id: source_entity_id as usize,
            }),
            _ => None,
        })
        .collect())
}

async fn load_relationship_edges_by_diagram_ids_with<'e, E>(
    executor: E,
    body: LoadRelationshipEdgesByDiagramIdsSchema,
) -> Result<Vec<DiagramRelationshipEdge>, sqlx::Error>
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

    let rows = sqlx::query_as::<_, (i64, i64, i64, RelationshipKind)>(
        r#"
            SELECT
                r.diagram_id,
                r.source_entity_id,
                r.target_entity_id,
                r.kind
            FROM relationship AS r
            INNER JOIN diagram AS d
                ON d.diagram_id = r.diagram_id
                AND d.deleted_at IS NULL
            INNER JOIN entity AS source
                ON source.entity_id = r.source_entity_id
                AND source.diagram_id = r.diagram_id
                AND source.deleted_at IS NULL
            INNER JOIN entity AS target
                ON target.entity_id = r.target_entity_id
                AND target.diagram_id = r.diagram_id
                AND target.deleted_at IS NULL
            WHERE
                r.diagram_id = ANY($1)
                AND r.deleted_at IS NULL
            ORDER BY r.diagram_id, r.relationship_id
        "#,
    )
    .bind(diagram_ids)
    .fetch_all(executor)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(
            |(diagram_id, source_entity_id, target_entity_id, kind)| match kind {
                RelationshipKind::Parent => Some(DiagramRelationshipEdge {
                    diagram_id: diagram_id as usize,
                    ancestor_id: source_entity_id as usize,
                    descendant_id: target_entity_id as usize,
                }),
                RelationshipKind::Child => Some(DiagramRelationshipEdge {
                    diagram_id: diagram_id as usize,
                    ancestor_id: target_entity_id as usize,
                    descendant_id: source_entity_id as usize,
                }),
                _ => None,
            },
        )
        .collect())
}

impl RelationshipRepository for RepositoryImpl<RelationshipTable> {}

impl ProvidesRelationshipRepository for RepositoryImpl<RelationshipTable> {
    type T = Self;

    fn relationship_repository(&self) -> &Self::T {
        self
    }
}
