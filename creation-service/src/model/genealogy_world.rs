use chrono::NaiveDate;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct GetGenealogyWorldSchema {
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
