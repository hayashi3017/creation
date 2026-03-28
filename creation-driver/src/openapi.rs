use axum::{response::Html, Json};
use creation_service::model::{
    diagram::{CreateDiagramSchema, Diagram, DiagramKind},
    family_tree::{FamilyTree, FamilyTreeEdge, FamilyTreeNode, FamilyTreeStats},
    person::{CreatePersonSchema, GenderKind, GetPersonsSchema, Person},
    relationship::{
        CreateRelationshipSchema, GetRelationshipsSchema, Relationship, RelationshipKind,
    },
    user::{FilteredUser, LoginUserSchema, RegisterUserSchema},
};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::{
    handler::{
        diagram::UpdateDiagramRequest, person::UpdatePersonRequest,
        relationship::UpdateRelationshipRequest,
    },
    response::{
        DiagramListResponse, ErrorResponse, FamilyTreeResponse, HealthCheckResponse,
        LoginUserResponse, PersonListResponse, RegisterUserResponse, RelationshipListResponse,
        StatusResponse, UserResponse,
    },
};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);

        components.add_security_scheme(
            "cookie_auth",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::with_description(
                "token",
                "HttpOnly authentication cookie.",
            ))),
        );
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("Bearer token accepted by the same auth middleware."))
                    .build(),
            ),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handler::health_check::health_checker_handler,
        crate::handler::user::register_user_handler,
        crate::handler::user::login_user_handler,
        crate::handler::user::logout_handler,
        crate::handler::user::get_me_handler,
        crate::handler::family_tree::get_family_tree_by_diagram_id,
        crate::handler::diagram::get_diagrams,
        crate::handler::diagram::create_diagram,
        crate::handler::diagram::update_diagram_by_id,
        crate::handler::diagram::delete_diagram_by_id,
        crate::handler::person::get_persons_by_diagram,
        crate::handler::person::create_person,
        crate::handler::person::update_person_by_entity_id,
        crate::handler::person::delete_person_by_entity_id,
        crate::handler::relationship::get_relationships_by_diagram,
        crate::handler::relationship::create_relationship,
        crate::handler::relationship::update_relationship_by_id,
        crate::handler::relationship::delete_relationship_by_id
    ),
    components(schemas(
        RegisterUserSchema,
        LoginUserSchema,
        FilteredUser,
        CreateDiagramSchema,
        UpdateDiagramRequest,
        Diagram,
        DiagramKind,
        FamilyTree,
        FamilyTreeNode,
        FamilyTreeEdge,
        FamilyTreeStats,
        GetPersonsSchema,
        CreatePersonSchema,
        UpdatePersonRequest,
        Person,
        GenderKind,
        GetRelationshipsSchema,
        CreateRelationshipSchema,
        UpdateRelationshipRequest,
        Relationship,
        RelationshipKind,
        ErrorResponse,
        HealthCheckResponse,
        StatusResponse,
        RegisterUserResponse,
        LoginUserResponse,
        UserResponse,
        DiagramListResponse,
        PersonListResponse,
        RelationshipListResponse,
        FamilyTreeResponse
    )),
    modifiers(&SecurityAddon),
    info(
        title = "Creation API",
        version = env!("CARGO_PKG_VERSION"),
        description = include_str!("openapi_docs/en/api.md")
    ),
    tags(
        (name = "Health", description = include_str!("openapi_docs/en/tags/health.md")),
        (name = "Auth", description = include_str!("openapi_docs/en/tags/auth.md")),
        (name = "Users", description = include_str!("openapi_docs/en/tags/users.md")),
        (
            name = "FamilyTrees",
            description = include_str!("openapi_docs/en/tags/family_trees.md")
        ),
        (name = "Diagrams", description = include_str!("openapi_docs/en/tags/diagrams.md")),
        (name = "Persons", description = include_str!("openapi_docs/en/tags/persons.md")),
        (
            name = "Relationships",
            description = include_str!("openapi_docs/en/tags/relationships.md")
        )
    )
)]
pub struct ApiDoc;

pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

pub async fn swagger_ui_html() -> Html<&'static str> {
    Html(include_str!("openapi_docs/en/swagger_ui.html"))
}
