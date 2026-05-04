use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use chrono::NaiveDate;
use creation_service::{
    model::{
        diagram::{DiagramKind, GetDiagramsSchema},
        entity::{Entity, EntityKind, LoadEntitiesByDiagramIdsSchema},
        genealogy_overview::{
            GenealogyOverview, GenealogyOverviewEdge, GenealogyOverviewEdgeSource,
            GenealogyOverviewNode, GenealogyOverviewStats, GenealogyOverviewWorld,
            GetGenealogyOverviewSchema,
        },
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
pub trait GenealogyOverviewUsecase:
    ProvidesWorldService
    + ProvidesDiagramRepository
    + ProvidesEntityService
    + ProvidesPersonService
    + ProvidesRelationshipService
{
}

#[derive(Debug, Error)]
pub enum GetGenealogyOverviewUsecaseError {
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
pub trait UsesGetGenealogyOverviewUsecase {
    async fn get_genealogy_overview(
        &self,
        body: GetGenealogyOverviewSchema,
    ) -> Result<GenealogyOverview, GetGenealogyOverviewUsecaseError>;
}

#[async_trait]
impl<T: GenealogyOverviewUsecase> UsesGetGenealogyOverviewUsecase for T {
    async fn get_genealogy_overview(
        &self,
        body: GetGenealogyOverviewSchema,
    ) -> Result<GenealogyOverview, GetGenealogyOverviewUsecaseError> {
        if body.world_id == 0
            || body.center_entity_id == Some(0)
            || body.ancestor_depth == Some(0)
            || body.descendant_depth == Some(0)
            || body
                .diagram_ids
                .as_ref()
                .is_some_and(|diagram_ids| diagram_ids.iter().any(|diagram_id| *diagram_id == 0))
        {
            return Err(GetGenealogyOverviewUsecaseError::InvalidParams);
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
                return Err(GetGenealogyOverviewUsecaseError::NotFound)
            }
            Err(err) => return Err(GetGenealogyOverviewUsecaseError::GetWorldServiceError(err)),
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
            return Err(GetGenealogyOverviewUsecaseError::NoVisibleGenealogyDiagrams);
        }

        let diagram_ids = diagrams
            .iter()
            .map(|diagram| diagram.diagram_id)
            .collect::<Vec<_>>();
        let diagram_id_set = diagram_ids.iter().copied().collect::<HashSet<_>>();

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
                diagram_ids: diagram_ids.clone(),
            })
            .await
            .map_err(map_load_relationships_by_diagram_ids_error)?;
        let mut edges = build_edges(
            relationships,
            &diagram_id_set,
            &active_entity_ids,
            &visible_entity_ids,
            body.as_of,
        );

        if let Some(center_entity_id) = body.center_entity_id {
            (nodes, edges) = filter_centered_overview(
                nodes,
                edges,
                center_entity_id,
                body.ancestor_depth,
                body.descendant_depth,
            )?;
        }

        let root_entity_ids = build_root_entity_ids(&nodes, &edges);

        Ok(GenealogyOverview {
            world: GenealogyOverviewWorld::from(world),
            diagram_ids,
            as_of: body.as_of,
            stats: GenealogyOverviewStats {
                diagram_count: diagrams.len(),
                node_count: nodes.len(),
                edge_count: edges.len(),
            },
            nodes,
            edges,
            root_entity_ids,
        })
    }
}

#[async_trait]
pub trait UsesGenealogyOverviewUsecase: UsesGetGenealogyOverviewUsecase {
    async fn get_genealogy_overview(
        &self,
        body: GetGenealogyOverviewSchema,
    ) -> Result<GenealogyOverview, GetGenealogyOverviewUsecaseError> {
        UsesGetGenealogyOverviewUsecase::get_genealogy_overview(self, body).await
    }
}

impl<T> UsesGenealogyOverviewUsecase for T where T: UsesGetGenealogyOverviewUsecase {}

pub trait ProvidesGenealogyOverviewUsecase: Send + Sync + 'static {
    type T: UsesGenealogyOverviewUsecase + Sized;
    fn genealogy_overview_usecase(&self) -> &Self::T;
}

fn build_nodes(
    entities: Vec<Entity>,
    person_records: Vec<PersonRecord>,
    as_of: Option<NaiveDate>,
) -> Vec<GenealogyOverviewNode> {
    let records_by_entity_id = person_records
        .into_iter()
        .map(|record| (record.entity_id, record))
        .collect::<HashMap<_, _>>();
    let mut nodes_by_entity_id = HashMap::<usize, GenealogyOverviewNode>::new();

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
            .or_insert_with(|| GenealogyOverviewNode {
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
    relationships: Vec<Relationship>,
    diagram_id_set: &HashSet<usize>,
    active_entity_ids: &HashSet<usize>,
    visible_entity_ids: &HashSet<usize>,
    as_of: Option<NaiveDate>,
) -> Vec<GenealogyOverviewEdge> {
    let mut edges_by_key = HashMap::<EdgeKey, GenealogyOverviewEdge>::new();

    for relationship in relationships {
        if !diagram_id_set.contains(&relationship.diagram_id)
            || relationship.source_entity_id == relationship.target_entity_id
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
                edge.source_diagram_ids.push(relationship.diagram_id);
            })
            .or_insert_with(|| GenealogyOverviewEdge {
                source_entity_id,
                target_entity_id,
                kind: relationship.kind,
                source: GenealogyOverviewEdgeSource::Explicit,
                start_date: relationship.start_date,
                end_date: relationship.end_date,
                end_reason: relationship.end_reason,
                source_relationship_ids: vec![relationship.relationship_id],
                source_diagram_ids: vec![relationship.diagram_id],
            });
    }

    let mut edges = edges_by_key.into_values().collect::<Vec<_>>();
    for edge in &mut edges {
        edge.source_relationship_ids.sort_unstable();
        edge.source_relationship_ids.dedup();
        edge.source_diagram_ids.sort_unstable();
        edge.source_diagram_ids.dedup();
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

fn filter_centered_overview(
    nodes: Vec<GenealogyOverviewNode>,
    edges: Vec<GenealogyOverviewEdge>,
    center_entity_id: usize,
    ancestor_depth: Option<usize>,
    descendant_depth: Option<usize>,
) -> Result<
    (Vec<GenealogyOverviewNode>, Vec<GenealogyOverviewEdge>),
    GetGenealogyOverviewUsecaseError,
> {
    if !nodes.iter().any(|node| node.entity_id == center_entity_id) {
        return Err(GetGenealogyOverviewUsecaseError::NotFound);
    }

    let mut visible_entity_ids = HashSet::from([center_entity_id]);
    collect_tree_neighborhood(
        center_entity_id,
        ancestor_depth.unwrap_or(usize::MAX),
        Direction::Ancestor,
        &edges,
        &mut visible_entity_ids,
    );
    collect_tree_neighborhood(
        center_entity_id,
        descendant_depth.unwrap_or(usize::MAX),
        Direction::Descendant,
        &edges,
        &mut visible_entity_ids,
    );

    let nodes = nodes
        .into_iter()
        .filter(|node| visible_entity_ids.contains(&node.entity_id))
        .collect::<Vec<_>>();
    let edges = edges
        .into_iter()
        .filter(|edge| {
            visible_entity_ids.contains(&edge.source_entity_id)
                && visible_entity_ids.contains(&edge.target_entity_id)
        })
        .collect::<Vec<_>>();

    Ok((nodes, edges))
}

#[derive(Clone, Copy)]
enum Direction {
    Ancestor,
    Descendant,
}

fn collect_tree_neighborhood(
    entity_id: usize,
    depth: usize,
    direction: Direction,
    edges: &[GenealogyOverviewEdge],
    visible_entity_ids: &mut HashSet<usize>,
) {
    if depth == 0 {
        return;
    }

    let related_entity_ids = edges
        .iter()
        .filter(|edge| edge.kind.is_tree_edge())
        .filter_map(|edge| match direction {
            Direction::Ancestor if edge.target_entity_id == entity_id => {
                Some(edge.source_entity_id)
            }
            Direction::Descendant if edge.source_entity_id == entity_id => {
                Some(edge.target_entity_id)
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    for related_entity_id in related_entity_ids {
        if visible_entity_ids.insert(related_entity_id) {
            collect_tree_neighborhood(
                related_entity_id,
                depth - 1,
                direction,
                edges,
                visible_entity_ids,
            );
        }
    }
}

fn build_root_entity_ids(
    nodes: &[GenealogyOverviewNode],
    edges: &[GenealogyOverviewEdge],
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

fn map_load_entities_by_diagram_ids_error(
    err: LoadEntitiesByDiagramIdsServiceError,
) -> GetGenealogyOverviewUsecaseError {
    match err {
        LoadEntitiesByDiagramIdsServiceError::InvalidParams => {
            GetGenealogyOverviewUsecaseError::InvalidParams
        }
        err => GetGenealogyOverviewUsecaseError::LoadEntitiesByDiagramIdsServiceError(err),
    }
}

fn map_get_person_records_error(
    err: GetPersonRecordsServiceError,
) -> GetGenealogyOverviewUsecaseError {
    match err {
        GetPersonRecordsServiceError::InvalidParams => {
            GetGenealogyOverviewUsecaseError::InvalidParams
        }
        err => GetGenealogyOverviewUsecaseError::GetPersonRecordsServiceError(err),
    }
}

fn map_load_relationships_by_diagram_ids_error(
    err: LoadRelationshipsByDiagramIdsServiceError,
) -> GetGenealogyOverviewUsecaseError {
    match err {
        LoadRelationshipsByDiagramIdsServiceError::InvalidParams => {
            GetGenealogyOverviewUsecaseError::InvalidParams
        }
        err => GetGenealogyOverviewUsecaseError::LoadRelationshipsByDiagramIdsServiceError(err),
    }
}
