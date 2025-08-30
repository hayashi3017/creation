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

#[async_trait]
impl UsesDiagramRepository for RepositoryImpl<DiagramTable> {
    async fn get_diagrams(
        &self,
        _body: GetDiagramsSchema,
    ) -> Result<Vec<Diagram>, GetDiagramsRepositoryError> {
        let diagram = sqlx::query_as!(
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
                    deleted_at IS NOT NULL
            "#,
        );

        Ok(vec![])
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
        Ok(())
    }

    async fn delete_diagram(
        &self,
        body: DeleteDiagramSchema,
    ) -> Result<(), DeleteDiagramRepositoryError> {
        Ok(())
    }
}

impl DiagramRepository for RepositoryImpl<DiagramTable> {}
impl DiagramService for RepositoryImpl<DiagramTable> {}
impl DiagramUsecase for RepositoryImpl<DiagramTable> {}

impl ProvidesDiagramRepository for RepositoryImpl<DiagramTable> {
    type T = Self;
    fn diagram_repository(&self) -> &Self::T {
        self
    }
}
impl ProvidesDiagramService for RepositoryImpl<DiagramTable> {
    type T = Self;
    fn diagram_service(&self) -> &Self::T {
        self
    }
}
impl ProvidesDiagramUsecase for RepositoryImpl<DiagramTable> {
    type T = Self;
    fn diagram_usecase(&self) -> &Self::T {
        self
    }
}
