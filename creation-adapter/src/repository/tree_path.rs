use async_trait::async_trait;
use creation_service::{
    model::tree_path::{
        CreateTreePathsSchema, DeleteTreePathsByEntityIdsSchema, LoadStaleRelatedConnectionsSchema,
        LoadStaleRelatedEntityIdsSchema, TreePathConnection,
    },
    repository::tree_path::{
        CreateTreePathsRepositoryError, DeleteTreePathsByEntityIdsRepositoryError,
        LoadStaleRelatedConnectionsRepositoryError, LoadStaleRelatedEntityIdsRepositoryError,
        ProvidesTreePathRepository, TreePathRepository, UsesTreePathRepository,
    },
};
use sqlx::{Executor, Postgres, QueryBuilder};

use crate::{
    model::tree_path::TreePathTable,
    repository::{transaction::closed_transaction_error, RepositoryImpl},
};

#[async_trait]
impl UsesTreePathRepository for RepositoryImpl<TreePathTable> {
    async fn load_stale_related_entity_ids(
        &self,
        body: LoadStaleRelatedEntityIdsSchema,
    ) -> Result<Vec<usize>, LoadStaleRelatedEntityIdsRepositoryError> {
        let entity_ids = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                load_stale_related_entity_ids_with(tx.as_mut(), body)
                    .await
                    .map_err(LoadStaleRelatedEntityIdsRepositoryError::Db)?
            } else {
                return Err(LoadStaleRelatedEntityIdsRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            load_stale_related_entity_ids_with(&self.pool.0, body)
                .await
                .map_err(LoadStaleRelatedEntityIdsRepositoryError::Db)?
        };

        Ok(entity_ids)
    }

    async fn load_stale_related_connections(
        &self,
        body: LoadStaleRelatedConnectionsSchema,
    ) -> Result<Vec<TreePathConnection>, LoadStaleRelatedConnectionsRepositoryError> {
        let connections = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                load_stale_related_connections_with(tx.as_mut(), body)
                    .await
                    .map_err(LoadStaleRelatedConnectionsRepositoryError::Db)?
            } else {
                return Err(LoadStaleRelatedConnectionsRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            load_stale_related_connections_with(&self.pool.0, body)
                .await
                .map_err(LoadStaleRelatedConnectionsRepositoryError::Db)?
        };

        Ok(connections)
    }

    async fn delete_tree_paths_by_entity_ids(
        &self,
        body: DeleteTreePathsByEntityIdsSchema,
    ) -> Result<(), DeleteTreePathsByEntityIdsRepositoryError> {
        if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                return delete_tree_paths_by_entity_ids_with(tx.as_mut(), body)
                    .await
                    .map_err(DeleteTreePathsByEntityIdsRepositoryError::Db);
            }

            return Err(DeleteTreePathsByEntityIdsRepositoryError::Db(
                closed_transaction_error(),
            ));
        }

        delete_tree_paths_by_entity_ids_with(&self.pool.0, body)
            .await
            .map_err(DeleteTreePathsByEntityIdsRepositoryError::Db)
    }

    async fn create_tree_paths(
        &self,
        body: CreateTreePathsSchema,
    ) -> Result<(), CreateTreePathsRepositoryError> {
        if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                return create_tree_paths_with(tx.as_mut(), body)
                    .await
                    .map_err(CreateTreePathsRepositoryError::Db);
            }

            return Err(CreateTreePathsRepositoryError::Db(
                closed_transaction_error(),
            ));
        }

        create_tree_paths_with(&self.pool.0, body)
            .await
            .map_err(CreateTreePathsRepositoryError::Db)
    }
}

async fn load_stale_related_entity_ids_with<'e, E>(
    executor: E,
    body: LoadStaleRelatedEntityIdsSchema,
) -> Result<Vec<usize>, sqlx::Error>
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
            SELECT ancestor_id, descendant_id
            FROM tree_path
            WHERE
                ancestor_id = ANY($1)
                OR descendant_id = ANY($1)
        "#,
    )
    .bind(entity_ids)
    .fetch_all(executor)
    .await?;

    Ok(rows
        .into_iter()
        .flat_map(|(ancestor_id, descendant_id)| [ancestor_id as usize, descendant_id as usize])
        .collect())
}

async fn delete_tree_paths_by_entity_ids_with<'e, E>(
    executor: E,
    body: DeleteTreePathsByEntityIdsSchema,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    if body.entity_ids.is_empty() {
        return Ok(());
    }

    let mut builder = QueryBuilder::<Postgres>::new("DELETE FROM tree_path WHERE ancestor_id IN (");
    {
        let mut separated = builder.separated(", ");
        for entity_id in body.entity_ids.iter() {
            separated.push_bind(*entity_id as i64);
        }
    }
    builder.push(") AND descendant_id IN (");
    {
        let mut separated = builder.separated(", ");
        for entity_id in body.entity_ids.iter() {
            separated.push_bind(*entity_id as i64);
        }
    }
    builder.push(")");

    builder.build().execute(executor).await?;

    Ok(())
}

async fn load_stale_related_connections_with<'e, E>(
    executor: E,
    body: LoadStaleRelatedConnectionsSchema,
) -> Result<Vec<TreePathConnection>, sqlx::Error>
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
            SELECT ancestor_id, descendant_id
            FROM tree_path
            WHERE
                ancestor_id = ANY($1)
                OR descendant_id = ANY($1)
        "#,
    )
    .bind(entity_ids)
    .fetch_all(executor)
    .await?;

    // service 側で seed と同じ diagram に振り分け直せるよう、pair のまま返す。
    Ok(rows
        .into_iter()
        .map(|(ancestor_id, descendant_id)| TreePathConnection {
            ancestor_id: ancestor_id as usize,
            descendant_id: descendant_id as usize,
        })
        .collect())
}

async fn create_tree_paths_with<'e, E>(
    executor: E,
    body: CreateTreePathsSchema,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    if body.tree_paths.is_empty() {
        return Ok(());
    }

    let mut builder =
        QueryBuilder::<Postgres>::new("INSERT INTO tree_path (ancestor_id, descendant_id, depth) ");
    builder.push_values(body.tree_paths.iter(), |mut separated, tree_path| {
        separated
            .push_bind(tree_path.ancestor_id as i64)
            .push_bind(tree_path.descendant_id as i64)
            .push_bind(tree_path.depth as i32);
    });

    builder.build().execute(executor).await?;

    Ok(())
}

impl TreePathRepository for RepositoryImpl<TreePathTable> {}

impl ProvidesTreePathRepository for RepositoryImpl<TreePathTable> {
    type T = Self;

    fn tree_path_repository(&self) -> &Self::T {
        self
    }
}
