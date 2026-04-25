use chrono::NaiveDate;
use serde::Serialize;
use utoipa::ToSchema;

use crate::model::{diagram::Diagram, person::GenderKind, relationship::RelationshipKind};

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FamilyTree {
    pub diagram: Diagram,
    pub root_entity_ids: Vec<usize>,
    pub nodes: Vec<FamilyTreeNode>,
    pub edges: Vec<FamilyTreeEdge>,
    pub stats: FamilyTreeStats,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FamilyTreeNode {
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
pub struct FamilyTreeEdge {
    pub relationship_id: usize,
    pub parent_entity_id: usize,
    pub child_entity_id: usize,
    pub kind: RelationshipKind,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub end_reason: Option<String>,
    pub notes: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FamilyTreeStats {
    pub person_count: usize,
    pub edge_count: usize,
    pub root_count: usize,
}

#[derive(Debug, Clone)]
pub struct GetFamilyTreeSchema {
    pub diagram_id: usize,
}
