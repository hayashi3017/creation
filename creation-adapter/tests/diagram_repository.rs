use creation_adapter::{model::diagram::DiagramTable, repository::RepositoryImpl};
use creation_service::{
    model::diagram::{CreateDiagramSchema, DiagramKind, GetDiagramsSchema},
    repository::diagram::UsesDiagramRepository,
};
use sqlx::PgPool;

#[sqlx::test(fixtures("diagram_repository"))]
async fn get_diagrams_excludes_deleted(db: PgPool) {
    let repo = RepositoryImpl::<DiagramTable>::new_test(db).await;
    let ret = repo.get_diagrams(GetDiagramsSchema {}).await.unwrap();

    let names: Vec<String> = ret.into_iter().map(|d| d.name).collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"Active Diagram 1".to_string()));
    assert!(names.contains(&"Active Diagram 2".to_string()));
}

#[sqlx::test(fixtures("diagram_repository"))]
async fn create_diagram_inserts_row(db: PgPool) {
    let repo = RepositoryImpl::<DiagramTable>::new_test(db.clone()).await;
    let body = CreateDiagramSchema {
        name: "New Diagram".to_string(),
        kind: DiagramKind::FamilyTree,
        description: "created from repository test".to_string(),
    };

    repo.create_diagram(body).await.unwrap();

    let count: i64 = sqlx::query_scalar(
        r#"
            SELECT COUNT(*) FROM diagram WHERE name = $1
        "#,
    )
    .bind("New Diagram")
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(count, 1);
}
