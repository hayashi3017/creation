use async_trait::async_trait;
use creation_service::{
    model::person::{
        CreatePersonRecordSchema, DeletePersonSchema, GetPersonRecordsSchema, PersonRecord,
        UpdatePersonRecordSchema,
    },
    repository::person::{
        CreatePersonRepositoryError, DeletePersonRepositoryError, GetPersonRecordsRepositoryError,
        PersonRepository, ProvidesPersonRepository, UpdatePersonRepositoryError,
        UsesPersonRepository,
    },
    service::person::{PersonService, ProvidesPersonService},
};
use sqlx::{Executor, Postgres};

use crate::{
    model::person::PersonTable,
    repository::{transaction::closed_transaction_error, RepositoryImpl},
};

#[async_trait]
impl UsesPersonRepository for RepositoryImpl<PersonTable> {
    async fn get_person_records(
        &self,
        body: GetPersonRecordsSchema,
    ) -> Result<Vec<PersonRecord>, GetPersonRecordsRepositoryError> {
        let entity_ids: Vec<i64> = body
            .entity_ids
            .into_iter()
            .map(|entity_id| entity_id as i64)
            .collect();

        let persons = sqlx::query_as::<_, PersonTable>(
            r#"
                SELECT
                    entity_id,
                    first_name,
                    middle_name,
                    last_name,
                    first_name_kana,
                    middle_name_kana,
                    last_name_kana,
                    first_name_romaji,
                    middle_name_romaji,
                    last_name_romaji,
                    gender,
                    birth_date,
                    death_date,
                    birthplace,
                    deathplace,
                    residence,
                    photo_url,
                    profile_text,
                    created_at,
                    updated_at,
                    deleted_at
                FROM person
                WHERE
                    deleted_at IS NULL
                    AND entity_id = ANY($1)
                ORDER BY entity_id
            "#,
        )
        .bind(entity_ids)
        .fetch_all(&self.pool.0)
        .await
        .map_err(GetPersonRecordsRepositoryError::Db)?;

        Ok(persons
            .into_iter()
            .map(|person| PersonRecord {
                entity_id: person.entity_id as usize,
                first_name: person.first_name,
                middle_name: person.middle_name,
                last_name: person.last_name,
                first_name_kana: person.first_name_kana,
                middle_name_kana: person.middle_name_kana,
                last_name_kana: person.last_name_kana,
                first_name_romaji: person.first_name_romaji,
                middle_name_romaji: person.middle_name_romaji,
                last_name_romaji: person.last_name_romaji,
                gender: person.gender,
                birth_date: person.birth_date,
                death_date: person.death_date,
                birthplace: person.birthplace,
                deathplace: person.deathplace,
                residence: person.residence,
                photo_url: person.photo_url,
                profile_text: person.profile_text,
            })
            .collect())
    }

    async fn create_person_record(
        &self,
        body: CreatePersonRecordSchema,
    ) -> Result<(), CreatePersonRepositoryError> {
        if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                return create_person_record_with(tx.as_mut(), body)
                    .await
                    .map_err(CreatePersonRepositoryError::Db);
            }

            return Err(CreatePersonRepositoryError::Db(closed_transaction_error()));
        }

        create_person_record_with(&self.pool.0, body)
            .await
            .map_err(CreatePersonRepositoryError::Db)
    }

    async fn update_person_record(
        &self,
        body: UpdatePersonRecordSchema,
    ) -> Result<(), UpdatePersonRepositoryError> {
        let result = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                update_person_record_with(tx.as_mut(), body)
                    .await
                    .map_err(UpdatePersonRepositoryError::Db)?
            } else {
                return Err(UpdatePersonRepositoryError::Db(closed_transaction_error()));
            }
        } else {
            update_person_record_with(&self.pool.0, body)
                .await
                .map_err(UpdatePersonRepositoryError::Db)?
        };

        if result == 0 {
            return Err(UpdatePersonRepositoryError::NotFound);
        }

        Ok(())
    }

    async fn delete_person_record(
        &self,
        body: DeletePersonSchema,
    ) -> Result<(), DeletePersonRepositoryError> {
        let result = if let Some(shared_tx) = &self.tx {
            let mut tx = shared_tx.lock().await;

            if let Some(tx) = tx.as_mut() {
                delete_person_record_with(tx.as_mut(), body)
                    .await
                    .map_err(DeletePersonRepositoryError::Db)?
            } else {
                return Err(DeletePersonRepositoryError::Db(closed_transaction_error()));
            }
        } else {
            delete_person_record_with(&self.pool.0, body)
                .await
                .map_err(DeletePersonRepositoryError::Db)?
        };

        if result == 0 {
            return Err(DeletePersonRepositoryError::NotFound);
        }

        Ok(())
    }
}

async fn create_person_record_with<'e, E>(
    executor: E,
    body: CreatePersonRecordSchema,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query(
        r#"
            INSERT INTO person
                (
                    entity_id,
                    first_name,
                    middle_name,
                    last_name,
                    first_name_kana,
                    middle_name_kana,
                    last_name_kana,
                    first_name_romaji,
                    middle_name_romaji,
                    last_name_romaji,
                    gender,
                    birth_date,
                    death_date,
                    birthplace,
                    deathplace,
                    residence,
                    photo_url,
                    profile_text
                )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9,
                $10, $11, $12, $13, $14, $15, $16, $17, $18
            )
        "#,
    )
    .bind(body.entity_id as i64)
    .bind(body.first_name)
    .bind(body.middle_name)
    .bind(body.last_name)
    .bind(body.first_name_kana)
    .bind(body.middle_name_kana)
    .bind(body.last_name_kana)
    .bind(body.first_name_romaji)
    .bind(body.middle_name_romaji)
    .bind(body.last_name_romaji)
    .bind(body.gender)
    .bind(body.birth_date)
    .bind(body.death_date)
    .bind(body.birthplace)
    .bind(body.deathplace)
    .bind(body.residence)
    .bind(body.photo_url)
    .bind(body.profile_text)
    .execute(executor)
    .await?;

    Ok(())
}

async fn update_person_record_with<'e, E>(
    executor: E,
    body: UpdatePersonRecordSchema,
) -> Result<u64, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let result = sqlx::query(
        r#"
            UPDATE person
            SET
                first_name = $1,
                middle_name = $2,
                last_name = $3,
                first_name_kana = $4,
                middle_name_kana = $5,
                last_name_kana = $6,
                first_name_romaji = $7,
                middle_name_romaji = $8,
                last_name_romaji = $9,
                gender = $10,
                birth_date = $11,
                death_date = $12,
                birthplace = $13,
                deathplace = $14,
                residence = $15,
                photo_url = $16,
                profile_text = $17,
                updated_at = now()
            WHERE
                entity_id = $18
                AND deleted_at IS NULL
        "#,
    )
    .bind(body.first_name)
    .bind(body.middle_name)
    .bind(body.last_name)
    .bind(body.first_name_kana)
    .bind(body.middle_name_kana)
    .bind(body.last_name_kana)
    .bind(body.first_name_romaji)
    .bind(body.middle_name_romaji)
    .bind(body.last_name_romaji)
    .bind(body.gender)
    .bind(body.birth_date)
    .bind(body.death_date)
    .bind(body.birthplace)
    .bind(body.deathplace)
    .bind(body.residence)
    .bind(body.photo_url)
    .bind(body.profile_text)
    .bind(body.entity_id as i64)
    .execute(executor)
    .await?;

    Ok(result.rows_affected())
}

async fn delete_person_record_with<'e, E>(
    executor: E,
    body: DeletePersonSchema,
) -> Result<u64, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let result = sqlx::query(
        r#"
            UPDATE person
            SET
                deleted_at = now(),
                updated_at = now()
            WHERE
                entity_id = $1
                AND deleted_at IS NULL
        "#,
    )
    .bind(body.entity_id as i64)
    .execute(executor)
    .await?;

    Ok(result.rows_affected())
}

impl PersonRepository for RepositoryImpl<PersonTable> {}
impl PersonService for RepositoryImpl<PersonTable> {}

impl ProvidesPersonRepository for RepositoryImpl<PersonTable> {
    type T = Self;

    fn person_repository(&self) -> &Self::T {
        self
    }
}

impl ProvidesPersonService for RepositoryImpl<PersonTable> {
    type T = Self;

    fn person_service(&self) -> &Self::T {
        self
    }
}
