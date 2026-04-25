use chrono::{DateTime, NaiveDate, Utc};
use creation_service::model::relationship::RelationshipKind;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct RelationshipTable {
    pub relationship_id: i64,
    pub diagram_id: i64,
    pub source_entity_id: i64,
    pub target_entity_id: i64,
    pub kind: RelationshipKind,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub end_reason: Option<String>,
    pub notes: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<DateTime<Utc>>,
}
