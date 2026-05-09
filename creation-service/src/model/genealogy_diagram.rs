use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct GetGenealogyDiagramSchema {
    pub diagram_id: usize,
    pub center_entity_id: Option<usize>,
    pub ancestor_depth: Option<usize>,
    pub descendant_depth: Option<usize>,
    pub as_of: Option<NaiveDate>,
}
