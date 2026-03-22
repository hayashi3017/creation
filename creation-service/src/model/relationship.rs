use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use utoipa::ToSchema;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct Relationship {
    pub id: usize,
    pub diagram_id: usize,
    pub source_entity_id: usize,
    pub target_entity_id: usize,
    pub kind: RelationshipKind,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationshipEndpoints {
    pub source_entity_id: usize,
    pub target_entity_id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatedRelationshipEndpoints {
    pub previous_source_entity_id: usize,
    pub previous_target_entity_id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationshipEdge {
    pub ancestor_id: usize,
    pub descendant_id: usize,
}

// bulk load 後に service 側で diagram ごとへ再グループ化するための内部表現。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramRelationshipEdge {
    pub diagram_id: usize,
    pub ancestor_id: usize,
    pub descendant_id: usize,
}

#[derive(Debug, Deserialize, Serialize, Type, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "relationship_kind")]
#[sqlx(rename_all = "snake_case")]
pub enum RelationshipKind {
    Parent,
    Child,
    Sibling,
    Spouse,
    AdoptedParent,
    AdoptedChild,
    DivorcedSpouse,
    Cohabitant,
    StepParent,
    StepChild,
}

impl RelationshipKind {
    pub fn is_lineage(&self) -> bool {
        matches!(self, Self::Parent | Self::Child)
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GetRelationshipsSchema {
    pub diagram_id: usize,
}

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct CreateRelationshipSchema {
    pub diagram_id: usize,
    pub source_entity_id: usize,
    pub target_entity_id: usize,
    pub kind: RelationshipKind,
    #[serde(default)]
    pub start_date: Option<NaiveDate>,
    #[serde(default)]
    pub end_date: Option<NaiveDate>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct UpdateRelationshipSchema {
    pub id: usize,
    pub source_entity_id: usize,
    pub target_entity_id: usize,
    pub kind: RelationshipKind,
    #[serde(default)]
    pub start_date: Option<NaiveDate>,
    #[serde(default)]
    pub end_date: Option<NaiveDate>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteRelationshipSchema {
    pub id: usize,
}

#[derive(Debug, Clone)]
pub struct DeleteRelationshipsForEntitySchema {
    pub entity_id: usize,
}

#[derive(Debug, Clone)]
pub struct LoadRelationshipEdgesSchema {
    pub diagram_id: usize,
}

#[derive(Debug, Clone)]
pub struct LoadRelationshipEdgesByDiagramIdsSchema {
    pub diagram_ids: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct LoadRelationshipDiagramIdSchema {
    pub id: usize,
}
