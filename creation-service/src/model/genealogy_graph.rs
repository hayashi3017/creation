use chrono::NaiveDate;
use serde::Serialize;
use utoipa::ToSchema;

use crate::model::{person::GenderKind, relationship::RelationshipKind};

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GenealogyGraphPayload {
    pub as_of: Option<NaiveDate>,
    pub context: GenealogyGraphContextPayload,
    pub center_entity_id: Option<usize>,
    pub root_entity_ids: Vec<usize>,
    pub nodes: Vec<GenealogyGraphNodePayload>,
    pub edges: Vec<GenealogyGraphEdgePayload>,
    pub stats: GenealogyGraphStatsPayload,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GenealogyGraphContextPayload {
    Diagram {
        diagram_id: usize,
        world_id: usize,
        diagram_ids: Vec<usize>,
        name: String,
    },
    World {
        world_id: usize,
        diagram_ids: Vec<usize>,
        name: String,
    },
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GenealogyGraphNodePayload {
    pub entity_id: usize,
    pub name: String,
    pub description: Option<String>,
    pub gender: Option<GenderKind>,
    pub birth_date: Option<NaiveDate>,
    pub death_date: Option<NaiveDate>,
    pub birthplace: Option<String>,
    pub residence: Option<String>,
    pub photo_url: Option<String>,
    pub source_diagram_ids: Vec<usize>,
    pub parent_entity_ids: Vec<usize>,
    pub child_entity_ids: Vec<usize>,
    pub is_root: bool,
    pub relation_to_center: Option<GenealogyRelationToCenter>,
    pub relation_path_to_center: Option<Vec<GenealogyRelationPathStepPayload>>,
    pub generation_offset_from_center: Option<i32>,
}

#[derive(Debug, Clone, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GenealogyRelationToCenter {
    #[serde(rename = "self")]
    Self_,
    Parent,
    Child,
    Ancestor,
    Descendant,
    Sibling,
    Spouse,
    Partner,
    Cohabitant,
    UncleOrAunt,
    Nibling,
    Cousin,
    InLaw,
    StepParent,
    StepChild,
    Relative,
    Unrelated,
    Unknown,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GenealogyRelationPathStepPayload {
    pub from_entity_id: usize,
    pub to_entity_id: usize,
    pub edge_id: String,
    pub kind: RelationshipKind,
    pub direction: GenealogyRelationPathDirection,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GenealogyRelationPathDirection {
    Forward,
    Reverse,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GenealogyGraphEdgePayload {
    pub edge_id: String,
    pub source_entity_id: usize,
    pub target_entity_id: usize,
    pub kind: RelationshipKind,
    pub source: GenealogyGraphEdgeSource,
    pub source_confidence: GenealogyGraphSourceConfidence,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub end_reason: Option<String>,
    pub notes: Option<String>,
    pub source_relationship_ids: Vec<usize>,
    pub source_diagram_ids: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GenealogyGraphEdgeSource {
    Explicit,
    Derived,
    Suggested,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GenealogyGraphSourceConfidence {
    Confirmed,
    Inferred,
    Conflicting,
    Unknown,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GenealogyGraphStatsPayload {
    pub node_count: usize,
    pub edge_count: usize,
    pub root_count: usize,
    pub diagram_count: usize,
}
