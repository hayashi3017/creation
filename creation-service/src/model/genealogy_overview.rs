use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::{person::GenderKind, relationship::RelationshipKind, world::World};

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GenealogyOverview {
    pub world: GenealogyOverviewWorld,
    pub diagram_ids: Vec<usize>,
    pub as_of: Option<NaiveDate>,
    pub nodes: Vec<GenealogyOverviewNode>,
    pub edges: Vec<GenealogyOverviewEdge>,
    pub root_entity_ids: Vec<usize>,
    pub stats: GenealogyOverviewStats,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GenealogyOverviewWorld {
    pub world_id: usize,
    pub name: String,
}

impl From<World> for GenealogyOverviewWorld {
    fn from(world: World) -> Self {
        Self {
            world_id: world.world_id,
            name: world.name,
        }
    }
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GenealogyOverviewNode {
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
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GenealogyOverviewEdge {
    pub from_entity_id: usize,
    pub to_entity_id: usize,
    pub kind: RelationshipKind,
    pub source: GenealogyOverviewEdgeSource,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub end_reason: Option<String>,
    pub source_relationship_ids: Vec<usize>,
    pub source_diagram_ids: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GenealogyOverviewEdgeSource {
    Explicit,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GenealogyOverviewStats {
    pub diagram_count: usize,
    pub node_count: usize,
    pub edge_count: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GetGenealogyOverviewSchema {
    pub world_id: usize,
    #[serde(default)]
    pub center_entity_id: Option<usize>,
    #[serde(default)]
    pub ancestor_depth: Option<usize>,
    #[serde(default)]
    pub descendant_depth: Option<usize>,
    #[serde(default)]
    pub diagram_ids: Option<Vec<usize>>,
    #[serde(default)]
    pub as_of: Option<NaiveDate>,
}
