use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const WORLD_NAME_MAX_CHARS: usize = 255;

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct World {
    pub world_id: usize,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct GetWorldsSchema {}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateWorldSchema {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateWorldSchema {
    pub world_id: usize,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteWorldSchema {
    pub world_id: usize,
}

#[derive(Debug, Deserialize)]
pub struct GetWorldSchema {
    pub world_id: usize,
}
