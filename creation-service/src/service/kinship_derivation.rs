use std::collections::HashSet;

use async_trait::async_trait;

use crate::model::{
    kinship_derivation::{
        CanonicalLineageEdge, DeriveKinshipInput, GenealogyRelationshipSource, KinshipDerivation,
        KinshipRelation, KinshipRelationKind,
    },
    relationship::RelationshipKind,
};

#[async_trait]
pub trait KinshipDerivationService: Send + Sync + 'static {}

#[async_trait]
pub trait UsesKinshipDerivationService: Send + Sync + 'static {
    async fn derive_kinship(&self, input: DeriveKinshipInput) -> KinshipDerivation;
}

#[async_trait]
impl<T: KinshipDerivationService> UsesKinshipDerivationService for T {
    async fn derive_kinship(&self, input: DeriveKinshipInput) -> KinshipDerivation {
        let active_person_ids = input.active_person_ids.into_iter().collect::<HashSet<_>>();
        let mut lineage_edges = Vec::new();
        let mut relations = Vec::new();

        for relationship in input.explicit_relationships {
            if !relationship.kind.is_tree_edge()
                || !active_person_ids.contains(&relationship.source_entity_id)
                || !active_person_ids.contains(&relationship.target_entity_id)
            {
                continue;
            }

            let lineage_edge = CanonicalLineageEdge {
                relationship_id: relationship.relationship_id,
                parent_entity_id: relationship.source_entity_id,
                child_entity_id: relationship.target_entity_id,
                kind: relationship.kind,
                start_date: relationship.start_date,
                end_date: relationship.end_date,
                end_reason: relationship.end_reason,
                notes: relationship.notes,
            };

            relations.extend(expand_lineage_edge(&lineage_edge));
            lineage_edges.push(lineage_edge);
        }

        KinshipDerivation {
            lineage_edges,
            relations,
        }
    }
}

pub trait ProvidesKinshipDerivationService: Send + Sync + 'static {
    type T: KinshipDerivationService;
    fn kinship_derivation_service(&self) -> &Self::T;
}

fn expand_lineage_edge(edge: &CanonicalLineageEdge) -> [KinshipRelation; 2] {
    let explicit_kind = match edge.kind {
        RelationshipKind::Parent => KinshipRelationKind::Parent,
        RelationshipKind::AdoptiveParent => KinshipRelationKind::AdoptiveParent,
        _ => unreachable!("non-tree-edge kinds are filtered before expansion"),
    };
    let inverse_kind = match edge.kind {
        RelationshipKind::Parent => KinshipRelationKind::Child,
        RelationshipKind::AdoptiveParent => KinshipRelationKind::AdoptiveChild,
        _ => unreachable!("non-tree-edge kinds are filtered before expansion"),
    };

    [
        KinshipRelation {
            from_entity_id: edge.parent_entity_id,
            to_entity_id: edge.child_entity_id,
            kind: explicit_kind,
            source: GenealogyRelationshipSource::Explicit,
            explicit_relationship_id: Some(edge.relationship_id),
            sibling_kind: None,
            generation_distance: Some(1),
        },
        KinshipRelation {
            from_entity_id: edge.child_entity_id,
            to_entity_id: edge.parent_entity_id,
            kind: inverse_kind,
            source: GenealogyRelationshipSource::Derived,
            explicit_relationship_id: Some(edge.relationship_id),
            sibling_kind: None,
            generation_distance: Some(1),
        },
    ]
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use crate::model::{
        kinship_derivation::DeriveKinshipInput,
        relationship::{Relationship, RelationshipKind},
    };

    struct TestService;

    impl KinshipDerivationService for TestService {}

    #[test]
    fn derive_kinship_returns_canonical_lineage_edges_and_inverse_relations() {
        let service = TestService;

        let ret = futures::executor::block_on(service.derive_kinship(DeriveKinshipInput {
            active_person_ids: vec![1, 2],
            explicit_relationships: vec![Relationship {
                relationship_id: 10,
                diagram_id: 1,
                source_entity_id: 1,
                target_entity_id: 2,
                kind: RelationshipKind::Parent,
                start_date: Some(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()),
                end_date: None,
                end_reason: None,
                notes: Some("lineage".to_string()),
            }],
        }));

        assert_eq!(ret.lineage_edges.len(), 1);
        assert_eq!(ret.lineage_edges[0].parent_entity_id, 1);
        assert_eq!(ret.lineage_edges[0].child_entity_id, 2);
        assert_eq!(ret.relations.len(), 2);
        assert_eq!(ret.relations[0].kind, KinshipRelationKind::Parent);
        assert_eq!(
            ret.relations[0].source,
            GenealogyRelationshipSource::Explicit
        );
        assert_eq!(ret.relations[1].kind, KinshipRelationKind::Child);
        assert_eq!(
            ret.relations[1].source,
            GenealogyRelationshipSource::Derived
        );
    }

    #[test]
    fn derive_kinship_filters_out_of_scope_and_non_tree_edges() {
        let service = TestService;

        let ret = futures::executor::block_on(service.derive_kinship(DeriveKinshipInput {
            active_person_ids: vec![1, 2],
            explicit_relationships: vec![
                Relationship {
                    relationship_id: 10,
                    diagram_id: 1,
                    source_entity_id: 1,
                    target_entity_id: 3,
                    kind: RelationshipKind::Parent,
                    start_date: None,
                    end_date: None,
                    end_reason: None,
                    notes: None,
                },
                Relationship {
                    relationship_id: 11,
                    diagram_id: 1,
                    source_entity_id: 1,
                    target_entity_id: 2,
                    kind: RelationshipKind::Spouse,
                    start_date: None,
                    end_date: None,
                    end_reason: None,
                    notes: None,
                },
            ],
        }));

        assert!(ret.lineage_edges.is_empty());
        assert!(ret.relations.is_empty());
    }
}
