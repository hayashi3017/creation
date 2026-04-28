use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;

pub const ENTITY_NAME_MAX_CHARS: usize = 255;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Entity {
    pub entity_id: usize,
    pub diagram_id: usize,
    pub kind: EntityKind,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedEntity {
    pub entity_id: usize,
    pub diagram_id: usize,
}

#[derive(Deserialize, Serialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "entity_kind")]
#[sqlx(rename_all = "snake_case")]
pub enum EntityKind {
    Person,
}

impl Debug for EntityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Person => write!(f, "PERSON"),
        }
    }
}

impl Clone for EntityKind {
    fn clone(&self) -> Self {
        match self {
            Self::Person => Self::Person,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetEntitiesSchema {
    pub diagram_id: usize,
}

#[derive(Debug, Deserialize)]
pub struct CreateEntitySchema {
    pub diagram_id: usize,
    pub kind: EntityKind,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

impl Display for CreateEntitySchema {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateEntitySchema {
    pub entity_id: usize,
    pub diagram_id: usize,
    pub kind: EntityKind,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteEntitySchema {
    pub entity_id: usize,
}

#[derive(Debug, Clone)]
pub struct DeleteDiagramEntityMembershipsSchema {
    pub diagram_id: usize,
}

#[derive(Debug, Clone)]
pub struct LoadSeedEntitiesSchema {
    pub entity_ids: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct LoadActiveEntityIdsSchema {
    pub diagram_id: usize,
}

#[derive(Debug, Clone)]
pub struct LoadActiveEntitiesByDiagramIdsSchema {
    pub diagram_ids: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct LoadEntitiesByDiagramIdsSchema {
    pub diagram_ids: Vec<usize>,
}
