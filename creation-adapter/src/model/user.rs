use serde::{Deserialize, Serialize};
use sqlx::types::{chrono::NaiveDateTime, Decimal};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct UserTable {
    pub id: uuid::Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub photo: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<Decimal>,
    #[serde(rename = "tzCreatedAt")]
    pub tz_created_at: Option<NaiveDateTime>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<Decimal>,
    #[serde(rename = "tzUpdatedAt")]
    pub tz_updated_at: Option<NaiveDateTime>,
}
