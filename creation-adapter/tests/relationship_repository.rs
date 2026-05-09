use creation_adapter::{
    model::{entity::EntityTable, relationship::RelationshipTable, tree_path::TreePathTable},
    repository::RepositoryImpl,
};
use creation_service::{
    model::{
        relationship::{
            CreateRelationshipSchema, DeleteRelationshipSchema, GetRelationshipsSchema,
            LoadRelationshipWorldIdSchema, RelationshipEndpoints, RelationshipKind,
            UpdateRelationshipSchema, UpdatedRelationshipEndpoints,
        },
        tree_path::SyncTreePathsByEntityIdsSchema,
    },
    repository::{
        entity::ProvidesEntityRepository,
        relationship::{
            CreateRelationshipRepositoryError, DeleteRelationshipRepositoryError,
            ProvidesRelationshipRepository, UpdateRelationshipRepositoryError,
            UsesRelationshipRepository,
        },
        tree_path::ProvidesTreePathRepository,
    },
    service::tree_path::{
        ProvidesTreePathService, SyncTreePathsServiceError, TreePathService, UsesTreePathService,
    },
};
use sqlx::{PgPool, Row};

struct TestModule {
    entity_repository: RepositoryImpl<EntityTable>,
    relationship_repository: RepositoryImpl<RelationshipTable>,
    tree_path_repository: RepositoryImpl<TreePathTable>,
}

impl TestModule {
    async fn new(db: PgPool) -> Self {
        Self {
            entity_repository: RepositoryImpl::<EntityTable>::new_test(db.clone()).await,
            relationship_repository: RepositoryImpl::<RelationshipTable>::new_test(db.clone())
                .await,
            tree_path_repository: RepositoryImpl::<TreePathTable>::new_test(db).await,
        }
    }
}

impl ProvidesEntityRepository for TestModule {
    type T = RepositoryImpl<EntityTable>;

    fn entity_repository(&self) -> &Self::T {
        &self.entity_repository
    }
}

impl ProvidesRelationshipRepository for TestModule {
    type T = RepositoryImpl<RelationshipTable>;

    fn relationship_repository(&self) -> &Self::T {
        &self.relationship_repository
    }
}

impl ProvidesTreePathRepository for TestModule {
    type T = RepositoryImpl<TreePathTable>;

    fn tree_path_repository(&self) -> &Self::T {
        &self.tree_path_repository
    }
}

impl TreePathService for TestModule {}

impl ProvidesTreePathService for TestModule {
    type T = Self;

    fn tree_path_service(&self) -> &Self::T {
        self
    }
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn get_relationships_filters_by_world_and_excludes_deleted(db: PgPool) {
    let repo = RepositoryImpl::<RelationshipTable>::new_test(db).await;

    let ret = repo
        .get_relationships(GetRelationshipsSchema { world_id: 1 })
        .await
        .unwrap();

    let ids: Vec<usize> = ret
        .into_iter()
        .map(|relationship| relationship.relationship_id)
        .collect();
    assert_eq!(ids, vec![1, 2, 3, 5]);
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn get_relationships_returns_empty_for_missing_world(db: PgPool) {
    let repo = RepositoryImpl::<RelationshipTable>::new_test(db).await;

    let ret = repo
        .get_relationships(GetRelationshipsSchema { world_id: 3 })
        .await
        .unwrap();

    assert!(ret.is_empty());
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn create_relationship_inserts_row(db: PgPool) {
    let repo = RepositoryImpl::<RelationshipTable>::new_test(db.clone()).await;

    repo.create_relationship(CreateRelationshipSchema {
        world_id: 1,
        source_entity_id: 3,
        target_entity_id: 7,
        kind: RelationshipKind::Parent,
        start_date: Some("2020-01-01".parse().unwrap()),
        end_date: None,
        end_reason: None,
        notes: Some("created from repository test".to_string()),
    })
    .await
    .unwrap();

    let row = sqlx::query(
        r#"
            SELECT kind, notes
            FROM relationship
            WHERE world_id = $1
                AND source_entity_id = $2
                AND target_entity_id = $3
                AND deleted_at IS NULL
        "#,
    )
    .bind(1_i64)
    .bind(3_i64)
    .bind(7_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(
        row.get::<RelationshipKind, _>("kind"),
        RelationshipKind::Parent
    );
    assert_eq!(
        row.get::<Option<String>, _>("notes").as_deref(),
        Some("created from repository test")
    );
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn create_relationship_returns_not_found_for_missing_entity(db: PgPool) {
    let repo = RepositoryImpl::<RelationshipTable>::new_test(db).await;

    let err = repo
        .create_relationship(CreateRelationshipSchema {
            world_id: 1,
            source_entity_id: 3,
            target_entity_id: 99,
            kind: RelationshipKind::Parent,
            start_date: None,
            end_date: None,
            end_reason: None,
            notes: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, CreateRelationshipRepositoryError::NotFound));
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn create_relationship_returns_not_found_for_missing_world(db: PgPool) {
    let repo = RepositoryImpl::<RelationshipTable>::new_test(db).await;

    let err = repo
        .create_relationship(CreateRelationshipSchema {
            world_id: 3,
            source_entity_id: 8,
            target_entity_id: 9,
            kind: RelationshipKind::Parent,
            start_date: None,
            end_date: None,
            end_reason: None,
            notes: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, CreateRelationshipRepositoryError::NotFound));
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn update_relationship_updates_active_row(db: PgPool) {
    let repo = RepositoryImpl::<RelationshipTable>::new_test(db.clone()).await;

    let endpoints = repo
        .update_relationship(UpdateRelationshipSchema {
            relationship_id: 2,
            source_entity_id: 7,
            target_entity_id: 3,
            kind: RelationshipKind::Parent,
            start_date: Some("2010-01-01".parse().unwrap()),
            end_date: None,
            end_reason: None,
            notes: Some("updated edge".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(
        endpoints,
        UpdatedRelationshipEndpoints {
            previous_source_entity_id: 2,
            previous_target_entity_id: 3,
        }
    );

    let row = sqlx::query(
        r#"
            SELECT source_entity_id, target_entity_id, kind, notes
            FROM relationship
            WHERE relationship_id = $1
        "#,
    )
    .bind(2_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(row.get::<i64, _>("source_entity_id"), 7);
    assert_eq!(row.get::<i64, _>("target_entity_id"), 3);
    assert_eq!(
        row.get::<RelationshipKind, _>("kind"),
        RelationshipKind::Parent
    );
    assert_eq!(
        row.get::<Option<String>, _>("notes").as_deref(),
        Some("updated edge")
    );
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn delete_relationship_marks_row_deleted(db: PgPool) {
    let repo = RepositoryImpl::<RelationshipTable>::new_test(db.clone()).await;

    let endpoints = repo
        .delete_relationship(DeleteRelationshipSchema { relationship_id: 2 })
        .await
        .unwrap();

    assert_eq!(
        endpoints,
        RelationshipEndpoints {
            source_entity_id: 2,
            target_entity_id: 3,
        }
    );

    let deleted_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        r#"
            SELECT deleted_at
            FROM relationship
            WHERE relationship_id = $1
        "#,
    )
    .bind(2_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert!(deleted_at.is_some());
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn update_relationship_returns_not_found_for_deleted_row(db: PgPool) {
    let repo = RepositoryImpl::<RelationshipTable>::new_test(db).await;

    let err = repo
        .update_relationship(UpdateRelationshipSchema {
            relationship_id: 4,
            source_entity_id: 1,
            target_entity_id: 7,
            kind: RelationshipKind::Parent,
            start_date: None,
            end_date: None,
            end_reason: None,
            notes: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, UpdateRelationshipRepositoryError::NotFound));
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn update_relationship_ignores_diagram_soft_delete(db: PgPool) {
    let repo = RepositoryImpl::<RelationshipTable>::new_test(db).await;

    repo.update_relationship(UpdateRelationshipSchema {
        relationship_id: 5,
        source_entity_id: 8,
        target_entity_id: 9,
        kind: RelationshipKind::Parent,
        start_date: None,
        end_date: None,
        end_reason: None,
        notes: None,
    })
    .await
    .unwrap();
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn delete_relationship_returns_not_found_for_deleted_row(db: PgPool) {
    let repo = RepositoryImpl::<RelationshipTable>::new_test(db).await;

    let err = repo
        .delete_relationship(DeleteRelationshipSchema { relationship_id: 4 })
        .await
        .unwrap_err();

    assert!(matches!(err, DeleteRelationshipRepositoryError::NotFound));
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn delete_relationship_ignores_diagram_soft_delete(db: PgPool) {
    let repo = RepositoryImpl::<RelationshipTable>::new_test(db).await;

    repo.delete_relationship(DeleteRelationshipSchema { relationship_id: 5 })
        .await
        .unwrap();
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn load_relationship_world_id_returns_world_id_for_active_row(db: PgPool) {
    let repo = RepositoryImpl::<RelationshipTable>::new_test(db).await;

    let world_id = repo
        .load_relationship_world_id(LoadRelationshipWorldIdSchema { relationship_id: 2 })
        .await
        .unwrap();

    assert_eq!(world_id, Some(1));
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn load_relationship_world_id_ignores_diagram_soft_delete(db: PgPool) {
    let repo = RepositoryImpl::<RelationshipTable>::new_test(db).await;

    let world_id = repo
        .load_relationship_world_id(LoadRelationshipWorldIdSchema { relationship_id: 5 })
        .await
        .unwrap();

    assert_eq!(world_id, Some(1));
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn load_relationship_world_id_returns_none_for_deleted_row(db: PgPool) {
    let repo = RepositoryImpl::<RelationshipTable>::new_test(db).await;

    let world_id = repo
        .load_relationship_world_id(LoadRelationshipWorldIdSchema { relationship_id: 4 })
        .await
        .unwrap();

    assert_eq!(world_id, None);
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn sync_tree_paths_by_entity_ids_creates_closure_rows(db: PgPool) {
    let module = TestModule::new(db.clone()).await;

    module
        .sync_tree_paths_by_entity_ids(SyncTreePathsByEntityIdsSchema {
            world_id: 1,
            entity_ids: vec![2, 3],
        })
        .await
        .unwrap();

    let rows = sqlx::query(
        r#"
            SELECT ancestor_id, descendant_id, depth
            FROM tree_path
            ORDER BY ancestor_id, descendant_id
        "#,
    )
    .fetch_all(&db)
    .await
    .unwrap();

    let tuples: Vec<(i64, i64, i32)> = rows
        .into_iter()
        .map(|row| {
            (
                row.get::<i64, _>("ancestor_id"),
                row.get::<i64, _>("descendant_id"),
                row.get::<i32, _>("depth"),
            )
        })
        .collect();

    assert!(tuples.contains(&(1, 1, 0)));
    assert!(tuples.contains(&(2, 2, 0)));
    assert!(tuples.contains(&(3, 3, 0)));
    assert!(tuples.contains(&(1, 2, 1)));
    assert!(tuples.contains(&(2, 3, 1)));
    assert!(tuples.contains(&(1, 3, 2)));
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn sync_tree_paths_by_entity_ids_uses_only_tree_edge_kinds(db: PgPool) {
    let module = TestModule::new(db.clone()).await;

    module
        .relationship_repository()
        .create_relationship(CreateRelationshipSchema {
            world_id: 1,
            source_entity_id: 3,
            target_entity_id: 7,
            kind: RelationshipKind::AdoptiveParent,
            start_date: None,
            end_date: None,
            end_reason: None,
            notes: Some("adoptive lineage".to_string()),
        })
        .await
        .unwrap();
    module
        .relationship_repository()
        .create_relationship(CreateRelationshipSchema {
            world_id: 1,
            source_entity_id: 7,
            target_entity_id: 1,
            kind: RelationshipKind::StepParent,
            start_date: None,
            end_date: None,
            end_reason: None,
            notes: Some("ignored step relation".to_string()),
        })
        .await
        .unwrap();
    module
        .relationship_repository()
        .create_relationship(CreateRelationshipSchema {
            world_id: 1,
            source_entity_id: 7,
            target_entity_id: 2,
            kind: RelationshipKind::Spouse,
            start_date: None,
            end_date: None,
            end_reason: None,
            notes: Some("ignored spouse relation".to_string()),
        })
        .await
        .unwrap();

    module
        .sync_tree_paths_by_entity_ids(SyncTreePathsByEntityIdsSchema {
            world_id: 1,
            entity_ids: vec![3, 7],
        })
        .await
        .unwrap();

    let rows = sqlx::query(
        r#"
            SELECT ancestor_id, descendant_id, depth
            FROM tree_path
            ORDER BY ancestor_id, descendant_id
        "#,
    )
    .fetch_all(&db)
    .await
    .unwrap();

    let tuples: Vec<(i64, i64, i32)> = rows
        .into_iter()
        .map(|row| {
            (
                row.get::<i64, _>("ancestor_id"),
                row.get::<i64, _>("descendant_id"),
                row.get::<i32, _>("depth"),
            )
        })
        .collect();

    assert!(tuples.contains(&(3, 7, 1)));
    assert!(tuples.contains(&(1, 7, 3)));
    assert!(!tuples.contains(&(7, 1, 1)));
    assert!(!tuples.contains(&(7, 2, 1)));
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn sync_tree_paths_by_entity_ids_detects_cycle(db: PgPool) {
    let module = TestModule::new(db.clone()).await;

    module
        .relationship_repository()
        .create_relationship(CreateRelationshipSchema {
            world_id: 1,
            source_entity_id: 3,
            target_entity_id: 1,
            kind: RelationshipKind::Parent,
            start_date: None,
            end_date: None,
            end_reason: None,
            notes: Some("cycle".to_string()),
        })
        .await
        .unwrap();

    let err = module
        .sync_tree_paths_by_entity_ids(SyncTreePathsByEntityIdsSchema {
            world_id: 1,
            entity_ids: vec![1, 3],
        })
        .await
        .unwrap_err();

    assert!(matches!(err, SyncTreePathsServiceError::CycleDetected));
}

#[sqlx::test(fixtures("relationship_repository"))]
async fn sync_tree_paths_by_entity_ids_rebuilds_each_related_component(db: PgPool) {
    let module = TestModule::new(db.clone()).await;

    module
        .sync_tree_paths_by_entity_ids(SyncTreePathsByEntityIdsSchema {
            world_id: 1,
            entity_ids: vec![5, 2, 5],
        })
        .await
        .unwrap();

    let rows = sqlx::query(
        r#"
            SELECT ancestor_id, descendant_id, depth
            FROM tree_path
            ORDER BY ancestor_id, descendant_id
        "#,
    )
    .fetch_all(&db)
    .await
    .unwrap();

    let tuples: Vec<(i64, i64, i32)> = rows
        .into_iter()
        .map(|row| {
            (
                row.get::<i64, _>("ancestor_id"),
                row.get::<i64, _>("descendant_id"),
                row.get::<i32, _>("depth"),
            )
        })
        .collect();

    assert!(tuples.contains(&(1, 3, 2)));
    assert!(tuples.contains(&(4, 5, 1)));
    assert_eq!(
        tuples.iter().filter(|&&(a, d, _)| a == 4 && d == 5).count(),
        1
    );
}
