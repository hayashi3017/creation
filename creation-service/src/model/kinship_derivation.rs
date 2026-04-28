use crate::model::relationship::RelationshipKind;

#[derive(Debug, Clone)]
pub struct DeriveKinshipInput {
    pub active_person_ids: Vec<usize>,
    pub explicit_relationships: Vec<crate::model::relationship::Relationship>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KinshipDerivation {
    pub lineage_edges: Vec<CanonicalLineageEdge>,
    pub relations: Vec<KinshipRelation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalLineageEdge {
    pub relationship_id: usize,
    pub parent_entity_id: usize,
    pub child_entity_id: usize,
    pub kind: RelationshipKind,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub end_reason: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KinshipRelation {
    pub from_entity_id: usize,
    pub to_entity_id: usize,
    pub kind: KinshipRelationKind,
    pub source: GenealogyRelationshipSource,
    pub explicit_relationship_id: Option<usize>,
    pub sibling_kind: Option<GenealogySiblingKind>,
    pub generation_distance: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KinshipRelationKind {
    Parent,
    AdoptiveParent,
    Child,
    AdoptiveChild,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenealogyRelationshipSource {
    Explicit,
    Derived,
    Suggested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenealogySiblingKind {
    Full,
    Half,
    Adoptive,
    Step,
}
