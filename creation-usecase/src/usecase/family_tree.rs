use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use creation_service::{
    model::{
        diagram::{DiagramKind, GetDiagramSchema},
        entity::{EntityKind, GetEntitiesSchema},
        family_tree::{
            FamilyTree, FamilyTreeEdge, FamilyTreeNode, FamilyTreeStats, GetFamilyTreeSchema,
        },
        person::{GetPersonRecordsSchema, Person, PersonRecord},
        relationship::{GetRelationshipsSchema, Relationship, RelationshipKind},
    },
    repository::diagram::{
        GetDiagramRepositoryError, ProvidesDiagramRepository, UsesDiagramRepository,
    },
    service::{
        entity::{GetEntitiesServiceError, ProvidesEntityService, UsesEntityService},
        person::{GetPersonRecordsServiceError, ProvidesPersonService, UsesPersonService},
        relationship::{
            GetRelationshipsServiceError, ProvidesRelationshipService, UsesRelationshipService,
        },
    },
};
use thiserror::Error;

#[async_trait]
pub trait FamilyTreeUsecase:
    ProvidesDiagramRepository
    + ProvidesEntityService
    + ProvidesPersonService
    + ProvidesRelationshipService
{
}

#[derive(Debug, Error)]
pub enum GetFamilyTreeUsecaseError {
    #[error("invalid parameter")]
    InvalidParams,
    #[error(transparent)]
    GetDiagramRepositoryError(#[from] GetDiagramRepositoryError),
    #[error(transparent)]
    GetEntitiesServiceError(#[from] GetEntitiesServiceError),
    #[error(transparent)]
    GetPersonRecordsServiceError(#[from] GetPersonRecordsServiceError),
    #[error(transparent)]
    GetRelationshipsServiceError(#[from] GetRelationshipsServiceError),
    #[error("not found")]
    NotFound,
    #[error("diagram kind must be family_tree")]
    InvalidDiagramKind,
}

#[async_trait]
pub trait UsesGetFamilyTreeUsecase {
    async fn get_family_tree(
        &self,
        body: GetFamilyTreeSchema,
    ) -> Result<FamilyTree, GetFamilyTreeUsecaseError>;
}

#[async_trait]
impl<T: FamilyTreeUsecase> UsesGetFamilyTreeUsecase for T {
    async fn get_family_tree(
        &self,
        body: GetFamilyTreeSchema,
    ) -> Result<FamilyTree, GetFamilyTreeUsecaseError> {
        if body.diagram_id == 0 {
            return Err(GetFamilyTreeUsecaseError::InvalidParams);
        }

        let Some(diagram) = self
            .diagram_repository()
            .get_diagram(GetDiagramSchema {
                id: body.diagram_id,
            })
            .await?
        else {
            return Err(GetFamilyTreeUsecaseError::NotFound);
        };

        if diagram.kind != DiagramKind::FamilyTree {
            return Err(GetFamilyTreeUsecaseError::InvalidDiagramKind);
        }

        let persons = load_persons_for_diagram(self, body.diagram_id).await?;
        let active_person_ids = persons
            .iter()
            .map(|person| person.entity_id)
            .collect::<HashSet<_>>();
        let relationships = self
            .relationship_service()
            .get_relationships(GetRelationshipsSchema {
                diagram_id: body.diagram_id,
            })
            .await
            .map_err(map_get_relationships_error)?;
        let edges = build_family_tree_edges(relationships, &active_person_ids);
        let (parent_entity_ids_by_child, child_entity_ids_by_parent) = build_adjacency_maps(&edges);

        let mut root_entity_ids = Vec::new();
        let mut nodes = Vec::with_capacity(persons.len());

        for person in persons {
            let parent_entity_ids = parent_entity_ids_by_child
                .get(&person.entity_id)
                .cloned()
                .unwrap_or_default();
            let child_entity_ids = child_entity_ids_by_parent
                .get(&person.entity_id)
                .cloned()
                .unwrap_or_default();
            let is_root = parent_entity_ids.is_empty();

            if is_root {
                root_entity_ids.push(person.entity_id);
            }

            nodes.push(FamilyTreeNode {
                entity_id: person.entity_id,
                diagram_id: person.diagram_id,
                name: person.name,
                description: person.description,
                gender: person.gender,
                birth_date: person.birth_date,
                death_date: person.death_date,
                birthplace: person.birthplace,
                residence: person.residence,
                photo_url: person.photo_url,
                parent_entity_ids,
                child_entity_ids,
                is_root,
            });
        }

        root_entity_ids.sort_unstable();

        Ok(FamilyTree {
            diagram,
            stats: FamilyTreeStats {
                person_count: nodes.len(),
                edge_count: edges.len(),
                root_count: root_entity_ids.len(),
            },
            root_entity_ids,
            nodes,
            edges,
        })
    }
}

#[async_trait]
pub trait UsesFamilyTreeUsecase: UsesGetFamilyTreeUsecase {
    async fn get_family_tree(
        &self,
        body: GetFamilyTreeSchema,
    ) -> Result<FamilyTree, GetFamilyTreeUsecaseError> {
        UsesGetFamilyTreeUsecase::get_family_tree(self, body).await
    }
}

impl<T> UsesFamilyTreeUsecase for T where T: UsesGetFamilyTreeUsecase {}

pub trait ProvidesFamilyTreeUsecase: Send + Sync + 'static {
    type T: UsesFamilyTreeUsecase + Sized;
    fn family_tree_usecase(&self) -> &Self::T;
}

async fn load_persons_for_diagram<T>(
    driver: &T,
    diagram_id: usize,
) -> Result<Vec<Person>, GetFamilyTreeUsecaseError>
where
    T: ProvidesEntityService + ProvidesPersonService,
{
    let entities = driver
        .entity_service()
        .get_entities(GetEntitiesSchema { diagram_id })
        .await
        .map_err(map_get_entities_error)?;

    let person_entities = entities
        .into_iter()
        .filter(|entity| matches!(entity.kind, EntityKind::Person))
        .collect::<Vec<_>>();

    if person_entities.is_empty() {
        return Ok(Vec::new());
    }

    let entity_ids = person_entities
        .iter()
        .map(|entity| entity.id)
        .collect::<Vec<_>>();
    let person_records = driver
        .person_service()
        .get_person_records(GetPersonRecordsSchema { entity_ids })
        .await
        .map_err(map_get_person_records_error)?;

    Ok(merge_persons(person_entities, person_records))
}

fn merge_persons(
    person_entities: Vec<creation_service::model::entity::Entity>,
    person_records: Vec<PersonRecord>,
) -> Vec<Person> {
    let mut records_by_entity_id: HashMap<usize, PersonRecord> = person_records
        .into_iter()
        .map(|record| (record.entity_id, record))
        .collect();

    let mut persons = person_entities
        .into_iter()
        .filter_map(|entity| {
            records_by_entity_id
                .remove(&entity.id)
                .map(|record| Person {
                    entity_id: entity.id,
                    diagram_id: entity.diagram_id,
                    name: entity.name,
                    description: entity.description,
                    gender: record.gender,
                    birth_date: record.birth_date,
                    death_date: record.death_date,
                    birthplace: record.birthplace,
                    residence: record.residence,
                    photo_url: record.photo_url,
                })
        })
        .collect::<Vec<_>>();

    persons.sort_unstable_by_key(|person| person.entity_id);
    persons
}

fn build_family_tree_edges(
    relationships: Vec<Relationship>,
    active_person_ids: &HashSet<usize>,
) -> Vec<FamilyTreeEdge> {
    relationships
        .into_iter()
        .filter_map(|relationship| {
            let (parent_entity_id, child_entity_id) = match relationship.kind {
                RelationshipKind::Parent => {
                    (relationship.source_entity_id, relationship.target_entity_id)
                }
                RelationshipKind::Child => {
                    (relationship.target_entity_id, relationship.source_entity_id)
                }
                _ => return None,
            };

            if !active_person_ids.contains(&parent_entity_id)
                || !active_person_ids.contains(&child_entity_id)
            {
                return None;
            }

            Some(FamilyTreeEdge {
                relationship_id: relationship.id,
                parent_entity_id,
                child_entity_id,
                kind: relationship.kind,
                start_date: relationship.start_date,
                end_date: relationship.end_date,
                notes: relationship.notes,
            })
        })
        .collect()
}

fn build_adjacency_maps(
    edges: &[FamilyTreeEdge],
) -> (HashMap<usize, Vec<usize>>, HashMap<usize, Vec<usize>>) {
    let mut parent_entity_ids_by_child = HashMap::<usize, Vec<usize>>::new();
    let mut child_entity_ids_by_parent = HashMap::<usize, Vec<usize>>::new();

    for edge in edges {
        parent_entity_ids_by_child
            .entry(edge.child_entity_id)
            .or_default()
            .push(edge.parent_entity_id);
        child_entity_ids_by_parent
            .entry(edge.parent_entity_id)
            .or_default()
            .push(edge.child_entity_id);
    }

    normalize_adjacency_lists(&mut parent_entity_ids_by_child);
    normalize_adjacency_lists(&mut child_entity_ids_by_parent);

    (parent_entity_ids_by_child, child_entity_ids_by_parent)
}

fn normalize_adjacency_lists(adjacency: &mut HashMap<usize, Vec<usize>>) {
    for entity_ids in adjacency.values_mut() {
        entity_ids.sort_unstable();
        entity_ids.dedup();
    }
}

fn map_get_entities_error(err: GetEntitiesServiceError) -> GetFamilyTreeUsecaseError {
    match err {
        GetEntitiesServiceError::InvalidParams => GetFamilyTreeUsecaseError::InvalidParams,
        err => GetFamilyTreeUsecaseError::GetEntitiesServiceError(err),
    }
}

fn map_get_person_records_error(err: GetPersonRecordsServiceError) -> GetFamilyTreeUsecaseError {
    match err {
        GetPersonRecordsServiceError::InvalidParams => GetFamilyTreeUsecaseError::InvalidParams,
        err => GetFamilyTreeUsecaseError::GetPersonRecordsServiceError(err),
    }
}

fn map_get_relationships_error(err: GetRelationshipsServiceError) -> GetFamilyTreeUsecaseError {
    match err {
        GetRelationshipsServiceError::InvalidParams => GetFamilyTreeUsecaseError::InvalidParams,
        err => GetFamilyTreeUsecaseError::GetRelationshipsServiceError(err),
    }
}
