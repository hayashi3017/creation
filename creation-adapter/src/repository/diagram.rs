use async_trait::async_trait;
use creation_service::{
    model::diagram::{
        CreateDiagramSchema, DeleteDiagramSchema, Diagram, DiagramKind, GetDiagramsSchema,
        UpdateDiagramSchema,
    },
    repository::diagram::{
        CreateDiagramRepositoryError, DeleteDiagramRepositoryError, DiagramRepository,
        GetDiagramsRepositoryError, ProvidesDiagramRepository, UpdateDiagramRepositoryError,
        UsesDiagramRepository,
    },
    service::diagram::{DiagramService, ProvidesDiagramService},
};
use creation_usecase::usecase::diagram::{DiagramUsecase, ProvidesDiagramUsecase};

use crate::{model::diagram::DiagramTable, repository::RepositoryImpl};

use super::impl_minimal_cake_bindings;

#[async_trait]
impl UsesDiagramRepository for RepositoryImpl<DiagramTable> {
    async fn get_diagrams(
        &self,
        _body: GetDiagramsSchema,
    ) -> Result<Vec<Diagram>, GetDiagramsRepositoryError> {
        let diagrams = sqlx::query_as!(
            DiagramTable,
            r#"
                SELECT
                    id,
                    name,
                    kind as "kind!: DiagramKind",
                    description,
                    created_at,
                    updated_at,
                    deleted_at
                FROM diagram
                WHERE
                    deleted_at IS NULL
            "#,
        )
        .fetch_all(&self.pool.0)
        .await
        .map_err(|e| GetDiagramsRepositoryError::Db(e))?;

        // convert
        let ret: Vec<Diagram> = diagrams
            .iter()
            .map(|d| {
                return Diagram {
                    id: d.id as usize,
                    name: d.name.clone(),
                    kind: d.kind.clone(),
                    description: d.description.clone(),
                };
            })
            .collect();

        Ok(ret)
    }

    async fn create_diagram(
        &self,
        body: CreateDiagramSchema,
    ) -> Result<(), CreateDiagramRepositoryError> {
        let _ = sqlx::query!(
            r#"
                INSERT INTO diagram
                    (name, kind, description)
                VALUES ($1, $2, $3)
            "#,
            body.name,
            body.kind as DiagramKind,
            body.description,
        )
        .execute(&self.pool.0)
        .await
        .map_err(|e| CreateDiagramRepositoryError::Db(e))?;

        Ok(())
    }

    async fn update_diagram(
        &self,
        body: UpdateDiagramSchema,
    ) -> Result<(), UpdateDiagramRepositoryError> {
        let _ = sqlx::query(
            r#"
                UPDATE diagram
                SET
                    name = $1,
                    kind = $2,
                    description = $3,
                    updated_at = now()
                WHERE
                    id = $4
                    AND deleted_at IS NULL
            "#,
        )
        .bind(body.name)
        .bind(body.kind)
        .bind(body.description)
        .bind(body.id as i64)
        .execute(&self.pool.0)
        .await
        .map_err(UpdateDiagramRepositoryError::Db)?;

        Ok(())
    }

    async fn delete_diagram(
        &self,
        body: DeleteDiagramSchema,
    ) -> Result<(), DeleteDiagramRepositoryError> {
        let _ = sqlx::query(
            r#"
                UPDATE diagram
                SET
                    deleted_at = now(),
                    updated_at = now()
                WHERE
                    id = $1
                    AND deleted_at IS NULL
            "#,
        )
        .bind(body.id as i64)
        .execute(&self.pool.0)
        .await
        .map_err(DeleteDiagramRepositoryError::Db)?;

        Ok(())
    }
}

impl_minimal_cake_bindings!(
    model = DiagramTable,
    repository_trait = DiagramRepository,
    provides_repository_trait = ProvidesDiagramRepository,
    repository_getter = diagram_repository,
    service_trait = DiagramService,
    provides_service_trait = ProvidesDiagramService,
    service_getter = diagram_service,
    usecase_trait = DiagramUsecase,
    provides_usecase_trait = ProvidesDiagramUsecase,
    usecase_getter = diagram_usecase,
);
