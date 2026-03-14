use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Entity {
    pub id: usize,
    pub diagram_id: usize,
    pub kind: EntityKind,
    pub name: String,
    pub description: Option<String>,
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
    pub description: String,
}

impl Display for CreateEntitySchema {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateEntitySchema {
    pub id: usize,
    pub diagram_id: usize,
    pub kind: EntityKind,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteEntitySchema {
    pub id: usize,
}
