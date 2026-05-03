use chrono::{DateTime, NaiveDate, Utc};
use creation_service::model::person::GenderKind;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct PersonTable {
    pub entity_id: i64,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub first_name_kana: Option<String>,
    pub middle_name_kana: Option<String>,
    pub last_name_kana: Option<String>,
    pub first_name_romaji: Option<String>,
    pub middle_name_romaji: Option<String>,
    pub last_name_romaji: Option<String>,
    pub gender: Option<GenderKind>,
    pub birth_date: Option<NaiveDate>,
    pub death_date: Option<NaiveDate>,
    pub birthplace: Option<String>,
    pub deathplace: Option<String>,
    pub residence: Option<String>,
    pub photo_url: Option<String>,
    pub profile_text: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<DateTime<Utc>>,
}
