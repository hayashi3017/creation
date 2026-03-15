use async_trait::async_trait;
use creation_service::{
    model::{
        entity::{CreateEntitySchema, DeleteEntitySchema, UpdateEntitySchema},
        person::{CreatePersonRecordSchema, DeletePersonSchema, UpdatePersonRecordSchema},
    },
    repository::unit_of_work::{
        BeginPersonWriteUnitOfWorkError, PersonWriteUnitOfWork, PersonWriteUnitOfWorkError,
    },
};
use sqlx::{Postgres, Transaction};

pub struct SqlxPersonWriteUnitOfWork {
    tx: Transaction<'static, Postgres>,
}

impl SqlxPersonWriteUnitOfWork {
    pub async fn begin(
        pool: &sqlx::Pool<Postgres>,
    ) -> Result<Self, BeginPersonWriteUnitOfWorkError> {
        let tx = pool
            .begin()
            .await
            .map_err(BeginPersonWriteUnitOfWorkError::Db)?;

        Ok(Self { tx })
    }
}

#[async_trait]
impl PersonWriteUnitOfWork for SqlxPersonWriteUnitOfWork {
    async fn create_entity(
        &mut self,
        body: CreateEntitySchema,
    ) -> Result<usize, PersonWriteUnitOfWorkError> {
        let entity_id = sqlx::query_scalar::<_, i64>(
            r#"
                INSERT INTO entity
                    (diagram_id, kind, name, description)
                VALUES ($1, $2, $3, $4)
                RETURNING id
            "#,
        )
        .bind(body.diagram_id as i64)
        .bind(body.kind)
        .bind(body.name)
        .bind(body.description)
        .fetch_one(&mut *self.tx)
        .await
        .map_err(PersonWriteUnitOfWorkError::Db)?;

        Ok(entity_id as usize)
    }

    async fn create_person_record(
        &mut self,
        body: CreatePersonRecordSchema,
    ) -> Result<(), PersonWriteUnitOfWorkError> {
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
        .execute(&mut *self.tx)
        .await
        .map_err(PersonWriteUnitOfWorkError::Db)?;

        Ok(())
    }

    async fn update_entity(
        &mut self,
        body: UpdateEntitySchema,
    ) -> Result<(), PersonWriteUnitOfWorkError> {
        let kind = body.kind;
        let result = sqlx::query(
            r#"
                UPDATE entity
                SET
                    diagram_id = $1,
                    kind = $2,
                    name = $3,
                    description = $4,
                    updated_at = now()
                WHERE
                    id = $5
                    AND kind = $2
                    AND deleted_at IS NULL
            "#,
        )
        .bind(body.diagram_id as i64)
        .bind(kind.clone())
        .bind(body.name)
        .bind(body.description)
        .bind(body.id as i64)
        .execute(&mut *self.tx)
        .await
        .map_err(PersonWriteUnitOfWorkError::Db)?;

        if result.rows_affected() == 0 {
            return Err(PersonWriteUnitOfWorkError::NotFound);
        }

        Ok(())
    }

    async fn update_person_record(
        &mut self,
        body: UpdatePersonRecordSchema,
    ) -> Result<(), PersonWriteUnitOfWorkError> {
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
        .execute(&mut *self.tx)
        .await
        .map_err(PersonWriteUnitOfWorkError::Db)?;

        if result.rows_affected() == 0 {
            return Err(PersonWriteUnitOfWorkError::NotFound);
        }

        Ok(())
    }

    async fn delete_entity(
        &mut self,
        body: DeleteEntitySchema,
    ) -> Result<(), PersonWriteUnitOfWorkError> {
        let result = sqlx::query(
            r#"
                UPDATE entity
                SET
                    deleted_at = now(),
                    updated_at = now()
                WHERE
                    id = $1
                    AND kind = 'person'::entity_kind
                    AND deleted_at IS NULL
            "#,
        )
        .bind(body.id as i64)
        .execute(&mut *self.tx)
        .await
        .map_err(PersonWriteUnitOfWorkError::Db)?;

        if result.rows_affected() == 0 {
            return Err(PersonWriteUnitOfWorkError::NotFound);
        }

        Ok(())
    }

    async fn delete_person_record(
        &mut self,
        body: DeletePersonSchema,
    ) -> Result<(), PersonWriteUnitOfWorkError> {
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
        .execute(&mut *self.tx)
        .await
        .map_err(PersonWriteUnitOfWorkError::Db)?;

        if result.rows_affected() == 0 {
            return Err(PersonWriteUnitOfWorkError::NotFound);
        }

        Ok(())
    }

    async fn commit(self) -> Result<(), PersonWriteUnitOfWorkError> {
        self.tx
            .commit()
            .await
            .map_err(PersonWriteUnitOfWorkError::Db)
    }
}
