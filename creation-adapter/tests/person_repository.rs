use creation_adapter::{model::person::PersonTable, repository::RepositoryImpl};
use creation_service::{
    model::person::{
        CreatePersonRecordSchema, DeletePersonSchema, GenderKind, GetPersonRecordsSchema,
        UpdatePersonRecordSchema,
    },
    repository::person::{
        DeletePersonRepositoryError, UpdatePersonRepositoryError, UsesPersonRepository,
    },
};
use sqlx::{PgPool, Row};

#[sqlx::test(fixtures("person_repository"))]
async fn get_person_records_filters_by_entity_ids_and_excludes_deleted_rows(db: PgPool) {
    let repo = RepositoryImpl::<PersonTable>::new_test(db).await;
    let ret = repo
        .get_person_records(GetPersonRecordsSchema {
            entity_ids: vec![1, 2, 4, 5],
        })
        .await
        .unwrap();

    let entity_ids: Vec<usize> = ret.into_iter().map(|person| person.entity_id).collect();
    assert_eq!(entity_ids, vec![1, 2, 4]);
}

#[sqlx::test(fixtures("person_repository"))]
async fn create_person_record_inserts_person_row_only(db: PgPool) {
    let repo = RepositoryImpl::<PersonTable>::new_test(db.clone()).await;

    sqlx::query(
        r#"
            INSERT INTO entity
                (entity_id, world_id, kind, name, description)
            VALUES
                (6, 1, 'person', 'Created Through Entity Seed', 'seed entity')
        "#,
    )
    .execute(&db)
    .await
    .unwrap();

    sqlx::query(
        r#"
            INSERT INTO diagram_entity
                (diagram_id, entity_id)
            VALUES
                (1, 6)
        "#,
    )
    .execute(&db)
    .await
    .unwrap();

    repo.create_person_record(CreatePersonRecordSchema {
        entity_id: 6,
        first_name: Some("Created".to_string()),
        middle_name: None,
        last_name: Some("Person".to_string()),
        first_name_kana: None,
        middle_name_kana: None,
        last_name_kana: None,
        first_name_romaji: None,
        middle_name_romaji: None,
        last_name_romaji: None,
        gender: Some(GenderKind::Unknown),
        birth_date: Some("2000-02-02".parse().unwrap()),
        death_date: None,
        birthplace: Some("Yokohama".to_string()),
        deathplace: None,
        residence: Some("Kobe".to_string()),
        photo_url: Some("https://example.com/new.png".to_string()),
        profile_text: Some("created profile".to_string()),
    })
    .await
    .unwrap();

    let row = sqlx::query(
        r#"
            SELECT first_name, last_name, gender, birth_date, birthplace, residence, photo_url, profile_text
            FROM person
            WHERE entity_id = $1
        "#,
    )
    .bind(6_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(
        row.get::<Option<String>, _>("first_name").as_deref(),
        Some("Created")
    );
    assert_eq!(
        row.get::<Option<String>, _>("last_name").as_deref(),
        Some("Person")
    );
    assert_eq!(row.get::<GenderKind, _>("gender"), GenderKind::Unknown);
    assert_eq!(
        row.get::<Option<chrono::NaiveDate>, _>("birth_date"),
        Some("2000-02-02".parse().unwrap())
    );
    assert_eq!(
        row.get::<Option<String>, _>("birthplace").as_deref(),
        Some("Yokohama")
    );
    assert_eq!(
        row.get::<Option<String>, _>("residence").as_deref(),
        Some("Kobe")
    );
}

#[sqlx::test(fixtures("person_repository"))]
async fn update_person_record_updates_person_row_only(db: PgPool) {
    let repo = RepositoryImpl::<PersonTable>::new_test(db.clone()).await;

    repo.update_person_record(UpdatePersonRecordSchema {
        entity_id: 1,
        first_name: Some("Updated".to_string()),
        middle_name: None,
        last_name: Some("Record".to_string()),
        first_name_kana: None,
        middle_name_kana: None,
        last_name_kana: None,
        first_name_romaji: Some("Updated".to_string()),
        middle_name_romaji: None,
        last_name_romaji: Some("Record".to_string()),
        gender: Some(GenderKind::Other),
        birth_date: Some("1991-01-01".parse().unwrap()),
        death_date: Some("2020-01-01".parse().unwrap()),
        birthplace: Some("Fukuoka".to_string()),
        deathplace: Some("Naha".to_string()),
        residence: None,
        photo_url: Some("https://example.com/updated.png".to_string()),
        profile_text: Some("updated profile".to_string()),
    })
    .await
    .unwrap();

    let person_row = sqlx::query(
        r#"
            SELECT first_name, last_name_romaji, gender, birth_date, death_date, birthplace, deathplace, residence, photo_url, profile_text
            FROM person
            WHERE entity_id = $1
        "#,
    )
    .bind(1_i64)
    .fetch_one(&db)
    .await
    .unwrap();
    let entity_row = sqlx::query(
        r#"
            SELECT de.diagram_id, e.name, e.description
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

    assert_eq!(
        person_row.get::<Option<String>, _>("first_name").as_deref(),
        Some("Updated")
    );
    assert_eq!(
        person_row
            .get::<Option<String>, _>("last_name_romaji")
            .as_deref(),
        Some("Record")
    );
    assert_eq!(person_row.get::<GenderKind, _>("gender"), GenderKind::Other);
    assert_eq!(
        person_row.get::<Option<chrono::NaiveDate>, _>("death_date"),
        Some("2020-01-01".parse().unwrap())
    );
    assert!(person_row.get::<Option<String>, _>("residence").is_none());
    assert_eq!(
        person_row.get::<Option<String>, _>("deathplace").as_deref(),
        Some("Naha")
    );
    assert_eq!(entity_row.get::<i64, _>("diagram_id"), 1);
    assert_eq!(entity_row.get::<String, _>("name"), "Active Person 1");
}

#[sqlx::test(fixtures("person_repository"))]
async fn delete_person_record_marks_only_person_deleted(db: PgPool) {
    let repo = RepositoryImpl::<PersonTable>::new_test(db.clone()).await;

    repo.delete_person_record(DeletePersonSchema {
        entity_id: 2,
        world_id: 1,
    })
    .await
    .unwrap();

    let row = sqlx::query(
        r#"
            SELECT e.deleted_at AS entity_deleted_at, p.deleted_at AS person_deleted_at
            FROM entity AS e
            INNER JOIN person AS p ON p.entity_id = e.entity_id
            WHERE e.entity_id = $1
        "#,
    )
    .bind(2_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert!(row
        .get::<Option<chrono::DateTime<chrono::Utc>>, _>("entity_deleted_at")
        .is_none());
    assert!(row
        .get::<Option<chrono::DateTime<chrono::Utc>>, _>("person_deleted_at")
        .is_some());
}

#[sqlx::test(fixtures("person_repository"))]
async fn update_person_record_returns_not_found_for_deleted_person_row(db: PgPool) {
    let repo = RepositoryImpl::<PersonTable>::new_test(db).await;

    let err = repo
        .update_person_record(UpdatePersonRecordSchema {
            entity_id: 5,
            first_name: None,
            middle_name: None,
            last_name: None,
            first_name_kana: None,
            middle_name_kana: None,
            last_name_kana: None,
            first_name_romaji: None,
            middle_name_romaji: None,
            last_name_romaji: None,
            gender: None,
            birth_date: None,
            death_date: None,
            birthplace: None,
            deathplace: None,
            residence: None,
            photo_url: None,
            profile_text: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, UpdatePersonRepositoryError::NotFound));
}

#[sqlx::test(fixtures("person_repository"))]
async fn delete_person_record_returns_not_found_for_deleted_person_row(db: PgPool) {
    let repo = RepositoryImpl::<PersonTable>::new_test(db).await;

    let err = repo
        .delete_person_record(DeletePersonSchema {
            entity_id: 5,
            world_id: 1,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, DeletePersonRepositoryError::NotFound));
}
