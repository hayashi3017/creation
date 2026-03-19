use creation_driver::AppModule;
use creation_service::{
    model::{
        entity::{CreateEntitySchema, EntityKind},
        person::CreatePersonRecordSchema,
    },
    service::{
        entity::{CreateEntityServiceError, ProvidesEntityService, UsesEntityService},
        person::{CreatePersonRecordServiceError, ProvidesPersonService, UsesPersonService},
        transaction::{ProvidesTransactionManager, TransactionContext},
    },
};
use sqlx::{PgPool, Row};

#[sqlx::test(fixtures("person"))]
async fn transaction_commit_persists_entity_and_person(db: PgPool) {
    let module = AppModule::new_test(db.clone()).await;
    let tx = module.begin_transaction().await.unwrap();

    let entity_id = tx
        .entity_service()
        .create_entity(CreateEntitySchema {
            diagram_id: 1,
            kind: EntityKind::Person,
            name: "Committed Transaction Person".to_string(),
            description: Some("committed".to_string()),
        })
        .await
        .unwrap();

    tx.person_service()
        .create_person_record(CreatePersonRecordSchema {
            entity_id,
            gender: None,
            birth_date: None,
            death_date: None,
            birthplace: Some("Tokyo".to_string()),
            residence: None,
            photo_url: None,
        })
        .await
        .unwrap();

    tx.commit().await.unwrap();

    let row = sqlx::query(
        r#"
            SELECT e.name, p.birthplace
            FROM entity AS e
            INNER JOIN person AS p ON p.entity_id = e.id
            WHERE e.id = $1
        "#,
    )
    .bind(entity_id as i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(row.get::<String, _>("name"), "Committed Transaction Person");
    assert_eq!(
        row.get::<Option<String>, _>("birthplace").as_deref(),
        Some("Tokyo")
    );
}

#[sqlx::test(fixtures("person"))]
async fn transaction_rolls_back_when_followup_service_fails(db: PgPool) {
    let module = AppModule::new_test(db.clone()).await;
    let tx = module.begin_transaction().await.unwrap();

    let entity_id = tx
        .entity_service()
        .create_entity(CreateEntitySchema {
            diagram_id: 1,
            kind: EntityKind::Person,
            name: "Rollback Candidate".to_string(),
            description: Some("should not persist".to_string()),
        })
        .await
        .unwrap();

    let result = tx
        .person_service()
        .create_person_record(CreatePersonRecordSchema {
            entity_id: entity_id + 10_000,
            gender: None,
            birth_date: None,
            death_date: None,
            birthplace: None,
            residence: None,
            photo_url: None,
        })
        .await;

    assert!(matches!(
        result,
        Err(CreatePersonRecordServiceError::CreatePersonRepositoryError(
            _
        ))
    ));

    drop(tx);

    let row = sqlx::query(
        r#"
            SELECT COUNT(*) AS count
            FROM entity
            WHERE name = $1
        "#,
    )
    .bind("Rollback Candidate")
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(row.get::<i64, _>("count"), 0);
}

#[sqlx::test(fixtures("person"))]
async fn closed_transaction_does_not_fallback_to_pool(db: PgPool) {
    let module = AppModule::new_test(db.clone()).await;
    let tx = module.begin_transaction().await.unwrap();
    let tx_clone = tx.clone();

    tx.commit().await.unwrap();

    let result = tx_clone
        .entity_service()
        .create_entity(CreateEntitySchema {
            diagram_id: 1,
            kind: EntityKind::Person,
            name: "Should Error After Commit".to_string(),
            description: None,
        })
        .await;

    assert!(matches!(
        result,
        Err(CreateEntityServiceError::CreateEntityRepositoryError(_))
    ));

    let row = sqlx::query(
        r#"
            SELECT COUNT(*) AS count
            FROM entity
            WHERE name = $1
        "#,
    )
    .bind("Should Error After Commit")
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(row.get::<i64, _>("count"), 0);
}
