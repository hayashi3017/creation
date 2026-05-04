use chrono::NaiveDate;
use serde::Serialize;
use utoipa::ToSchema;

use crate::model::{diagram::Diagram, person::GenderKind, relationship::RelationshipKind};

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GenealogyDiagramGraph {
    pub diagram: Diagram,
    pub as_of: Option<NaiveDate>,
    pub root_entity_ids: Vec<usize>,
    pub nodes: Vec<GenealogyDiagramNode>,
    pub edges: Vec<GenealogyDiagramEdge>,
    pub stats: GenealogyDiagramStats,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GenealogyDiagramNode {
    pub entity_id: usize,
    pub diagram_id: usize,
    pub name: String,
    pub description: Option<String>,
    pub gender: Option<GenderKind>,
    pub birth_date: Option<NaiveDate>,
    pub death_date: Option<NaiveDate>,
    pub birthplace: Option<String>,
    pub residence: Option<String>,
    pub photo_url: Option<String>,
    pub parent_entity_ids: Vec<usize>,
    pub child_entity_ids: Vec<usize>,
    pub is_root: bool,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GenealogyDiagramEdge {
    pub relationship_id: usize,
    pub source_entity_id: usize,
    pub target_entity_id: usize,
    pub kind: RelationshipKind,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub end_reason: Option<String>,
    pub notes: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GenealogyDiagramStats {
    pub person_count: usize,
    pub edge_count: usize,
    pub root_count: usize,
}

#[derive(Debug, Clone)]
pub struct GetGenealogyDiagramSchema {
    pub diagram_id: usize,
    pub center_entity_id: Option<usize>,
    pub ancestor_depth: Option<usize>,
    pub descendant_depth: Option<usize>,
    pub as_of: Option<NaiveDate>,
}
