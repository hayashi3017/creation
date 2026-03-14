use creation_adapter::{model::entity::EntityTable, repository::RepositoryImpl};
use creation_service::{
    model::entity::{
        CreateEntitySchema, DeleteEntitySchema, EntityKind, GetEntitiesSchema, UpdateEntitySchema,
    },
    repository::entity::UsesEntityRepository,
};
use sqlx::{PgPool, Row};

#[sqlx::test(fixtures("entity_repository"))]
async fn get_entities_filters_by_diagram_and_excludes_deleted(db: PgPool) {
    let repo = RepositoryImpl::<EntityTable>::new_test(db).await;
    let ret = repo
        .get_entities(GetEntitiesSchema { diagram_id: 1 })
        .await
        .unwrap();

    let names: Vec<String> = ret.into_iter().map(|entity| entity.name).collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"Active Entity 1".to_string()));
    assert!(names.contains(&"Active Entity 2".to_string()));
    assert!(!names.contains(&"Deleted Entity".to_string()));
    assert!(!names.contains(&"Other Diagram Entity".to_string()));
}

#[sqlx::test(fixtures("entity_repository"))]
async fn create_entity_inserts_row(db: PgPool) {
    let repo = RepositoryImpl::<EntityTable>::new_test(db.clone()).await;
    let body = CreateEntitySchema {
        diagram_id: 1,
        kind: EntityKind::Person,
        name: "New Entity".to_string(),
        description: "created from repository test".to_string(),
    };

    repo.create_entity(body).await.unwrap();

    let count: i64 = sqlx::query_scalar(
        r#"
            SELECT COUNT(*) FROM entity WHERE name = $1 AND diagram_id = $2
        "#,
    )
    .bind("New Entity")
    .bind(1_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(count, 1);
}

#[sqlx::test(fixtures("entity_repository"))]
async fn update_entity_updates_active_row(db: PgPool) {
    let repo = RepositoryImpl::<EntityTable>::new_test(db.clone()).await;
    let body = UpdateEntitySchema {
        id: 1,
        diagram_id: 2,
        kind: EntityKind::Person,
        name: "Updated Entity".to_string(),
        description: "updated from repository test".to_string(),
    };

    repo.update_entity(body).await.unwrap();

    let row = sqlx::query(
        r#"
            SELECT diagram_id, kind, name, description, deleted_at
            FROM entity
            WHERE id = $1
        "#,
    )
    .bind(1_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    let kind = row.get::<EntityKind, _>("kind");

    assert_eq!(row.get::<i64, _>("diagram_id"), 2);
    assert!(matches!(kind, EntityKind::Person));
    assert_eq!(row.get::<String, _>("name"), "Updated Entity");
    assert_eq!(
        row.get::<Option<String>, _>("description").as_deref(),
        Some("updated from repository test")
    );
    assert!(row
        .get::<Option<chrono::DateTime<chrono::Utc>>, _>("deleted_at")
        .is_none());
}

#[sqlx::test(fixtures("entity_repository"))]
async fn delete_entity_marks_row_deleted(db: PgPool) {
    let repo = RepositoryImpl::<EntityTable>::new_test(db.clone()).await;

    repo.delete_entity(DeleteEntitySchema { id: 2 })
        .await
        .unwrap();

    let deleted_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        r#"
            SELECT deleted_at FROM entity WHERE id = $1
        "#,
    )
    .bind(2_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert!(deleted_at.is_some());

    let ret = repo
        .get_entities(GetEntitiesSchema { diagram_id: 1 })
        .await
        .unwrap();
    let names: Vec<String> = ret.into_iter().map(|entity| entity.name).collect();

    assert_eq!(names.len(), 1);
    assert!(names.contains(&"Active Entity 1".to_string()));
}
