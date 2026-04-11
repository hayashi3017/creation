use std::collections::{HashMap, HashSet, VecDeque};

use async_trait::async_trait;
use thiserror::Error;

use crate::{
    model::{
        entity::{LoadActiveEntitiesByDiagramIdsSchema, LoadSeedEntitiesSchema, SeedEntity},
        relationship::{
            DiagramRelationshipEdge, LoadRelationshipEdgesByDiagramIdsSchema, RelationshipEdge,
        },
        tree_path::{
            CreateTreePathsSchema, DeleteTreePathsByEntityIdsSchema,
            LoadStaleRelatedConnectionsSchema, SyncTreePathsByEntityIdsSchema, TreePath,
            TreePathConnection,
        },
    },
    repository::{
        entity::{
            LoadActiveEntitiesByDiagramIdsRepositoryError, LoadSeedEntitiesRepositoryError,
            ProvidesEntityRepository, UsesEntityRepository,
        },
        relationship::{
            LoadRelationshipEdgesByDiagramIdsRepositoryError, ProvidesRelationshipRepository,
            UsesRelationshipRepository,
        },
        tree_path::{
            CreateTreePathsRepositoryError, DeleteTreePathsByEntityIdsRepositoryError,
            LoadStaleRelatedConnectionsRepositoryError, ProvidesTreePathRepository,
            UsesTreePathRepository,
        },
    },
};

#[async_trait]
pub trait TreePathService:
    ProvidesEntityRepository + ProvidesRelationshipRepository + ProvidesTreePathRepository
{
}

#[derive(Debug, Error)]
pub enum SyncTreePathsServiceError {
    #[error(transparent)]
    LoadSeedEntitiesRepositoryError(#[from] LoadSeedEntitiesRepositoryError),
    #[error(transparent)]
    LoadActiveEntitiesByDiagramIdsRepositoryError(
        #[from] LoadActiveEntitiesByDiagramIdsRepositoryError,
    ),
    #[error(transparent)]
    LoadRelationshipEdgesByDiagramIdsRepositoryError(
        #[from] LoadRelationshipEdgesByDiagramIdsRepositoryError,
    ),
    #[error(transparent)]
    LoadStaleRelatedConnectionsRepositoryError(#[from] LoadStaleRelatedConnectionsRepositoryError),
    #[error(transparent)]
    DeleteTreePathsByEntityIdsRepositoryError(#[from] DeleteTreePathsByEntityIdsRepositoryError),
    #[error(transparent)]
    CreateTreePathsRepositoryError(#[from] CreateTreePathsRepositoryError),
    #[error("cycle detected")]
    CycleDetected,
}

#[async_trait]
pub trait UsesTreePathService {
    async fn sync_tree_paths_by_entity_ids(
        &self,
        body: SyncTreePathsByEntityIdsSchema,
    ) -> Result<(), SyncTreePathsServiceError>;
}

#[async_trait]
impl<T: TreePathService> UsesTreePathService for T {
    async fn sync_tree_paths_by_entity_ids(
        &self,
        body: SyncTreePathsByEntityIdsSchema,
    ) -> Result<(), SyncTreePathsServiceError> {
        let entity_ids = normalize_entity_ids(body.entity_ids);

        if entity_ids.is_empty() {
            return Ok(());
        }

        let seed_entities = self
            .entity_repository()
            .load_seed_entities(LoadSeedEntitiesSchema { entity_ids })
            .await?;
        let mut seed_entity_ids_by_diagram_id = group_seed_entity_ids(seed_entities);

        if seed_entity_ids_by_diagram_id.is_empty() {
            return Ok(());
        }

        // read は diagram ごとの N+1 を避けるため一括取得し、service 内で diagram 単位へ戻す。
        let diagram_ids = sorted_diagram_ids(&seed_entity_ids_by_diagram_id);
        let mut active_entity_ids_by_diagram_id = group_seed_entity_ids(
            self.entity_repository()
                .load_active_entities_by_diagram_ids(LoadActiveEntitiesByDiagramIdsSchema {
                    diagram_ids: diagram_ids.clone(),
                })
                .await?,
        );
        let mut relationship_edges_by_diagram_id = group_relationship_edges_by_diagram_id(
            self.relationship_repository()
                .load_relationship_edges_by_diagram_ids(LoadRelationshipEdgesByDiagramIdsSchema {
                    diagram_ids,
                })
                .await?,
        );
        let mut stale_related_entity_ids_by_diagram_id =
            group_stale_related_entity_ids_by_diagram_id(
                &seed_entity_ids_by_diagram_id,
                self.tree_path_repository()
                    .load_stale_related_connections(LoadStaleRelatedConnectionsSchema {
                        entity_ids: seed_entity_ids_by_diagram_id
                            .values()
                            .flat_map(|entity_ids| entity_ids.iter().copied())
                            .collect(),
                    })
                    .await?,
            );

        for diagram_id in sorted_diagram_ids(&seed_entity_ids_by_diagram_id) {
            let seed_entity_ids = seed_entity_ids_by_diagram_id
                .remove(&diagram_id)
                .unwrap_or_default();
            let active_entity_ids = active_entity_ids_by_diagram_id
                .remove(&diagram_id)
                .unwrap_or_default();
            let edges = relationship_edges_by_diagram_id
                .remove(&diagram_id)
                .unwrap_or_default();
            let stale_related_entity_ids = stale_related_entity_ids_by_diagram_id
                .remove(&diagram_id)
                .unwrap_or_default();

            let affected_entity_ids = collect_affected_entity_ids(
                &seed_entity_ids,
                &active_entity_ids,
                &edges,
                stale_related_entity_ids,
            );
            let tree_paths =
                build_tree_paths_for_scope(&active_entity_ids, &edges, &affected_entity_ids)?;

            // write scope は affected_entity_ids が diagram ごとに異なるため個別に反映する。
            self.tree_path_repository()
                .delete_tree_paths_by_entity_ids(DeleteTreePathsByEntityIdsSchema {
                    entity_ids: affected_entity_ids,
                })
                .await?;
            self.tree_path_repository()
                .create_tree_paths(CreateTreePathsSchema { tree_paths })
                .await?;
        }

        Ok(())
    }
}

pub trait ProvidesTreePathService: Send + Sync + 'static {
    type T: TreePathService;
    fn tree_path_service(&self) -> &Self::T;
}

fn group_seed_entity_ids(seed_entities: Vec<SeedEntity>) -> HashMap<usize, Vec<usize>> {
    let mut groups = HashMap::<usize, Vec<usize>>::new();

    for seed in seed_entities {
        groups
            .entry(seed.diagram_id)
            .or_default()
            .push(seed.entity_id);
    }

    for entity_ids in groups.values_mut() {
        let normalized = normalize_entity_ids(std::mem::take(entity_ids));
        *entity_ids = normalized;
    }

    groups
}

fn sorted_diagram_ids(grouped_entity_ids: &HashMap<usize, Vec<usize>>) -> Vec<usize> {
    let mut diagram_ids = grouped_entity_ids.keys().copied().collect::<Vec<_>>();
    diagram_ids.sort_unstable();
    diagram_ids
}

fn group_relationship_edges_by_diagram_id(
    diagram_relationship_edges: Vec<DiagramRelationshipEdge>,
) -> HashMap<usize, Vec<RelationshipEdge>> {
    let mut grouped = HashMap::<usize, Vec<RelationshipEdge>>::new();

    for edge in diagram_relationship_edges {
        grouped
            .entry(edge.diagram_id)
            .or_default()
            .push(RelationshipEdge {
                ancestor_id: edge.ancestor_id,
                descendant_id: edge.descendant_id,
            });
    }

    grouped
}

fn group_stale_related_entity_ids_by_diagram_id(
    seed_entity_ids_by_diagram_id: &HashMap<usize, Vec<usize>>,
    stale_connections: Vec<TreePathConnection>,
) -> HashMap<usize, Vec<usize>> {
    // tree_path には diagram_id がないため、seed に触れている row から diagram を逆引きする。
    let mut diagram_id_by_seed_entity_id = HashMap::<usize, usize>::new();

    for (diagram_id, seed_entity_ids) in seed_entity_ids_by_diagram_id {
        for seed_entity_id in seed_entity_ids {
            diagram_id_by_seed_entity_id.insert(*seed_entity_id, *diagram_id);
        }
    }

    let mut grouped = HashMap::<usize, Vec<usize>>::new();

    for connection in stale_connections {
        let Some(diagram_id) = diagram_id_by_seed_entity_id
            .get(&connection.ancestor_id)
            .or_else(|| diagram_id_by_seed_entity_id.get(&connection.descendant_id))
        else {
            continue;
        };

        grouped
            .entry(*diagram_id)
            .or_default()
            .extend([connection.ancestor_id, connection.descendant_id]);
    }

    for entity_ids in grouped.values_mut() {
        let normalized = normalize_entity_ids(std::mem::take(entity_ids));
        *entity_ids = normalized;
    }

    grouped
}

fn collect_affected_entity_ids(
    seed_entity_ids: &[usize],
    active_entity_ids: &[usize],
    edges: &[RelationshipEdge],
    stale_related_entity_ids: Vec<usize>,
) -> Vec<usize> {
    let mut affected_entity_ids = seed_entity_ids.to_vec();
    affected_entity_ids.extend(stale_related_entity_ids);
    affected_entity_ids.extend(expand_related_entity_ids_from_graph(
        seed_entity_ids,
        active_entity_ids,
        edges,
    ));

    normalize_entity_ids(affected_entity_ids)
}

fn expand_related_entity_ids_from_graph(
    seed_entity_ids: &[usize],
    active_entity_ids: &[usize],
    edges: &[RelationshipEdge],
) -> Vec<usize> {
    let active_entity_set = active_entity_ids.iter().copied().collect::<HashSet<_>>();
    let mut adjacency = HashMap::<usize, Vec<usize>>::new();
    let mut reverse_adjacency = HashMap::<usize, Vec<usize>>::new();

    for edge in edges {
        if active_entity_set.contains(&edge.ancestor_id)
            && active_entity_set.contains(&edge.descendant_id)
        {
            adjacency
                .entry(edge.ancestor_id)
                .or_default()
                .push(edge.descendant_id);
            reverse_adjacency
                .entry(edge.descendant_id)
                .or_default()
                .push(edge.ancestor_id);
        }
    }

    let mut related_entity_ids = HashSet::new();

    for seed_entity_id in seed_entity_ids {
        if !active_entity_set.contains(seed_entity_id) {
            continue;
        }

        related_entity_ids.extend(traverse_graph(*seed_entity_id, &adjacency));
        related_entity_ids.extend(traverse_graph(*seed_entity_id, &reverse_adjacency));
    }

    related_entity_ids.into_iter().collect()
}

fn traverse_graph(
    start_entity_id: usize,
    adjacency: &HashMap<usize, Vec<usize>>,
) -> HashSet<usize> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::from([start_entity_id]);

    while let Some(entity_id) = queue.pop_front() {
        if !visited.insert(entity_id) {
            continue;
        }

        if let Some(next_entity_ids) = adjacency.get(&entity_id) {
            for next_entity_id in next_entity_ids {
                if !visited.contains(next_entity_id) {
                    queue.push_back(*next_entity_id);
                }
            }
        }
    }

    visited
}

fn build_tree_paths_for_scope(
    active_entity_ids: &[usize],
    edges: &[RelationshipEdge],
    affected_entity_ids: &[usize],
) -> Result<Vec<TreePath>, SyncTreePathsServiceError> {
    let active_entity_set = active_entity_ids.iter().copied().collect::<HashSet<_>>();
    let affected_entity_set = affected_entity_ids
        .iter()
        .copied()
        .filter(|entity_id| active_entity_set.contains(entity_id))
        .collect::<HashSet<_>>();

    if affected_entity_set.is_empty() {
        return Ok(Vec::new());
    }

    let mut adjacency = HashMap::<usize, Vec<usize>>::new();

    for edge in edges {
        if active_entity_set.contains(&edge.ancestor_id)
            && active_entity_set.contains(&edge.descendant_id)
        {
            adjacency
                .entry(edge.ancestor_id)
                .or_default()
                .push(edge.descendant_id);
        }
    }

    for descendants in adjacency.values_mut() {
        descendants.sort_unstable();
        descendants.dedup();
    }

    detect_cycle(active_entity_ids, &adjacency)?;

    let mut affected_active_entity_ids = affected_entity_set.iter().copied().collect::<Vec<_>>();
    affected_active_entity_ids.sort_unstable();

    let mut tree_paths = Vec::new();

    for ancestor_id in affected_active_entity_ids {
        tree_paths.push(TreePath {
            ancestor_id,
            descendant_id: ancestor_id,
            depth: 0,
        });

        let mut visited = HashSet::from([ancestor_id]);
        let mut queue = VecDeque::new();

        if let Some(children) = adjacency.get(&ancestor_id) {
            for child_id in children {
                queue.push_back((*child_id, 1_usize));
            }
        }

        while let Some((descendant_id, depth)) = queue.pop_front() {
            if !visited.insert(descendant_id) {
                continue;
            }

            if affected_entity_set.contains(&descendant_id) {
                tree_paths.push(TreePath {
                    ancestor_id,
                    descendant_id,
                    depth,
                });
            }

            if let Some(children) = adjacency.get(&descendant_id) {
                for child_id in children {
                    if !visited.contains(child_id) {
                        queue.push_back((*child_id, depth + 1));
                    }
                }
            }
        }
    }

    Ok(tree_paths)
}

fn detect_cycle(
    entity_ids: &[usize],
    adjacency: &HashMap<usize, Vec<usize>>,
) -> Result<(), SyncTreePathsServiceError> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum VisitState {
        Visiting,
        Visited,
    }

    fn visit(
        node_id: usize,
        adjacency: &HashMap<usize, Vec<usize>>,
        states: &mut HashMap<usize, VisitState>,
    ) -> bool {
        if matches!(states.get(&node_id), Some(VisitState::Visiting)) {
            return true;
        }

        if matches!(states.get(&node_id), Some(VisitState::Visited)) {
            return false;
        }

        states.insert(node_id, VisitState::Visiting);

        if let Some(children) = adjacency.get(&node_id) {
            for child_id in children {
                if visit(*child_id, adjacency, states) {
                    return true;
                }
            }
        }

        states.insert(node_id, VisitState::Visited);
        false
    }

    let mut states = HashMap::new();

    for entity_id in entity_ids {
        if visit(*entity_id, adjacency, &mut states) {
            return Err(SyncTreePathsServiceError::CycleDetected);
        }
    }

    Ok(())
}

fn normalize_entity_ids(mut entity_ids: Vec<usize>) -> Vec<usize> {
    entity_ids.retain(|entity_id| *entity_id != 0);
    entity_ids.sort_unstable();
    entity_ids.dedup();
    entity_ids
}
