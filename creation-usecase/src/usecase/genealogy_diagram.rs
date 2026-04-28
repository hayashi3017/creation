use std::collections::HashMap;

use async_trait::async_trait;
use creation_service::{
    model::{
        diagram::{DiagramKind, GetDiagramSchema},
        entity::{EntityKind, GetEntitiesSchema},
        genealogy_diagram::{
            GenealogyDiagramEdge, GenealogyDiagramGraph, GenealogyDiagramNode,
            GenealogyDiagramStats, GetGenealogyDiagramSchema,
        },
        kinship_derivation::{CanonicalLineageEdge, DeriveKinshipInput},
        person::{GetPersonRecordsSchema, Person, PersonRecord},
        relationship::GetRelationshipsSchema,
    },
    repository::diagram::{
        GetDiagramRepositoryError, ProvidesDiagramRepository, UsesDiagramRepository,
    },
    service::{
        entity::{GetEntitiesServiceError, ProvidesEntityService, UsesEntityService},
        kinship_derivation::{ProvidesKinshipDerivationService, UsesKinshipDerivationService},
        person::{GetPersonRecordsServiceError, ProvidesPersonService, UsesPersonService},
        relationship::{
            GetRelationshipsServiceError, ProvidesRelationshipService, UsesRelationshipService,
        },
    },
};
use thiserror::Error;

#[async_trait]
pub trait GenealogyDiagramUsecase:
    ProvidesDiagramRepository
    + ProvidesEntityService
    + ProvidesPersonService
    + ProvidesRelationshipService
    + ProvidesKinshipDerivationService
{
}

#[derive(Debug, Error)]
pub enum GetGenealogyDiagramUsecaseError {
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
    #[error("diagram kind must be genealogy-compatible family_tree")]
    InvalidDiagramKind,
}

#[async_trait]
pub trait UsesGetGenealogyDiagramUsecase {
    async fn get_genealogy_diagram(
        &self,
        body: GetGenealogyDiagramSchema,
    ) -> Result<GenealogyDiagramGraph, GetGenealogyDiagramUsecaseError>;
}

#[async_trait]
impl<T: GenealogyDiagramUsecase> UsesGetGenealogyDiagramUsecase for T {
    async fn get_genealogy_diagram(
        &self,
        body: GetGenealogyDiagramSchema,
    ) -> Result<GenealogyDiagramGraph, GetGenealogyDiagramUsecaseError> {
        if body.diagram_id == 0 {
            return Err(GetGenealogyDiagramUsecaseError::InvalidParams);
        }

        let Some(diagram) = self
            .diagram_repository()
            .get_diagram(GetDiagramSchema {
                diagram_id: body.diagram_id,
            })
            .await?
        else {
            return Err(GetGenealogyDiagramUsecaseError::NotFound);
        };

        if diagram.kind != DiagramKind::FamilyTree {
            return Err(GetGenealogyDiagramUsecaseError::InvalidDiagramKind);
        }

        let persons = load_persons_for_diagram(self, body.diagram_id).await?;
        let active_person_ids = persons
            .iter()
            .map(|person| person.entity_id)
            .collect::<Vec<_>>();
        let relationships = self
            .relationship_service()
            .get_relationships(GetRelationshipsSchema {
                diagram_id: body.diagram_id,
            })
            .await
            .map_err(map_get_relationships_error)?;
        let kinship_derivation = self
            .kinship_derivation_service()
            .derive_kinship(DeriveKinshipInput {
                active_person_ids,
                explicit_relationships: relationships,
            })
            .await;
        let edges = build_genealogy_diagram_edges(kinship_derivation.lineage_edges);
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

            nodes.push(GenealogyDiagramNode {
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

        Ok(GenealogyDiagramGraph {
            diagram,
            stats: GenealogyDiagramStats {
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
pub trait UsesGenealogyDiagramUsecase: UsesGetGenealogyDiagramUsecase {
    async fn get_genealogy_diagram(
        &self,
        body: GetGenealogyDiagramSchema,
    ) -> Result<GenealogyDiagramGraph, GetGenealogyDiagramUsecaseError> {
        UsesGetGenealogyDiagramUsecase::get_genealogy_diagram(self, body).await
    }
}

impl<T> UsesGenealogyDiagramUsecase for T where T: UsesGetGenealogyDiagramUsecase {}

pub trait ProvidesGenealogyDiagramUsecase: Send + Sync + 'static {
    type T: UsesGenealogyDiagramUsecase + Sized;
    fn genealogy_diagram_usecase(&self) -> &Self::T;
}

async fn load_persons_for_diagram<T>(
    driver: &T,
    diagram_id: usize,
) -> Result<Vec<Person>, GetGenealogyDiagramUsecaseError>
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
        .map(|entity| entity.entity_id)
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
                .remove(&entity.entity_id)
                .map(|record| Person {
                    entity_id: entity.entity_id,
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

fn build_genealogy_diagram_edges(
    lineage_edges: Vec<CanonicalLineageEdge>,
) -> Vec<GenealogyDiagramEdge> {
    lineage_edges
        .into_iter()
        .map(|lineage_edge| GenealogyDiagramEdge {
            relationship_id: lineage_edge.relationship_id,
            parent_entity_id: lineage_edge.parent_entity_id,
            child_entity_id: lineage_edge.child_entity_id,
            kind: lineage_edge.kind,
            start_date: lineage_edge.start_date,
            end_date: lineage_edge.end_date,
            end_reason: lineage_edge.end_reason,
            notes: lineage_edge.notes,
        })
        .collect()
}

fn build_adjacency_maps(
    edges: &[GenealogyDiagramEdge],
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

fn map_get_entities_error(err: GetEntitiesServiceError) -> GetGenealogyDiagramUsecaseError {
    match err {
        GetEntitiesServiceError::InvalidParams => GetGenealogyDiagramUsecaseError::InvalidParams,
        err => GetGenealogyDiagramUsecaseError::GetEntitiesServiceError(err),
    }
}

fn map_get_person_records_error(
    err: GetPersonRecordsServiceError,
) -> GetGenealogyDiagramUsecaseError {
    match err {
        GetPersonRecordsServiceError::InvalidParams => {
            GetGenealogyDiagramUsecaseError::InvalidParams
        }
        err => GetGenealogyDiagramUsecaseError::GetPersonRecordsServiceError(err),
    }
}

fn map_get_relationships_error(
    err: GetRelationshipsServiceError,
) -> GetGenealogyDiagramUsecaseError {
    match err {
        GetRelationshipsServiceError::InvalidParams => {
            GetGenealogyDiagramUsecaseError::InvalidParams
        }
        err => GetGenealogyDiagramUsecaseError::GetRelationshipsServiceError(err),
    }
}
