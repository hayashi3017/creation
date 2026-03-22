use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use utoipa::ToSchema;

pub const PERSON_BIRTHPLACE_MAX_CHARS: usize = 255;
pub const PERSON_RESIDENCE_MAX_CHARS: usize = 255;
pub const PERSON_PHOTO_URL_MAX_CHARS: usize = 512;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct Person {
    pub entity_id: usize,
    pub diagram_id: usize,
    pub name: String,
    pub description: Option<String>,
    pub gender: Option<GenderKind>,
    pub birth_date: Option<NaiveDate>,
    pub death_date: Option<NaiveDate>,
    pub birthplace: Option<String>,
    pub residence: Option<String>,
    pub photo_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Type, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "gender_kind")]
#[sqlx(rename_all = "snake_case")]
pub enum GenderKind {
    Male,
    Female,
    Other,
    Unknown,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GetPersonsSchema {
    pub diagram_id: usize,
}

#[derive(Debug, Clone)]
pub struct PersonRecord {
    pub entity_id: usize,
    pub gender: Option<GenderKind>,
    pub birth_date: Option<NaiveDate>,
    pub death_date: Option<NaiveDate>,
    pub birthplace: Option<String>,
    pub residence: Option<String>,
    pub photo_url: Option<String>,
}

#[derive(Debug)]
pub struct GetPersonRecordsSchema {
    pub entity_ids: Vec<usize>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePersonSchema {
    pub diagram_id: usize,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub gender: Option<GenderKind>,
    #[serde(default)]
    pub birth_date: Option<NaiveDate>,
    #[serde(default)]
    pub death_date: Option<NaiveDate>,
    #[serde(default)]
    pub birthplace: Option<String>,
    #[serde(default)]
    pub residence: Option<String>,
    #[serde(default)]
    pub photo_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreatePersonRecordSchema {
    pub entity_id: usize,
    pub gender: Option<GenderKind>,
    pub birth_date: Option<NaiveDate>,
    pub death_date: Option<NaiveDate>,
    pub birthplace: Option<String>,
    pub residence: Option<String>,
    pub photo_url: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePersonSchema {
    pub entity_id: usize,
    pub diagram_id: usize,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub gender: Option<GenderKind>,
    #[serde(default)]
    pub birth_date: Option<NaiveDate>,
    #[serde(default)]
    pub death_date: Option<NaiveDate>,
    #[serde(default)]
    pub birthplace: Option<String>,
    #[serde(default)]
    pub residence: Option<String>,
    #[serde(default)]
    pub photo_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdatePersonRecordSchema {
    pub entity_id: usize,
    pub gender: Option<GenderKind>,
    pub birth_date: Option<NaiveDate>,
    pub death_date: Option<NaiveDate>,
    pub birthplace: Option<String>,
    pub residence: Option<String>,
    pub photo_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeletePersonSchema {
    pub entity_id: usize,
}
