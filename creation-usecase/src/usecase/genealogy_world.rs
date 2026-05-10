use std::collections::{HashMap, HashSet, VecDeque};

use async_trait::async_trait;
use chrono::NaiveDate;
use creation_service::{
    model::{
        diagram::{DiagramKind, GetDiagramsSchema},
        entity::{Entity, EntityKind, LoadEntitiesByDiagramIdsSchema},
        genealogy_graph::{
            GenealogyGraphContextPayload, GenealogyGraphEdgePayload, GenealogyGraphEdgeSource,
            GenealogyGraphNodePayload, GenealogyGraphPayload, GenealogyGraphSourceConfidence,
            GenealogyGraphStatsPayload, GenealogyRelationPathDirection,
            GenealogyRelationPathStepPayload, GenealogyRelationToCenter,
        },
        genealogy_world::GetGenealogyWorldSchema,
        person::{GetPersonRecordsSchema, PersonRecord},
        relationship::{
            LoadRelationshipsByDiagramIdsSchema, Relationship, RelationshipKind,
            RelationshipTopology,
        },
        world::GetWorldSchema,
    },
    repository::diagram::{
        GetDiagramsRepositoryError, ProvidesDiagramRepository, UsesDiagramRepository,
    },
    service::{
        entity::{LoadEntitiesByDiagramIdsServiceError, ProvidesEntityService, UsesEntityService},
        person::{GetPersonRecordsServiceError, ProvidesPersonService, UsesPersonService},
        relationship::{
            LoadRelationshipsByDiagramIdsServiceError, ProvidesRelationshipService,
            UsesRelationshipService,
        },
        world::{GetWorldServiceError, ProvidesWorldService, UsesWorldService},
    },
};
use thiserror::Error;

#[async_trait]
pub trait GenealogyWorldUsecase:
    ProvidesWorldService
    + ProvidesDiagramRepository
    + ProvidesEntityService
    + ProvidesPersonService
    + ProvidesRelationshipService
{
}

#[derive(Debug, Error)]
pub enum GetGenealogyWorldUsecaseError {
    #[error("invalid parameter")]
    InvalidParams,
    #[error(transparent)]
    GetWorldServiceError(#[from] GetWorldServiceError),
    #[error(transparent)]
    GetDiagramsRepositoryError(#[from] GetDiagramsRepositoryError),
    #[error(transparent)]
    LoadEntitiesByDiagramIdsServiceError(#[from] LoadEntitiesByDiagramIdsServiceError),
    #[error(transparent)]
    GetPersonRecordsServiceError(#[from] GetPersonRecordsServiceError),
    #[error(transparent)]
    LoadRelationshipsByDiagramIdsServiceError(#[from] LoadRelationshipsByDiagramIdsServiceError),
    #[error("not found")]
    NotFound,
    #[error("no visible genealogy diagrams")]
    NoVisibleGenealogyDiagrams,
}

#[async_trait]
pub trait UsesGetGenealogyWorldUsecase {
    async fn get_genealogy_world(
        &self,
        body: GetGenealogyWorldSchema,
    ) -> Result<GenealogyGraphPayload, GetGenealogyWorldUsecaseError>;
}

#[async_trait]
impl<T: GenealogyWorldUsecase> UsesGetGenealogyWorldUsecase for T {
    async fn get_genealogy_world(
        &self,
        body: GetGenealogyWorldSchema,
    ) -> Result<GenealogyGraphPayload, GetGenealogyWorldUsecaseError> {
        if body.world_id == 0
            || body.center_entity_id == Some(0)
            || body.ancestor_depth == Some(0)
            || body.descendant_depth == Some(0)
            || body
                .diagram_ids
                .as_ref()
                .is_some_and(|diagram_ids| diagram_ids.iter().any(|diagram_id| *diagram_id == 0))
        {
            return Err(GetGenealogyWorldUsecaseError::InvalidParams);
        }

        let world = match self
            .world_service()
            .get_world(GetWorldSchema {
                world_id: body.world_id,
            })
            .await
        {
            Ok(world) => world,
            Err(GetWorldServiceError::NotFound) => {
                return Err(GetGenealogyWorldUsecaseError::NotFound)
            }
            Err(err) => return Err(GetGenealogyWorldUsecaseError::GetWorldServiceError(err)),
        };

        let requested_diagram_ids = body
            .diagram_ids
            .as_ref()
            .map(|diagram_ids| diagram_ids.iter().copied().collect::<HashSet<_>>());

        let diagrams = self
            .diagram_repository()
            .get_diagrams(GetDiagramsSchema {
                world_id: body.world_id,
            })
            .await?
            .into_iter()
            .filter(|diagram| diagram.kind == DiagramKind::FamilyTree)
            .filter(|diagram| diagram.genealogy_overview_enabled)
            .filter(|diagram| {
                requested_diagram_ids
                    .as_ref()
                    .is_none_or(|ids| ids.contains(&diagram.diagram_id))
            })
            .collect::<Vec<_>>();

        if diagrams.is_empty() {
            return Err(GetGenealogyWorldUsecaseError::NoVisibleGenealogyDiagrams);
        }

        let diagram_ids = diagrams
            .iter()
            .map(|diagram| diagram.diagram_id)
            .collect::<Vec<_>>();

        let entity_rows = self
            .entity_service()
            .load_entities_by_diagram_ids(LoadEntitiesByDiagramIdsSchema {
                diagram_ids: diagram_ids.clone(),
            })
            .await
            .map_err(map_load_entities_by_diagram_ids_error)?;
        let active_entity_ids = entity_rows
            .iter()
            .map(|entity| entity.entity_id)
            .collect::<HashSet<_>>();
        let person_entity_ids = entity_rows
            .iter()
            .filter(|entity| matches!(entity.kind, EntityKind::Person))
            .map(|entity| entity.entity_id)
            .collect::<HashSet<_>>();
        let person_records = self
            .person_service()
            .get_person_records(GetPersonRecordsSchema {
                entity_ids: person_entity_ids.iter().copied().collect(),
            })
            .await
            .map_err(map_get_person_records_error)?;

        let mut nodes = build_nodes(entity_rows, person_records, body.as_of);
        let visible_entity_ids = nodes
            .iter()
            .map(|node| node.entity_id)
            .collect::<HashSet<_>>();
        let relationships = self
            .relationship_service()
            .load_relationships_by_diagram_ids(LoadRelationshipsByDiagramIdsSchema {
                world_id: body.world_id,
                diagram_ids: diagram_ids.clone(),
            })
            .await
            .map_err(map_load_relationships_by_diagram_ids_error)?;
        let edges = build_edges(
            body.world_id,
            relationships,
            &active_entity_ids,
            &visible_entity_ids,
            &source_diagram_ids_by_entity_id(&nodes),
            body.as_of,
        );

        let root_entity_ids = build_root_entity_ids(&nodes, &edges);
        apply_adjacency_and_roots(&mut nodes, &edges, &root_entity_ids);
        apply_center_metadata(body.center_entity_id, &mut nodes, &edges);

        Ok(GenealogyGraphPayload {
            context: GenealogyGraphContextPayload::World {
                world_id: world.world_id,
                diagram_ids: diagram_ids.clone(),
                name: world.name,
            },
            as_of: body.as_of,
            center_entity_id: body.center_entity_id,
            stats: GenealogyGraphStatsPayload {
                node_count: nodes.len(),
                edge_count: edges.len(),
                root_count: root_entity_ids.len(),
                diagram_count: diagrams.len(),
            },
            nodes,
            edges,
            root_entity_ids,
        })
    }
}

#[async_trait]
pub trait UsesGenealogyWorldUsecase: UsesGetGenealogyWorldUsecase {
    async fn get_genealogy_world(
        &self,
        body: GetGenealogyWorldSchema,
    ) -> Result<GenealogyGraphPayload, GetGenealogyWorldUsecaseError> {
        UsesGetGenealogyWorldUsecase::get_genealogy_world(self, body).await
    }
}

impl<T> UsesGenealogyWorldUsecase for T where T: UsesGetGenealogyWorldUsecase {}

pub trait ProvidesGenealogyWorldUsecase: Send + Sync + 'static {
    type T: UsesGenealogyWorldUsecase + Sized;
    fn genealogy_world_usecase(&self) -> &Self::T;
}

fn build_nodes(
    entities: Vec<Entity>,
    person_records: Vec<PersonRecord>,
    as_of: Option<NaiveDate>,
) -> Vec<GenealogyGraphNodePayload> {
    let records_by_entity_id = person_records
        .into_iter()
        .map(|record| (record.entity_id, record))
        .collect::<HashMap<_, _>>();
    let mut nodes_by_entity_id = HashMap::<usize, GenealogyGraphNodePayload>::new();

    for entity in entities {
        let Some(record) = records_by_entity_id.get(&entity.entity_id) else {
            continue;
        };

        if as_of.is_some_and(|as_of| {
            record
                .birth_date
                .is_some_and(|birth_date| birth_date > as_of)
        }) {
            continue;
        }

        nodes_by_entity_id
            .entry(entity.entity_id)
            .and_modify(|node| node.source_diagram_ids.push(entity.diagram_id))
            .or_insert_with(|| GenealogyGraphNodePayload {
                entity_id: entity.entity_id,
                name: entity.name,
                description: entity.description,
                gender: record.gender.clone(),
                birth_date: record.birth_date,
                death_date: record.death_date,
                birthplace: record.birthplace.clone(),
                residence: record.residence.clone(),
                photo_url: record.photo_url.clone(),
                source_diagram_ids: vec![entity.diagram_id],
                parent_entity_ids: Vec::new(),
                child_entity_ids: Vec::new(),
                is_root: false,
                relation_to_center: None,
                relation_path_to_center: None,
                generation_offset_from_center: None,
            });
    }

    let mut nodes = nodes_by_entity_id.into_values().collect::<Vec<_>>();
    for node in &mut nodes {
        node.source_diagram_ids.sort_unstable();
        node.source_diagram_ids.dedup();
    }
    nodes.sort_unstable_by_key(|node| node.entity_id);
    nodes
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EdgeKey {
    source_entity_id: usize,
    target_entity_id: usize,
    kind: RelationshipKind,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    end_reason: Option<String>,
}

fn build_edges(
    world_id: usize,
    relationships: Vec<Relationship>,
    active_entity_ids: &HashSet<usize>,
    visible_entity_ids: &HashSet<usize>,
    source_diagram_ids_by_entity_id: &HashMap<usize, Vec<usize>>,
    as_of: Option<NaiveDate>,
) -> Vec<GenealogyGraphEdgePayload> {
    let mut edges_by_key = HashMap::<EdgeKey, GenealogyGraphEdgePayload>::new();

    for relationship in relationships {
        if relationship.source_entity_id == relationship.target_entity_id
            || !active_entity_ids.contains(&relationship.source_entity_id)
            || !active_entity_ids.contains(&relationship.target_entity_id)
            || !visible_entity_ids.contains(&relationship.source_entity_id)
            || !visible_entity_ids.contains(&relationship.target_entity_id)
            || !is_relationship_visible_as_of(&relationship, as_of)
        {
            continue;
        }

        let (source_entity_id, target_entity_id) = normalize_edge_endpoints(&relationship);
        let key = EdgeKey {
            source_entity_id,
            target_entity_id,
            kind: relationship.kind,
            start_date: relationship.start_date,
            end_date: relationship.end_date,
            end_reason: relationship.end_reason.clone(),
        };

        edges_by_key
            .entry(key)
            .and_modify(|edge| {
                edge.source_relationship_ids
                    .push(relationship.relationship_id);
                edge.source_diagram_ids.extend(source_diagram_ids_for_edge(
                    source_diagram_ids_by_entity_id,
                    relationship.source_entity_id,
                    relationship.target_entity_id,
                ));
            })
            .or_insert_with(|| GenealogyGraphEdgePayload {
                edge_id: String::new(),
                source_entity_id,
                target_entity_id,
                kind: relationship.kind,
                source: GenealogyGraphEdgeSource::Explicit,
                source_confidence: GenealogyGraphSourceConfidence::Confirmed,
                start_date: relationship.start_date,
                end_date: relationship.end_date,
                end_reason: relationship.end_reason,
                notes: relationship.notes,
                source_relationship_ids: vec![relationship.relationship_id],
                source_diagram_ids: source_diagram_ids_for_edge(
                    source_diagram_ids_by_entity_id,
                    relationship.source_entity_id,
                    relationship.target_entity_id,
                ),
            });
    }

    let mut edges = edges_by_key.into_values().collect::<Vec<_>>();
    for edge in &mut edges {
        edge.source_relationship_ids.sort_unstable();
        edge.source_relationship_ids.dedup();
        edge.source_diagram_ids.sort_unstable();
        edge.source_diagram_ids.dedup();
        edge.edge_id = format!(
            "world:{}:edge:{}",
            world_id,
            edge.source_relationship_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join("-")
        );
    }
    edges.sort_unstable_by_key(|edge| (edge.source_entity_id, edge.target_entity_id, edge.kind));
    edges
}

fn normalize_edge_endpoints(relationship: &Relationship) -> (usize, usize) {
    if relationship.kind.topology() == RelationshipTopology::Symmetric {
        (
            relationship
                .source_entity_id
                .min(relationship.target_entity_id),
            relationship
                .source_entity_id
                .max(relationship.target_entity_id),
        )
    } else {
        (relationship.source_entity_id, relationship.target_entity_id)
    }
}

fn source_diagram_ids_by_entity_id(
    nodes: &[GenealogyGraphNodePayload],
) -> HashMap<usize, Vec<usize>> {
    nodes
        .iter()
        .map(|node| (node.entity_id, node.source_diagram_ids.clone()))
        .collect()
}

fn source_diagram_ids_for_edge(
    source_diagram_ids_by_entity_id: &HashMap<usize, Vec<usize>>,
    source_entity_id: usize,
    target_entity_id: usize,
) -> Vec<usize> {
    let mut diagram_ids = Vec::new();
    if let Some(source_diagram_ids) = source_diagram_ids_by_entity_id.get(&source_entity_id) {
        diagram_ids.extend(source_diagram_ids);
    }
    if let Some(target_diagram_ids) = source_diagram_ids_by_entity_id.get(&target_entity_id) {
        diagram_ids.extend(target_diagram_ids);
    }
    diagram_ids.sort_unstable();
    diagram_ids.dedup();
    diagram_ids
}

fn is_relationship_visible_as_of(relationship: &Relationship, as_of: Option<NaiveDate>) -> bool {
    let Some(as_of) = as_of else {
        return true;
    };

    relationship
        .start_date
        .is_none_or(|start_date| start_date <= as_of)
        && relationship
            .end_date
            .is_none_or(|end_date| end_date >= as_of)
}

fn build_root_entity_ids(
    nodes: &[GenealogyGraphNodePayload],
    edges: &[GenealogyGraphEdgePayload],
) -> Vec<usize> {
    let mut incoming_tree_edge_entity_ids = HashSet::new();
    for edge in edges {
        if edge.kind.is_tree_edge() {
            incoming_tree_edge_entity_ids.insert(edge.target_entity_id);
        }
    }

    nodes
        .iter()
        .filter(|node| !incoming_tree_edge_entity_ids.contains(&node.entity_id))
        .map(|node| node.entity_id)
        .collect()
}

fn apply_adjacency_and_roots(
    nodes: &mut [GenealogyGraphNodePayload],
    edges: &[GenealogyGraphEdgePayload],
    root_entity_ids: &[usize],
) {
    let mut parent_entity_ids_by_child = HashMap::<usize, Vec<usize>>::new();
    let mut child_entity_ids_by_parent = HashMap::<usize, Vec<usize>>::new();

    for edge in edges {
        if edge.kind.is_tree_edge() {
            parent_entity_ids_by_child
                .entry(edge.target_entity_id)
                .or_default()
                .push(edge.source_entity_id);
            child_entity_ids_by_parent
                .entry(edge.source_entity_id)
                .or_default()
                .push(edge.target_entity_id);
        }
    }

    let root_entity_ids = root_entity_ids.iter().copied().collect::<HashSet<_>>();
    for node in nodes {
        node.parent_entity_ids = parent_entity_ids_by_child
            .remove(&node.entity_id)
            .unwrap_or_default();
        node.child_entity_ids = child_entity_ids_by_parent
            .remove(&node.entity_id)
            .unwrap_or_default();
        node.parent_entity_ids.sort_unstable();
        node.parent_entity_ids.dedup();
        node.child_entity_ids.sort_unstable();
        node.child_entity_ids.dedup();
        node.is_root = root_entity_ids.contains(&node.entity_id);
    }
}

fn apply_center_metadata(
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

    match offset {
        Some(value) if value < 0 => GenealogyRelationToCenter::Ancestor,
        Some(value) if value > 0 => GenealogyRelationToCenter::Descendant,
        Some(0) => GenealogyRelationToCenter::Relative,
        _ => GenealogyRelationToCenter::Unknown,
    }
}

fn map_load_entities_by_diagram_ids_error(
    err: LoadEntitiesByDiagramIdsServiceError,
) -> GetGenealogyWorldUsecaseError {
    match err {
        LoadEntitiesByDiagramIdsServiceError::InvalidParams => {
            GetGenealogyWorldUsecaseError::InvalidParams
        }
        err => GetGenealogyWorldUsecaseError::LoadEntitiesByDiagramIdsServiceError(err),
    }
}

fn map_get_person_records_error(
    err: GetPersonRecordsServiceError,
) -> GetGenealogyWorldUsecaseError {
    match err {
        GetPersonRecordsServiceError::InvalidParams => GetGenealogyWorldUsecaseError::InvalidParams,
        err => GetGenealogyWorldUsecaseError::GetPersonRecordsServiceError(err),
    }
}

fn map_load_relationships_by_diagram_ids_error(
    err: LoadRelationshipsByDiagramIdsServiceError,
) -> GetGenealogyWorldUsecaseError {
    match err {
        LoadRelationshipsByDiagramIdsServiceError::InvalidParams => {
            GetGenealogyWorldUsecaseError::InvalidParams
        }
        err => GetGenealogyWorldUsecaseError::LoadRelationshipsByDiagramIdsServiceError(err),
    }
}
