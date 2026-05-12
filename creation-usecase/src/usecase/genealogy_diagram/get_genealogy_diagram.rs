use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use chrono::NaiveDate;
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
        relationship::{GetRelationshipsSchema, Relationship, RelationshipTopology},
    },
    repository::diagram::{
        GetDiagramRepositoryError, ProvidesDiagramRepository, UsesDiagramRepository,
    },
    service::{
        entity::{GetEntitiesServiceError, ProvidesEntityService, UsesEntityService},
        kinship_derivation::UsesKinshipDerivationService,
        person::{GetPersonRecordsServiceError, ProvidesPersonService, UsesPersonService},
        relationship::{GetRelationshipsServiceError, UsesRelationshipService},
    },
};
use thiserror::Error;

use super::GenealogyDiagramUsecase;

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

        let mut persons =
            load_persons_for_diagram(self, diagram.world_id, body.diagram_id, body.as_of).await?;
        let active_person_ids = persons
            .iter()
            .map(|person| person.entity_id)
            .collect::<Vec<_>>();
        let active_person_id_set = active_person_ids.iter().copied().collect::<HashSet<_>>();
        let relationships = self
            .relationship_service()
            .get_relationships(GetRelationshipsSchema {
                diagram_id: body.diagram_id,
            })
            .await
            .map_err(map_get_relationships_error)?;
        let mut relationships =
            filter_relationships(relationships, &active_person_id_set, body.as_of);

        if let Some(center_entity_id) = body.center_entity_id {
            (persons, relationships) = filter_centered_diagram(
                persons,
                relationships,
                center_entity_id,
                body.ancestor_depth,
                body.descendant_depth,
            )?;
        }

        let active_person_ids = persons
            .iter()
            .map(|person| person.entity_id)
            .collect::<Vec<_>>();
        let kinship_derivation = self
            .kinship_derivation_service()
            .derive_kinship(DeriveKinshipInput {
                active_person_ids,
                explicit_relationships: relationships.clone(),
            })
            .await;
        let edges = build_genealogy_diagram_edges(relationships);
        let (parent_entity_ids_by_child, child_entity_ids_by_parent) =
            build_adjacency_maps(&kinship_derivation.lineage_edges);

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
            as_of: body.as_of,
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

async fn load_persons_for_diagram<T>(
    driver: &T,
    world_id: usize,
    diagram_id: usize,
    as_of: Option<NaiveDate>,
) -> Result<Vec<Person>, GetGenealogyDiagramUsecaseError>
where
    T: ProvidesEntityService + ProvidesPersonService,
{
    let entities = driver
        .entity_service()
        .get_entities(GetEntitiesSchema {
            world_id,
            diagram_id,
        })
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

    let persons = merge_persons(person_entities, person_records)
        .into_iter()
        .filter(|person| is_person_visible_as_of(person, as_of))
        .collect();

    Ok(persons)
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
                    first_name: record.first_name,
                    middle_name: record.middle_name,
                    last_name: record.last_name,
                    first_name_kana: record.first_name_kana,
                    middle_name_kana: record.middle_name_kana,
                    last_name_kana: record.last_name_kana,
                    first_name_romaji: record.first_name_romaji,
                    middle_name_romaji: record.middle_name_romaji,
                    last_name_romaji: record.last_name_romaji,
                    gender: record.gender,
                    birth_date: record.birth_date,
                    death_date: record.death_date,
                    birthplace: record.birthplace,
                    deathplace: record.deathplace,
                    residence: record.residence,
                    photo_url: record.photo_url,
                    profile_text: record.profile_text,
                })
        })
        .collect::<Vec<_>>();

    persons.sort_unstable_by_key(|person| person.entity_id);
    persons
}

fn is_person_visible_as_of(person: &Person, as_of: Option<NaiveDate>) -> bool {
    let Some(as_of) = as_of else {
        return true;
    };

    person
        .birth_date
        .is_none_or(|birth_date| birth_date <= as_of)
}

fn filter_relationships(
    relationships: Vec<creation_service::model::relationship::Relationship>,
    active_person_ids: &HashSet<usize>,
    as_of: Option<NaiveDate>,
) -> Vec<creation_service::model::relationship::Relationship> {
    relationships
        .into_iter()
        .filter(|relationship| {
            active_person_ids.contains(&relationship.source_entity_id)
                && active_person_ids.contains(&relationship.target_entity_id)
                && is_relationship_visible_as_of(relationship, as_of)
        })
        .collect()
}

fn is_relationship_visible_as_of(
    relationship: &creation_service::model::relationship::Relationship,
    as_of: Option<NaiveDate>,
) -> bool {
    let Some(as_of) = as_of else {
        return true;
    };

    relationship
        .start_date
        .is_none_or(|start_date| start_date <= as_of)
        && relationship
            .end_date
            .is_none_or(|end_date| as_of <= end_date)
}

fn filter_centered_diagram(
    persons: Vec<Person>,
    relationships: Vec<Relationship>,
    center_entity_id: usize,
    ancestor_depth: Option<usize>,
    descendant_depth: Option<usize>,
) -> Result<(Vec<Person>, Vec<Relationship>), GetGenealogyDiagramUsecaseError> {
    if !persons
        .iter()
        .any(|person| person.entity_id == center_entity_id)
    {
        return Err(GetGenealogyDiagramUsecaseError::NotFound);
    }

    let mut visible_entity_ids = HashSet::from([center_entity_id]);
    collect_tree_neighborhood(
        center_entity_id,
        ancestor_depth.unwrap_or(usize::MAX),
        Direction::Ancestor,
        &relationships,
        &mut visible_entity_ids,
    );
    collect_tree_neighborhood(
        center_entity_id,
        descendant_depth.unwrap_or(usize::MAX),
        Direction::Descendant,
        &relationships,
        &mut visible_entity_ids,
    );

    let persons = persons
        .into_iter()
        .filter(|person| visible_entity_ids.contains(&person.entity_id))
        .collect::<Vec<_>>();
    let relationships = relationships
        .into_iter()
        .filter(|relationship| {
            visible_entity_ids.contains(&relationship.source_entity_id)
                && visible_entity_ids.contains(&relationship.target_entity_id)
        })
        .collect::<Vec<_>>();

    Ok((persons, relationships))
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
    relationships: &[Relationship],
    visible_entity_ids: &mut HashSet<usize>,
) {
    if depth == 0 {
        return;
    }

    let related_entity_ids = relationships
        .iter()
        .filter(|relationship| relationship.kind.is_tree_edge())
        .filter_map(|relationship| match direction {
            Direction::Ancestor if relationship.target_entity_id == entity_id => {
                Some(relationship.source_entity_id)
            }
            Direction::Descendant if relationship.source_entity_id == entity_id => {
                Some(relationship.target_entity_id)
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
                relationships,
                visible_entity_ids,
            );
        }
    }
}

fn build_genealogy_diagram_edges(relationships: Vec<Relationship>) -> Vec<GenealogyDiagramEdge> {
    let mut edges = relationships
        .into_iter()
        .map(|relationship| {
            let (source_entity_id, target_entity_id) = normalize_edge_endpoints(&relationship);

            GenealogyDiagramEdge {
                relationship_id: relationship.relationship_id,
                source_entity_id,
                target_entity_id,
                kind: relationship.kind,
                start_date: relationship.start_date,
                end_date: relationship.end_date,
                end_reason: relationship.end_reason,
                notes: relationship.notes,
            }
        })
        .collect::<Vec<_>>();

    edges.sort_unstable_by_key(|edge| {
        (
            edge.source_entity_id,
            edge.target_entity_id,
            edge.kind,
            edge.relationship_id,
        )
    });
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

fn build_adjacency_maps(
    edges: &[CanonicalLineageEdge],
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
