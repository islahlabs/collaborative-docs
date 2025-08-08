use utoipa::OpenApi;
use crate::models::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::signup,
        crate::handlers::login,
        crate::handlers::create_document,
        crate::handlers::get_document
    ),
    components(
        schemas(
            Document,
            DocumentHistory,
            CreateDocumentResponse,
            UpdateDocumentRequest,
            User,
            Role,
            SignupRequest,
            LoginRequest,
            AuthResponse,
            UpdateUserRoleRequest,
            crate::crdt::DocumentState,
            crate::crdt::DocumentUpdate
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "documents", description = "Document management endpoints")
    ),
    info(
        title = "Collaborative Docs API",
        description = "A real-time collaborative document editing API",
        version = "1.0.0",
        contact(
            name = "API Support",
            email = "support@islahlabs.com"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Development server"),
        (url = "https://docs.islahlabs.com", description = "Production server")
    )
)]
pub struct ApiDoc;

// Re-export for convenience
pub use utoipa_swagger_ui::SwaggerUi; 