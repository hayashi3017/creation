use creation_service::model::{
    diagram::Diagram, family_tree::FamilyTree, person::Person, relationship::Relationship,
    user::FilteredUser,
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
pub struct FamilyTreeResponse {
    pub status: String,
    pub data: FamilyTree,
}
