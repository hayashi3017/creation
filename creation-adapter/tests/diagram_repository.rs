use creation_adapter::{model::diagram::DiagramTable, repository::RepositoryImpl};
use creation_service::{
    model::diagram::{
        CreateDiagramSchema, DeleteDiagramSchema, DiagramKind, ExistsActiveDiagramSchema,
        GetDiagramSchema, GetDiagramsSchema, UpdateDiagramSchema,
    },
    repository::diagram::{
        DeleteDiagramRepositoryError, UpdateDiagramRepositoryError, UsesDiagramRepository,
    },
};
use sqlx::{PgPool, Row};

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
async fn exists_active_diagram_returns_true_for_active_row(db: PgPool) {
    let repo = RepositoryImpl::<DiagramTable>::new_test(db).await;

    let exists = repo
        .exists_active_diagram(ExistsActiveDiagramSchema { diagram_id: 1 })
        .await
        .unwrap();

    assert!(exists);
}

#[sqlx::test(fixtures("diagram_repository"))]
async fn exists_active_diagram_returns_false_for_deleted_row(db: PgPool) {
    let repo = RepositoryImpl::<DiagramTable>::new_test(db).await;

    let exists = repo
        .exists_active_diagram(ExistsActiveDiagramSchema { diagram_id: 3 })
        .await
        .unwrap();

    assert!(!exists);
}

#[sqlx::test(fixtures("diagram_repository"))]
async fn exists_active_diagram_returns_false_for_missing_row(db: PgPool) {
    let repo = RepositoryImpl::<DiagramTable>::new_test(db).await;

    let exists = repo
        .exists_active_diagram(ExistsActiveDiagramSchema { diagram_id: 99 })
        .await
        .unwrap();

    assert!(!exists);
}

#[sqlx::test(fixtures("diagram_repository"))]
async fn get_diagram_returns_diagram_for_active_row(db: PgPool) {
    let repo = RepositoryImpl::<DiagramTable>::new_test(db).await;

    let diagram = repo
        .get_diagram(GetDiagramSchema { diagram_id: 2 })
        .await
        .unwrap()
        .unwrap();

    assert_eq!(diagram.diagram_id, 2);
    assert_eq!(diagram.name, "Active Diagram 2");
    assert!(matches!(diagram.kind, DiagramKind::Correlation));
    assert_eq!(diagram.description.as_deref(), Some("second active"));
}

#[sqlx::test(fixtures("diagram_repository"))]
async fn get_diagram_returns_none_for_deleted_row(db: PgPool) {
    let repo = RepositoryImpl::<DiagramTable>::new_test(db).await;

    let diagram = repo
        .get_diagram(GetDiagramSchema { diagram_id: 3 })
        .await
        .unwrap();

    assert!(diagram.is_none());
}

#[sqlx::test(fixtures("diagram_repository"))]
async fn get_diagram_returns_none_for_missing_row(db: PgPool) {
    let repo = RepositoryImpl::<DiagramTable>::new_test(db).await;

    let diagram = repo
        .get_diagram(GetDiagramSchema { diagram_id: 99 })
        .await
        .unwrap();

    assert!(diagram.is_none());
}

#[sqlx::test(fixtures("diagram_repository"))]
async fn create_diagram_inserts_row(db: PgPool) {
    let repo = RepositoryImpl::<DiagramTable>::new_test(db.clone()).await;
    let body = CreateDiagramSchema {
        world_id: 1,
        name: "New Diagram".to_string(),
        kind: DiagramKind::FamilyTree,
        genealogy_overview_enabled: true,
        description: Some("created from repository test".to_string()),
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

#[sqlx::test(fixtures("diagram_repository"))]
async fn update_diagram_updates_active_row(db: PgPool) {
    let repo = RepositoryImpl::<DiagramTable>::new_test(db.clone()).await;
    let body = UpdateDiagramSchema {
        diagram_id: 1,
        name: "Updated Diagram".to_string(),
        kind: DiagramKind::Correlation,
        genealogy_overview_enabled: true,
        description: Some("updated from repository test".to_string()),
    };

    repo.update_diagram(body).await.unwrap();

    let row = sqlx::query(
        r#"
            SELECT
                name,
                kind,
                description,
                deleted_at
            FROM diagram
            WHERE diagram_id = $1
        "#,
    )
    .bind(1_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    let kind = row.get::<DiagramKind, _>("kind");

    assert_eq!(row.get::<String, _>("name"), "Updated Diagram");
    assert!(matches!(kind, DiagramKind::Correlation));
    assert_eq!(
        row.get::<Option<String>, _>("description").as_deref(),
        Some("updated from repository test")
    );
    assert!(row
        .get::<Option<chrono::DateTime<chrono::Utc>>, _>("deleted_at")
        .is_none());
}

#[sqlx::test(fixtures("diagram_repository"))]
async fn delete_diagram_marks_row_deleted(db: PgPool) {
    let repo = RepositoryImpl::<DiagramTable>::new_test(db.clone()).await;

    repo.delete_diagram(DeleteDiagramSchema { diagram_id: 2 })
        .await
        .unwrap();

    let deleted_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        r#"
            SELECT deleted_at FROM diagram WHERE diagram_id = $1
        "#,
    )
    .bind(2_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert!(deleted_at.is_some());

    let ret = repo.get_diagrams(GetDiagramsSchema {}).await.unwrap();
    let names: Vec<String> = ret.into_iter().map(|d| d.name).collect();

    assert_eq!(names.len(), 1);
    assert!(names.contains(&"Active Diagram 1".to_string()));
}

#[sqlx::test(fixtures("diagram_repository"))]
async fn update_diagram_returns_not_found_for_deleted_row(db: PgPool) {
    let repo = RepositoryImpl::<DiagramTable>::new_test(db).await;
    let body = UpdateDiagramSchema {
        diagram_id: 3,
        name: "Missing Diagram".to_string(),
        kind: DiagramKind::Correlation,
        genealogy_overview_enabled: true,
        description: Some("should fail".to_string()),
    };

    let err = repo.update_diagram(body).await.unwrap_err();

    assert!(matches!(err, UpdateDiagramRepositoryError::NotFound));
}

#[sqlx::test(fixtures("diagram_repository"))]
async fn delete_diagram_returns_not_found_for_deleted_row(db: PgPool) {
    let repo = RepositoryImpl::<DiagramTable>::new_test(db).await;

    let err = repo
        .delete_diagram(DeleteDiagramSchema { diagram_id: 3 })
        .await
        .unwrap_err();

    assert!(matches!(err, DeleteDiagramRepositoryError::NotFound));
}
