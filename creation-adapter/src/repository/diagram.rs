use async_trait::async_trait;
use creation_service::{
    model::diagram::{
        CreateDiagramSchema, DeleteDiagramSchema, Diagram, DiagramKind, ExistsActiveDiagramSchema,
        GetDiagramSchema, GetDiagramsSchema, UpdateDiagramSchema,
    },
    repository::diagram::{
        CreateDiagramRepositoryError, DeleteDiagramRepositoryError, DiagramRepository,
        ExistsActiveDiagramRepositoryError, GetDiagramRepositoryError, GetDiagramsRepositoryError,
        ProvidesDiagramRepository, UpdateDiagramRepositoryError, UsesDiagramRepository,
    },
    service::diagram::{DiagramService, ProvidesDiagramService},
};

use crate::repository::transaction::closed_transaction_error;
use crate::{model::diagram::DiagramTable, repository::RepositoryImpl};

#[async_trait]
impl UsesDiagramRepository for RepositoryImpl<DiagramTable> {
    async fn get_diagrams(
        &self,
        body: GetDiagramsSchema,
    ) -> Result<Vec<Diagram>, GetDiagramsRepositoryError> {
        let diagrams = sqlx::query_as::<_, DiagramTable>(
            r#"
                SELECT
                    diagram_id,
                    world_id,
                    name,
                    kind,
                    genealogy_overview_enabled,
                    description,
                    created_at,
                    updated_at,
                    deleted_at
                FROM diagram
                WHERE
                    world_id = $1
                    AND
                    deleted_at IS NULL
            "#,
        )
        .bind(body.world_id as i64)
        .fetch_all(&self.pool.0)
        .await
        .map_err(GetDiagramsRepositoryError::Db)?;

        Ok(diagrams.into_iter().map(map_diagram_table).collect())
    }

    async fn exists_active_diagram(
        &self,
        body: ExistsActiveDiagramSchema,
    ) -> Result<bool, ExistsActiveDiagramRepositoryError> {
        let exists = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                exists_active_diagram_with(tx.as_mut(), body)
                    .await
                    .map_err(ExistsActiveDiagramRepositoryError::Db)?
            } else {
                return Err(ExistsActiveDiagramRepositoryError::Db(
                    closed_transaction_error(),
                ));
            }
        } else {
            exists_active_diagram_with(&self.pool.0, body)
                .await
                .map_err(ExistsActiveDiagramRepositoryError::Db)?
        };

        Ok(exists)
    }

    async fn get_diagram(
        &self,
        body: GetDiagramSchema,
    ) -> Result<Option<Diagram>, GetDiagramRepositoryError> {
        let diagram = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                get_diagram_with(tx.as_mut(), body)
                    .await
                    .map_err(GetDiagramRepositoryError::Db)?
            } else {
                return Err(GetDiagramRepositoryError::Db(closed_transaction_error()));
            }
        } else {
            get_diagram_with(&self.pool.0, body)
                .await
                .map_err(GetDiagramRepositoryError::Db)?
        };

        Ok(diagram)
    }

    async fn create_diagram(
        &self,
        body: CreateDiagramSchema,
    ) -> Result<(), CreateDiagramRepositoryError> {
        let result = sqlx::query(
            r#"
                INSERT INTO diagram
                    (world_id, name, kind, genealogy_overview_enabled, description)
                SELECT $1, $2, $3, $4, $5
                FROM world
                WHERE
                    world_id = $1
                    AND deleted_at IS NULL
            "#,
        )
        .bind(body.world_id as i64)
        .bind(body.name)
        .bind(body.kind as DiagramKind)
        .bind(body.genealogy_overview_enabled)
        .bind(body.description)
        .execute(&self.pool.0)
        .await
        .map_err(CreateDiagramRepositoryError::Db)?;

        if result.rows_affected() == 0 {
            return Err(CreateDiagramRepositoryError::Db(sqlx::Error::RowNotFound));
        }

        Ok(())
    }

    async fn update_diagram(
        &self,
        body: UpdateDiagramSchema,
    ) -> Result<(), UpdateDiagramRepositoryError> {
        let result = sqlx::query(
            r#"
                UPDATE diagram
                SET
                    name = $1,
                    kind = $2,
                    genealogy_overview_enabled = $3,
                    description = $4,
                    updated_at = now()
                WHERE
                    diagram_id = $5
                    AND world_id = $6
                    AND deleted_at IS NULL
            "#,
        )
        .bind(body.name)
        .bind(body.kind)
        .bind(body.genealogy_overview_enabled)
        .bind(body.description)
        .bind(body.diagram_id as i64)
        .bind(body.world_id as i64)
        .execute(&self.pool.0)
        .await
        .map_err(UpdateDiagramRepositoryError::Db)?;

        if result.rows_affected() == 0 {
            return Err(UpdateDiagramRepositoryError::NotFound);
        }

        Ok(())
    }

    async fn delete_diagram(
        &self,
        body: DeleteDiagramSchema,
    ) -> Result<(), DeleteDiagramRepositoryError> {
        let result = sqlx::query(
            r#"
                UPDATE diagram
                SET
                    deleted_at = now(),
                    updated_at = now()
                WHERE
                    diagram_id = $1
                    AND world_id = $2
                    AND deleted_at IS NULL
            "#,
        )
        .bind(body.diagram_id as i64)
        .bind(body.world_id as i64)
        .execute(&self.pool.0)
        .await
        .map_err(DeleteDiagramRepositoryError::Db)?;

        if result.rows_affected() == 0 {
            return Err(DeleteDiagramRepositoryError::NotFound);
        }

        Ok(())
    }
}

async fn exists_active_diagram_with<'e, E>(
    executor: E,
    body: ExistsActiveDiagramSchema,
) -> Result<bool, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_scalar::<_, bool>(
        r#"
            SELECT EXISTS (
                SELECT 1
                FROM diagram
                WHERE
                    diagram_id = $1
                    AND deleted_at IS NULL
            )
        "#,
    )
    .bind(body.diagram_id as i64)
    .fetch_one(executor)
    .await
}

async fn get_diagram_with<'e, E>(
    executor: E,
    body: GetDiagramSchema,
) -> Result<Option<Diagram>, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let diagram = sqlx::query_as::<_, DiagramTable>(
        r#"
            SELECT
                diagram_id,
                world_id,
                name,
                kind,
                genealogy_overview_enabled,
                description,
                created_at,
                updated_at,
                deleted_at
            FROM diagram
            WHERE
                diagram_id = $1
                AND deleted_at IS NULL
        "#,
    )
    .bind(body.diagram_id as i64)
    .fetch_optional(executor)
    .await?;

    Ok(diagram.map(map_diagram_table))
}

fn map_diagram_table(diagram: DiagramTable) -> Diagram {
    Diagram {
        diagram_id: diagram.diagram_id as usize,
        world_id: diagram.world_id as usize,
        name: diagram.name,
        kind: diagram.kind,
        genealogy_overview_enabled: diagram.genealogy_overview_enabled,
        description: diagram.description,
    }
}

impl DiagramRepository for RepositoryImpl<DiagramTable> {}

impl ProvidesDiagramRepository for RepositoryImpl<DiagramTable> {
    type T = Self;

    fn diagram_repository(&self) -> &Self::T {
        self
    }
}

impl DiagramService for RepositoryImpl<DiagramTable> {}

impl ProvidesDiagramService for RepositoryImpl<DiagramTable> {
    type T = Self;

    fn diagram_service(&self) -> &Self::T {
        self
    }
}
