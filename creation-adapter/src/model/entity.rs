use std::fmt::Debug;

use chrono::{DateTime, Utc};
use creation_service::model::entity::EntityKind;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct EntityTable {
    pub entity_id: i64,
    pub diagram_id: i64,
    pub world_id: i64,
    pub kind: EntityKind,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<DateTime<Utc>>,
}
