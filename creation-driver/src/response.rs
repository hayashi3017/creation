use creation_service::model::{
    diagram::Diagram, genealogy_graph::GenealogyGraphPayload, person::Person,
    relationship::Relationship, user::FilteredUser, world::World,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, Debug, ToSchema)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct HealthCheckResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct StatusResponse {
    pub status: String,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct OperationResultData {
    pub response: String,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct RegisterUserResponse {
    pub status: String,
    pub data: OperationResultData,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct LoginUserResponse {
    pub status: String,
    pub token: String,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct UserData {
    pub user: FilteredUser,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct UserResponse {
    pub status: String,
    pub data: UserData,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct DiagramListResponse {
    pub status: String,
    pub data: Vec<Diagram>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct WorldResponse {
    pub status: String,
    pub data: World,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct WorldListResponse {
    pub status: String,
    pub data: Vec<World>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct PersonListResponse {
    pub status: String,
    pub data: Vec<Person>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct RelationshipListResponse {
    pub status: String,
    pub data: Vec<Relationship>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct GenealogyGraphResponse {
    pub status: String,
    pub data: GenealogyGraphPayload,
}
