use std::collections::{HashMap, HashSet, VecDeque};

use async_trait::async_trait;
use thiserror::Error;

use crate::{
    model::{
        entity::LoadActiveEntitiesByDiagramIdsSchema,
        relationship::{LoadRelationshipEdgesByWorldIdSchema, RelationshipEdge},
        tree_path::{
            CreateTreePathsSchema, DeleteTreePathsByEntityIdsSchema, DeleteTreePathsByWorldSchema,
            DeleteTreePathsForDiagramSchema, SyncTreePathsByEntityIdsSchema, TreePath,
        },
    },
    repository::{
        entity::{
            LoadActiveEntitiesByDiagramIdsRepositoryError, LoadEntitiesByWorldRepositoryError,
            LoadSeedEntitiesRepositoryError, ProvidesEntityRepository, UsesEntityRepository,
        },
        relationship::{
            LoadRelationshipEdgesByWorldIdRepositoryError, ProvidesRelationshipRepository,
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
    LoadEntitiesByWorldRepositoryError(#[from] LoadEntitiesByWorldRepositoryError),
    #[error(transparent)]
    LoadRelationshipEdgesByWorldIdRepositoryError(
        #[from] LoadRelationshipEdgesByWorldIdRepositoryError,
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

#[derive(Debug, Error)]
pub enum DeleteTreePathsForDiagramServiceError {
    #[error(transparent)]
    LoadActiveEntitiesByDiagramIdsRepositoryError(
        #[from] LoadActiveEntitiesByDiagramIdsRepositoryError,
    ),
    #[error(transparent)]
    DeleteTreePathsByEntityIdsRepositoryError(#[from] DeleteTreePathsByEntityIdsRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
}

#[async_trait]
pub trait UsesTreePathService {
    async fn sync_tree_paths_by_entity_ids(
        &self,
        body: SyncTreePathsByEntityIdsSchema,
    ) -> Result<(), SyncTreePathsServiceError>;
    async fn delete_tree_paths_for_diagram(
        &self,
        body: DeleteTreePathsForDiagramSchema,
    ) -> Result<(), DeleteTreePathsForDiagramServiceError>;
}

#[async_trait]
impl<T: TreePathService> UsesTreePathService for T {
    async fn sync_tree_paths_by_entity_ids(
        &self,
        body: SyncTreePathsByEntityIdsSchema,
    ) -> Result<(), SyncTreePathsServiceError> {
        let entity_ids = normalize_entity_ids(body.entity_ids);

        if body.world_id == 0 || entity_ids.is_empty() {
            return Ok(());
        }

        let active_entity_ids = self
            .entity_repository()
            .load_entities_by_world(crate::model::entity::LoadEntitiesByWorldSchema {
                world_id: body.world_id,
            })
            .await?
            .into_iter()
            .map(|entity| entity.entity_id)
            .collect::<Vec<_>>();

        if active_entity_ids.is_empty() {
            return Ok(());
        }

        let edges = self
            .relationship_repository()
            .load_relationship_edges_by_world_id(LoadRelationshipEdgesByWorldIdSchema {
                world_id: body.world_id,
            })
            .await?
            .into_iter()
            .map(|edge| RelationshipEdge {
                ancestor_id: edge.ancestor_id,
                descendant_id: edge.descendant_id,
            })
            .collect::<Vec<_>>();

        let tree_paths = build_tree_paths_for_scope(body.world_id, &active_entity_ids, &edges)?;

        self.tree_path_repository()
            .delete_tree_paths_by_world(DeleteTreePathsByWorldSchema {
                world_id: body.world_id,
            })
            .await?;
        self.tree_path_repository()
            .create_tree_paths(CreateTreePathsSchema { tree_paths })
            .await?;

        Ok(())
    }

    async fn delete_tree_paths_for_diagram(
        &self,
        body: DeleteTreePathsForDiagramSchema,
    ) -> Result<(), DeleteTreePathsForDiagramServiceError> {
        if body.diagram_id == 0 {
            return Err(DeleteTreePathsForDiagramServiceError::InvalidParams);
        }

        let entity_ids = self
            .entity_repository()
            .load_active_entities_by_diagram_ids(LoadActiveEntitiesByDiagramIdsSchema {
                diagram_ids: vec![body.diagram_id],
            })
            .await?
            .into_iter()
            .map(|entity| entity.entity_id)
            .collect();

        self.tree_path_repository()
            .delete_tree_paths_by_entity_ids(DeleteTreePathsByEntityIdsSchema { entity_ids })
            .await?;

        Ok(())
    }
}

pub trait ProvidesTreePathService: Send + Sync + 'static {
    type T: TreePathService;
    fn tree_path_service(&self) -> &Self::T;
}

fn build_tree_paths_for_scope(
    world_id: usize,
    active_entity_ids: &[usize],
    edges: &[RelationshipEdge],
) -> Result<Vec<TreePath>, SyncTreePathsServiceError> {
    let active_entity_set = active_entity_ids.iter().copied().collect::<HashSet<_>>();
    if active_entity_set.is_empty() {
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

    let mut affected_active_entity_ids = active_entity_set.iter().copied().collect::<Vec<_>>();
    affected_active_entity_ids.sort_unstable();

    let mut tree_paths = Vec::new();

    for ancestor_id in affected_active_entity_ids {
        tree_paths.push(TreePath {
            world_id,
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

            if active_entity_set.contains(&descendant_id) {
                tree_paths.push(TreePath {
                    world_id,
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
