use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use utoipa::ToSchema;

pub const DIAGRAM_NAME_MAX_CHARS: usize = 255;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct Diagram {
    #[serde(rename = "id")]
    pub diagram_id: usize,
    pub name: String,
    pub kind: DiagramKind,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Type, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "diagram_kind")]
#[sqlx(rename_all = "snake_case")]
pub enum DiagramKind {
    FamilyTree,
    Correlation,
}

impl Debug for DiagramKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FamilyTree => write!(f, "FAMILY TREE"),
            Self::Correlation => write!(f, "CORRELATION"),
        }
    }
}

impl Clone for DiagramKind {
    fn clone(&self) -> Self {
        match self {
            Self::FamilyTree => Self::FamilyTree,
            Self::Correlation => Self::Correlation,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetDiagramsSchema {}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDiagramSchema {
    pub name: String,
    pub kind: DiagramKind,
    #[serde(default)]
    pub description: Option<String>,
}

impl Display for CreateDiagramSchema {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDiagramSchema {
    pub diagram_id: usize,
    pub name: String,
    pub kind: DiagramKind,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteDiagramSchema {
    pub diagram_id: usize,
}

#[derive(Debug, Clone)]
pub struct ExistsActiveDiagramSchema {
    pub diagram_id: usize,
}

#[derive(Debug, Clone)]
pub struct GetDiagramSchema {
    pub diagram_id: usize,
}
