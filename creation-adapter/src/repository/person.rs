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

use crate::{model::person::PersonTable, repository::RepositoryImpl};

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
                    gender,
                    birth_date,
                    death_date,
                    birthplace,
                    residence,
                    photo_url,
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
                gender: person.gender,
                birth_date: person.birth_date,
                death_date: person.death_date,
                birthplace: person.birthplace,
                residence: person.residence,
                photo_url: person.photo_url,
            })
            .collect())
    }

    async fn create_person_record(
        &self,
        body: CreatePersonRecordSchema,
    ) -> Result<(), CreatePersonRepositoryError> {
        sqlx::query(
            r#"
                INSERT INTO person
                    (entity_id, gender, birth_date, death_date, birthplace, residence, photo_url)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(body.entity_id as i64)
        .bind(body.gender)
        .bind(body.birth_date)
        .bind(body.death_date)
        .bind(body.birthplace)
        .bind(body.residence)
        .bind(body.photo_url)
        .execute(&self.pool.0)
        .await
        .map_err(CreatePersonRepositoryError::Db)?;

        Ok(())
    }

    async fn update_person_record(
        &self,
        body: UpdatePersonRecordSchema,
    ) -> Result<(), UpdatePersonRepositoryError> {
        let result = sqlx::query(
            r#"
                UPDATE person
                SET
                    gender = $1,
                    birth_date = $2,
                    death_date = $3,
                    birthplace = $4,
                    residence = $5,
                    photo_url = $6,
                    updated_at = now()
                WHERE
                    entity_id = $7
                    AND deleted_at IS NULL
            "#,
        )
        .bind(body.gender)
        .bind(body.birth_date)
        .bind(body.death_date)
        .bind(body.birthplace)
        .bind(body.residence)
        .bind(body.photo_url)
        .bind(body.entity_id as i64)
        .execute(&self.pool.0)
        .await
        .map_err(UpdatePersonRepositoryError::Db)?;

        if result.rows_affected() == 0 {
            return Err(UpdatePersonRepositoryError::NotFound);
        }

        Ok(())
    }

    async fn delete_person_record(
        &self,
        body: DeletePersonSchema,
    ) -> Result<(), DeletePersonRepositoryError> {
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
        .execute(&self.pool.0)
        .await
        .map_err(DeletePersonRepositoryError::Db)?;

        if result.rows_affected() == 0 {
            return Err(DeletePersonRepositoryError::NotFound);
        }

        Ok(())
    }
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
