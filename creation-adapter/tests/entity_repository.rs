use creation_adapter::{model::entity::EntityTable, repository::RepositoryImpl};
use creation_service::{
    model::entity::{
        CreateDiagramEntityMembershipSchema, CreateEntitySchema, DeleteEntitySchema, EntityKind,
        GetEntitiesSchema, SyncDiagramEntityMembershipsSchema, SyncEntityDiagramMembershipsSchema,
        UpdateEntitySchema,
    },
    repository::entity::{
        CreateDiagramEntityMembershipRepositoryError, CreateEntityRepositoryError,
        DeleteEntityRepositoryError, UpdateEntityRepositoryError, UsesEntityRepository,
    },
};
use sqlx::{PgPool, Row};

#[sqlx::test(fixtures("entity_repository"))]
async fn get_entities_filters_by_diagram_and_excludes_deleted(db: PgPool) {
    let repo = RepositoryImpl::<EntityTable>::new_test(db).await;
    let ret = repo
        .get_entities(GetEntitiesSchema {
            world_id: 1,
            diagram_id: 1,
        })
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
        world_id: 1,
        kind: EntityKind::Person,
        name: "New Entity".to_string(),
        description: Some("created from repository test".to_string()),
    };

    let entity_id = repo.create_entity(body).await.unwrap();

    let row = sqlx::query(
        r#"
            SELECT world_id, kind, name, description, deleted_at
            FROM entity
            WHERE entity_id = $1
        "#,
    )
    .bind(entity_id as i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(row.get::<i64, _>("world_id"), 1);
    assert!(matches!(
        row.get::<EntityKind, _>("kind"),
        EntityKind::Person
    ));
    assert_eq!(row.get::<String, _>("name"), "New Entity");
    assert_eq!(
        row.get::<Option<String>, _>("description").as_deref(),
        Some("created from repository test")
    );
    assert!(row
        .get::<Option<chrono::DateTime<chrono::Utc>>, _>("deleted_at")
        .is_none());

    let membership_count: i64 = sqlx::query_scalar(
        r#"
            SELECT COUNT(*)
            FROM diagram_entity
            WHERE entity_id = $1
        "#,
    )
    .bind(entity_id as i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(membership_count, 0);
}

#[sqlx::test(fixtures("entity_repository"))]
async fn create_entity_returns_not_found_for_deleted_world(db: PgPool) {
    let repo = RepositoryImpl::<EntityTable>::new_test(db).await;
    let body = CreateEntitySchema {
        world_id: 3,
        kind: EntityKind::Person,
        name: "Rejected Entity".to_string(),
        description: None,
    };

    let err = repo.create_entity(body).await.unwrap_err();

    assert!(matches!(err, CreateEntityRepositoryError::NotFound));
}

#[sqlx::test(fixtures("entity_repository"))]
async fn create_diagram_entity_membership_inserts_row(db: PgPool) {
    let repo = RepositoryImpl::<EntityTable>::new_test(db.clone()).await;

    repo.create_diagram_entity_membership(CreateDiagramEntityMembershipSchema {
        diagram_id: 1,
        entity_id: 4,
    })
    .await
    .unwrap();

    let count: i64 = sqlx::query_scalar(
        r#"
            SELECT COUNT(*)
            FROM diagram_entity
            WHERE diagram_id = $1 AND entity_id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(1_i64)
    .bind(4_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(count, 1);
}

#[sqlx::test(fixtures("entity_repository"))]
async fn create_diagram_entity_membership_returns_not_found_for_world_mismatch(db: PgPool) {
    let repo = RepositoryImpl::<EntityTable>::new_test(db).await;

    let err = repo
        .create_diagram_entity_membership(CreateDiagramEntityMembershipSchema {
            diagram_id: 1,
            entity_id: 5,
        })
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        CreateDiagramEntityMembershipRepositoryError::NotFound
    ));
}

#[sqlx::test(fixtures("entity_repository"))]
async fn sync_diagram_entity_memberships_links_entities_to_diagram(db: PgPool) {
    let repo = RepositoryImpl::<EntityTable>::new_test(db.clone()).await;

    repo.sync_diagram_entity_memberships(SyncDiagramEntityMembershipsSchema {
        diagram_id: 2,
        entity_ids: vec![1, 2],
    })
    .await
    .unwrap();

    let active_entity_ids = sqlx::query_scalar::<_, i64>(
        r#"
            SELECT entity_id
            FROM diagram_entity
            WHERE diagram_id = $1 AND deleted_at IS NULL
            ORDER BY entity_id
        "#,
    )
    .bind(2_i64)
    .fetch_all(&db)
    .await
    .unwrap();

    assert_eq!(active_entity_ids, vec![1, 2, 4]);
}

#[sqlx::test(fixtures("entity_repository"))]
async fn sync_entity_diagram_memberships_replaces_active_memberships(db: PgPool) {
    let repo = RepositoryImpl::<EntityTable>::new_test(db.clone()).await;

    repo.sync_entity_diagram_memberships(SyncEntityDiagramMembershipsSchema {
        entity_id: 1,
        world_id: 1,
        diagram_ids: vec![2],
    })
    .await
    .unwrap();

    let active_diagram_ids = sqlx::query_scalar::<_, i64>(
        r#"
            SELECT diagram_id
            FROM diagram_entity
            WHERE entity_id = $1 AND deleted_at IS NULL
            ORDER BY diagram_id
        "#,
    )
    .bind(1_i64)
    .fetch_all(&db)
    .await
    .unwrap();

    assert_eq!(active_diagram_ids, vec![2]);
}

#[sqlx::test(fixtures("entity_repository"))]
async fn sync_entity_diagram_memberships_returns_not_found_for_world_mismatch(db: PgPool) {
    let repo = RepositoryImpl::<EntityTable>::new_test(db).await;

    let err = repo
        .sync_entity_diagram_memberships(SyncEntityDiagramMembershipsSchema {
            entity_id: 5,
            world_id: 2,
            diagram_ids: vec![1],
        })
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        creation_service::repository::entity::SyncEntityDiagramMembershipsRepositoryError::NotFound
    ));
}

#[sqlx::test(fixtures("entity_repository"))]
async fn update_entity_updates_active_row(db: PgPool) {
    let repo = RepositoryImpl::<EntityTable>::new_test(db.clone()).await;
    let body = UpdateEntitySchema {
        entity_id: 1,
        world_id: 1,
        kind: EntityKind::Person,
        name: "Updated Entity".to_string(),
        description: Some("updated from repository test".to_string()),
    };

    repo.update_entity(body).await.unwrap();

    let row = sqlx::query(
        r#"
            SELECT de.diagram_id, e.kind, e.name, e.description, e.deleted_at
            FROM entity AS e
            INNER JOIN diagram_entity AS de
                ON de.entity_id = e.entity_id
            WHERE e.entity_id = $1
        "#,
    )
    .bind(1_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    let kind = row.get::<EntityKind, _>("kind");

    assert_eq!(row.get::<i64, _>("diagram_id"), 1);
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

    repo.delete_entity(DeleteEntitySchema {
        entity_id: 2,
        world_id: 1,
    })
    .await
    .unwrap();

    let deleted_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        r#"
            SELECT deleted_at FROM entity WHERE entity_id = $1
        "#,
    )
    .bind(2_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert!(deleted_at.is_some());

    let ret = repo
        .get_entities(GetEntitiesSchema {
            world_id: 1,
            diagram_id: 1,
        })
        .await
        .unwrap();
    let names: Vec<String> = ret.into_iter().map(|entity| entity.name).collect();

    assert_eq!(names.len(), 1);
    assert!(names.contains(&"Active Entity 1".to_string()));
}

#[sqlx::test(fixtures("entity_repository"))]
async fn update_entity_returns_not_found_for_deleted_row(db: PgPool) {
    let repo = RepositoryImpl::<EntityTable>::new_test(db).await;
    let body = UpdateEntitySchema {
        entity_id: 3,
        world_id: 1,
        kind: EntityKind::Person,
        name: "Missing Entity".to_string(),
        description: Some("should fail".to_string()),
    };

    let err = repo.update_entity(body).await.unwrap_err();

    assert!(matches!(err, UpdateEntityRepositoryError::NotFound));
}

#[sqlx::test(fixtures("entity_repository"))]
async fn delete_entity_returns_not_found_for_deleted_row(db: PgPool) {
    let repo = RepositoryImpl::<EntityTable>::new_test(db).await;

    let err = repo
        .delete_entity(DeleteEntitySchema {
            entity_id: 3,
            world_id: 1,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, DeleteEntityRepositoryError::NotFound));
}
