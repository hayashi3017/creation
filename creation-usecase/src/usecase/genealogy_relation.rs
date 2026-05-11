use std::collections::{HashMap, HashSet, VecDeque};

use creation_service::model::{
    genealogy_graph::{
        GenealogyGraphEdgePayload, GenealogyGraphNodePayload, GenealogyRelationPathDirection,
        GenealogyRelationPathStepPayload, GenealogyRelationToCenter,
    },
    relationship::RelationshipKind,
};

pub(super) fn apply_center_metadata(
    center_entity_id: Option<usize>,
    nodes: &mut [GenealogyGraphNodePayload],
    edges: &[GenealogyGraphEdgePayload],
) {
    let Some(center_entity_id) = center_entity_id else {
        return;
    };

    let paths = shortest_paths_from_center(center_entity_id, edges);

    for node in nodes {
        if node.entity_id == center_entity_id {
            node.relation_to_center = Some(GenealogyRelationToCenter::Self_);
            node.relation_path_to_center = Some(Vec::new());
            node.generation_offset_from_center = Some(0);
            continue;
        }

        let Some(path) = paths.get(&node.entity_id).cloned() else {
            node.relation_to_center = Some(GenealogyRelationToCenter::Unrelated);
            node.relation_path_to_center = Some(Vec::new());
            node.generation_offset_from_center = None;
            continue;
        };

        let offset = generation_offset(&path);
        node.generation_offset_from_center = offset;
        node.relation_to_center = Some(relation_to_center(&path, offset));
        node.relation_path_to_center = Some(path);
    }
}

fn shortest_paths_from_center(
    center_entity_id: usize,
    edges: &[GenealogyGraphEdgePayload],
) -> HashMap<usize, Vec<GenealogyRelationPathStepPayload>> {
    let mut adjacency = HashMap::<usize, Vec<(usize, GenealogyRelationPathStepPayload)>>::new();

    for edge in edges {
        adjacency.entry(edge.source_entity_id).or_default().push((
            edge.target_entity_id,
            GenealogyRelationPathStepPayload {
                from_entity_id: edge.source_entity_id,
                to_entity_id: edge.target_entity_id,
                edge_id: edge.edge_id.clone(),
                kind: edge.kind,
                direction: GenealogyRelationPathDirection::Forward,
            },
        ));
        adjacency.entry(edge.target_entity_id).or_default().push((
            edge.source_entity_id,
            GenealogyRelationPathStepPayload {
                from_entity_id: edge.target_entity_id,
                to_entity_id: edge.source_entity_id,
                edge_id: edge.edge_id.clone(),
                kind: edge.kind,
                direction: GenealogyRelationPathDirection::Reverse,
            },
        ));
    }

    let mut paths = HashMap::<usize, Vec<GenealogyRelationPathStepPayload>>::new();
    let mut visited = HashSet::from([center_entity_id]);
    let mut queue = VecDeque::from([center_entity_id]);

    while let Some(entity_id) = queue.pop_front() {
        let current_path = paths.get(&entity_id).cloned().unwrap_or_default();
        let mut next_edges = adjacency.remove(&entity_id).unwrap_or_default();
        next_edges
            .sort_unstable_by_key(|(next_entity_id, step)| (*next_entity_id, step.edge_id.clone()));

        for (next_entity_id, step) in next_edges {
            if !visited.insert(next_entity_id) {
                continue;
            }

            let mut next_path = current_path.clone();
            next_path.push(step);
            paths.insert(next_entity_id, next_path);
            queue.push_back(next_entity_id);
        }
    }

    paths
}

fn generation_offset(path: &[GenealogyRelationPathStepPayload]) -> Option<i32> {
    let mut offset = 0_i32;
    for step in path {
        match (step.kind, &step.direction) {
            (
                RelationshipKind::Parent | RelationshipKind::AdoptiveParent,
                GenealogyRelationPathDirection::Forward,
            ) => offset += 1,
            (
                RelationshipKind::Parent | RelationshipKind::AdoptiveParent,
                GenealogyRelationPathDirection::Reverse,
            ) => offset -= 1,
            _ => {}
        }
    }
    Some(offset)
}

fn relation_to_center(
    path: &[GenealogyRelationPathStepPayload],
    offset: Option<i32>,
) -> GenealogyRelationToCenter {
    if path.len() == 1 {
        let step = &path[0];
        return match (step.kind, &step.direction) {
            (
                RelationshipKind::Parent | RelationshipKind::AdoptiveParent,
                GenealogyRelationPathDirection::Reverse,
            ) => GenealogyRelationToCenter::Parent,
            (
                RelationshipKind::Parent | RelationshipKind::AdoptiveParent,
                GenealogyRelationPathDirection::Forward,
            ) => GenealogyRelationToCenter::Child,
            (RelationshipKind::StepParent, GenealogyRelationPathDirection::Reverse) => {
                GenealogyRelationToCenter::StepParent
            }
            (RelationshipKind::StepParent, GenealogyRelationPathDirection::Forward) => {
                GenealogyRelationToCenter::StepChild
            }
            (RelationshipKind::Spouse, _) => GenealogyRelationToCenter::Spouse,
            (RelationshipKind::Partner, _) => GenealogyRelationToCenter::Partner,
            (RelationshipKind::Cohabitant, _) => GenealogyRelationToCenter::Cohabitant,
        };
    }

    if let Some(directions) = effective_lineage_directions(path) {
        return relation_to_center_from_lineage_directions(&directions);
    }

    if path
        .iter()
        .any(|step| step.kind == RelationshipKind::Spouse)
    {
        return GenealogyRelationToCenter::InLaw;
    }

    match offset {
        Some(_) => GenealogyRelationToCenter::Relative,
        None => GenealogyRelationToCenter::Unknown,
    }
}

fn relation_to_center_from_lineage_directions(
    directions: &[GenealogyRelationPathDirection],
) -> GenealogyRelationToCenter {
    if has_direction_pattern(directions, &[GenealogyRelationPathDirection::Reverse]) {
        return GenealogyRelationToCenter::Parent;
    }

    if has_direction_pattern(directions, &[GenealogyRelationPathDirection::Forward]) {
        return GenealogyRelationToCenter::Child;
    }

    if has_direction_pattern(
        directions,
        &[
            GenealogyRelationPathDirection::Reverse,
            GenealogyRelationPathDirection::Forward,
        ],
    ) {
        return GenealogyRelationToCenter::Sibling;
    }

    if has_direction_pattern(
        directions,
        &[
            GenealogyRelationPathDirection::Reverse,
            GenealogyRelationPathDirection::Reverse,
            GenealogyRelationPathDirection::Forward,
        ],
    ) {
        return GenealogyRelationToCenter::UncleOrAunt;
    }

    if has_direction_pattern(
        directions,
        &[
            GenealogyRelationPathDirection::Reverse,
            GenealogyRelationPathDirection::Forward,
            GenealogyRelationPathDirection::Forward,
        ],
    ) {
        return GenealogyRelationToCenter::Nibling;
    }

    if has_direction_pattern(
        directions,
        &[
            GenealogyRelationPathDirection::Reverse,
            GenealogyRelationPathDirection::Reverse,
            GenealogyRelationPathDirection::Forward,
            GenealogyRelationPathDirection::Forward,
        ],
    ) {
        return GenealogyRelationToCenter::Cousin;
    }

    if has_only_direction(directions, GenealogyRelationPathDirection::Reverse) {
        return GenealogyRelationToCenter::Ancestor;
    }

    if has_only_direction(directions, GenealogyRelationPathDirection::Forward) {
        return GenealogyRelationToCenter::Descendant;
    }

    GenealogyRelationToCenter::Relative
}

fn effective_lineage_directions(
    path: &[GenealogyRelationPathStepPayload],
) -> Option<Vec<GenealogyRelationPathDirection>> {
    let mut directions = Vec::new();

    for step in path {
        if is_lineage_step(step) {
            directions.push(clone_direction(&step.direction));
            continue;
        }

        if step.kind == RelationshipKind::Spouse && is_ancestor_side_spouse_bridge(&directions) {
            continue;
        }

        return None;
    }

    Some(directions)
}

fn is_ancestor_side_spouse_bridge(directions: &[GenealogyRelationPathDirection]) -> bool {
    !directions.is_empty()
        && directions
            .iter()
            .all(|direction| matches!(direction, GenealogyRelationPathDirection::Reverse))
}

fn has_direction_pattern(
    directions: &[GenealogyRelationPathDirection],
    expected: &[GenealogyRelationPathDirection],
) -> bool {
    directions.len() == expected.len()
        && directions
            .iter()
            .zip(expected)
            .all(|(direction, expected)| same_direction(direction, expected))
}

fn has_only_direction(
    directions: &[GenealogyRelationPathDirection],
    direction: GenealogyRelationPathDirection,
) -> bool {
    !directions.is_empty()
        && directions
            .iter()
            .all(|current| same_direction(current, &direction))
}

fn is_lineage_step(step: &GenealogyRelationPathStepPayload) -> bool {
    matches!(
        step.kind,
        RelationshipKind::Parent | RelationshipKind::AdoptiveParent
    )
}

fn same_direction(
    direction: &GenealogyRelationPathDirection,
    expected: &GenealogyRelationPathDirection,
) -> bool {
    matches!(
        (direction, expected),
        (
            GenealogyRelationPathDirection::Forward,
            GenealogyRelationPathDirection::Forward
        ) | (
            GenealogyRelationPathDirection::Reverse,
            GenealogyRelationPathDirection::Reverse
        )
    )
}

fn clone_direction(direction: &GenealogyRelationPathDirection) -> GenealogyRelationPathDirection {
    match direction {
        GenealogyRelationPathDirection::Forward => GenealogyRelationPathDirection::Forward,
        GenealogyRelationPathDirection::Reverse => GenealogyRelationPathDirection::Reverse,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use creation_service::model::genealogy_graph::{
        GenealogyGraphEdgeSource, GenealogyGraphSourceConfidence,
    };

    #[test]
    fn relation_to_center_classifies_all_supported_relations() {
        let cases = vec![
            (
                "parent",
                vec![step(
                    RelationshipKind::Parent,
                    GenealogyRelationPathDirection::Reverse,
                )],
                GenealogyRelationToCenter::Parent,
            ),
            (
                "child",
                vec![step(
                    RelationshipKind::Parent,
                    GenealogyRelationPathDirection::Forward,
                )],
                GenealogyRelationToCenter::Child,
            ),
            (
                "ancestor",
                lineage_path(&[
                    GenealogyRelationPathDirection::Reverse,
                    GenealogyRelationPathDirection::Reverse,
                ]),
                GenealogyRelationToCenter::Ancestor,
            ),
            (
                "descendant",
                lineage_path(&[
                    GenealogyRelationPathDirection::Forward,
                    GenealogyRelationPathDirection::Forward,
                ]),
                GenealogyRelationToCenter::Descendant,
            ),
            (
                "sibling",
                lineage_path(&[
                    GenealogyRelationPathDirection::Reverse,
                    GenealogyRelationPathDirection::Forward,
                ]),
                GenealogyRelationToCenter::Sibling,
            ),
            (
                "spouse",
                vec![step(
                    RelationshipKind::Spouse,
                    GenealogyRelationPathDirection::Forward,
                )],
                GenealogyRelationToCenter::Spouse,
            ),
            (
                "partner",
                vec![step(
                    RelationshipKind::Partner,
                    GenealogyRelationPathDirection::Forward,
                )],
                GenealogyRelationToCenter::Partner,
            ),
            (
                "cohabitant",
                vec![step(
                    RelationshipKind::Cohabitant,
                    GenealogyRelationPathDirection::Forward,
                )],
                GenealogyRelationToCenter::Cohabitant,
            ),
            (
                "uncle_or_aunt",
                lineage_path(&[
                    GenealogyRelationPathDirection::Reverse,
                    GenealogyRelationPathDirection::Reverse,
                    GenealogyRelationPathDirection::Forward,
                ]),
                GenealogyRelationToCenter::UncleOrAunt,
            ),
            (
                "nibling",
                lineage_path(&[
                    GenealogyRelationPathDirection::Reverse,
                    GenealogyRelationPathDirection::Forward,
                    GenealogyRelationPathDirection::Forward,
                ]),
                GenealogyRelationToCenter::Nibling,
            ),
            (
                "cousin",
                lineage_path(&[
                    GenealogyRelationPathDirection::Reverse,
                    GenealogyRelationPathDirection::Reverse,
                    GenealogyRelationPathDirection::Forward,
                    GenealogyRelationPathDirection::Forward,
                ]),
                GenealogyRelationToCenter::Cousin,
            ),
            (
                "in_law",
                vec![
                    step(
                        RelationshipKind::Spouse,
                        GenealogyRelationPathDirection::Forward,
                    ),
                    step(
                        RelationshipKind::Parent,
                        GenealogyRelationPathDirection::Reverse,
                    ),
                ],
                GenealogyRelationToCenter::InLaw,
            ),
            (
                "spouse_side_parent",
                vec![
                    step(
                        RelationshipKind::Parent,
                        GenealogyRelationPathDirection::Reverse,
                    ),
                    step(
                        RelationshipKind::Spouse,
                        GenealogyRelationPathDirection::Forward,
                    ),
                ],
                GenealogyRelationToCenter::Parent,
            ),
            (
                "spouse_side_ancestor",
                vec![
                    step(
                        RelationshipKind::Parent,
                        GenealogyRelationPathDirection::Reverse,
                    ),
                    step(
                        RelationshipKind::Spouse,
                        GenealogyRelationPathDirection::Forward,
                    ),
                    step(
                        RelationshipKind::Parent,
                        GenealogyRelationPathDirection::Reverse,
                    ),
                ],
                GenealogyRelationToCenter::Ancestor,
            ),
            (
                "spouse_side_uncle_or_aunt",
                vec![
                    step(
                        RelationshipKind::Parent,
                        GenealogyRelationPathDirection::Reverse,
                    ),
                    step(
                        RelationshipKind::Spouse,
                        GenealogyRelationPathDirection::Forward,
                    ),
                    step(
                        RelationshipKind::Parent,
                        GenealogyRelationPathDirection::Reverse,
                    ),
                    step(
                        RelationshipKind::Parent,
                        GenealogyRelationPathDirection::Forward,
                    ),
                ],
                GenealogyRelationToCenter::UncleOrAunt,
            ),
            (
                "spouse_side_sibling",
                vec![
                    step(
                        RelationshipKind::Parent,
                        GenealogyRelationPathDirection::Reverse,
                    ),
                    step(
                        RelationshipKind::Spouse,
                        GenealogyRelationPathDirection::Forward,
                    ),
                    step(
                        RelationshipKind::Parent,
                        GenealogyRelationPathDirection::Forward,
                    ),
                ],
                GenealogyRelationToCenter::Sibling,
            ),
            (
                "step_parent",
                vec![step(
                    RelationshipKind::StepParent,
                    GenealogyRelationPathDirection::Reverse,
                )],
                GenealogyRelationToCenter::StepParent,
            ),
            (
                "step_child",
                vec![step(
                    RelationshipKind::StepParent,
                    GenealogyRelationPathDirection::Forward,
                )],
                GenealogyRelationToCenter::StepChild,
            ),
            (
                "relative",
                vec![
                    step(
                        RelationshipKind::Partner,
                        GenealogyRelationPathDirection::Forward,
                    ),
                    step(
                        RelationshipKind::Parent,
                        GenealogyRelationPathDirection::Reverse,
                    ),
                ],
                GenealogyRelationToCenter::Relative,
            ),
        ];

        for (name, path, expected) in cases {
            assert_eq!(
                relation_to_center(&path, generation_offset(&path)),
                expected,
                "{name}"
            );
        }

        let unknown_path = vec![
            step(
                RelationshipKind::Partner,
                GenealogyRelationPathDirection::Forward,
            ),
            step(
                RelationshipKind::Cohabitant,
                GenealogyRelationPathDirection::Forward,
            ),
        ];
        assert_eq!(
            relation_to_center(&unknown_path, None),
            GenealogyRelationToCenter::Unknown
        );
    }

    #[test]
    fn apply_center_metadata_marks_self_and_unrelated_nodes() {
        let mut nodes = vec![node(1), node(2), node(3)];
        let edges = vec![edge(2, 1, RelationshipKind::Parent)];

        apply_center_metadata(Some(1), &mut nodes, &edges);

        assert_eq!(
            nodes[0].relation_to_center,
            Some(GenealogyRelationToCenter::Self_)
        );
        assert_eq!(
            nodes[1].relation_to_center,
            Some(GenealogyRelationToCenter::Parent)
        );
        assert_eq!(
            nodes[2].relation_to_center,
            Some(GenealogyRelationToCenter::Unrelated)
        );
    }

    #[test]
    fn apply_center_metadata_classifies_spouse_side_lineage() {
        let mut nodes = vec![node(1), node(2), node(3), node(4), node(5)];
        let edges = vec![
            edge(2, 1, RelationshipKind::Parent),
            edge(2, 3, RelationshipKind::Spouse),
            edge(4, 3, RelationshipKind::Parent),
            edge(4, 5, RelationshipKind::Parent),
        ];

        apply_center_metadata(Some(1), &mut nodes, &edges);

        assert_eq!(
            nodes[2].relation_to_center,
            Some(GenealogyRelationToCenter::Parent)
        );
        assert_eq!(nodes[2].generation_offset_from_center, Some(-1));
        assert_eq!(
            nodes[3].relation_to_center,
            Some(GenealogyRelationToCenter::Ancestor)
        );
        assert_eq!(nodes[3].generation_offset_from_center, Some(-2));
        assert_eq!(
            nodes[4].relation_to_center,
            Some(GenealogyRelationToCenter::UncleOrAunt)
        );
        assert_eq!(nodes[4].generation_offset_from_center, Some(-1));
    }

    fn lineage_path(
        directions: &[GenealogyRelationPathDirection],
    ) -> Vec<GenealogyRelationPathStepPayload> {
        directions
            .iter()
            .map(|direction| {
                step(
                    RelationshipKind::Parent,
                    match direction {
                        GenealogyRelationPathDirection::Forward => {
                            GenealogyRelationPathDirection::Forward
                        }
                        GenealogyRelationPathDirection::Reverse => {
                            GenealogyRelationPathDirection::Reverse
                        }
                    },
                )
            })
            .collect()
    }

    fn step(
        kind: RelationshipKind,
        direction: GenealogyRelationPathDirection,
    ) -> GenealogyRelationPathStepPayload {
        GenealogyRelationPathStepPayload {
            from_entity_id: 1,
            to_entity_id: 2,
            edge_id: "edge".to_string(),
            kind,
            direction,
        }
    }

    fn node(entity_id: usize) -> GenealogyGraphNodePayload {
        GenealogyGraphNodePayload {
            entity_id,
            name: entity_id.to_string(),
            description: None,
            gender: None,
            birth_date: None,
            death_date: None,
            birthplace: None,
            residence: None,
            photo_url: None,
            source_diagram_ids: vec![1],
            parent_entity_ids: Vec::new(),
            child_entity_ids: Vec::new(),
            is_root: false,
            relation_to_center: None,
            relation_path_to_center: None,
            generation_offset_from_center: None,
        }
    }

    fn edge(
        source_entity_id: usize,
        target_entity_id: usize,
        kind: RelationshipKind,
    ) -> GenealogyGraphEdgePayload {
        GenealogyGraphEdgePayload {
            edge_id: format!("edge:{source_entity_id}:{target_entity_id}"),
            source_entity_id,
            target_entity_id,
            kind,
            source: GenealogyGraphEdgeSource::Explicit,
            source_confidence: GenealogyGraphSourceConfidence::Confirmed,
            start_date: None,
            end_date: None,
            end_reason: None,
            notes: None,
            source_relationship_ids: Vec::new(),
            source_diagram_ids: vec![1],
        }
    }
}
