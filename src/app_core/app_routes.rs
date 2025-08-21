use crate::app_core::auth_middleware::AuthMiddleware;
use crate::apps::cart::routes::{create_cart, delete_cart, get_card_by_tenant, get_cards};
use crate::apps::orchestrator::routes::{
    authorize_app, create_orchestrator, delete_orchestrator, get_orchestrator, list_orchestrators,
    sync_all_users_with_app,
};
use crate::apps::product::routes::{
    create_product, delete_product, get_product, list_products, update_product,
};
use crate::apps::user::keycloak::routes::login_keycloak;
use crate::apps::user::routes::{
    change_password, confirm_email, create_user, delete_user, forgot_password, get_me, list_users,
    login, update_profile, update_user,
};
use actix_web::{HttpResponse, Responder, Scope, web};

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "message": "Servidor Actix Web funcionando!",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

pub fn api_v1_scope() -> Scope {
    web::scope("/api/v1")
        .route("/health/", web::get().to(health_check))
        // Rotas públicas do user (sem autenticação)
        .service(
            web::scope("/auth")
                .route("/register/", web::post().to(create_user))
                .route("/login/", web::post().to(login))
                .route("/login-keycloak/", web::post().to(login_keycloak))
                .route("/confirm-email/{code}/", web::get().to(confirm_email))
                .route("/forgot-password/", web::post().to(forgot_password))
                .route("/change-password/", web::post().to(change_password)),
        )
        // Rota pública do orchestrator para autorização de apps
        .service(
            web::scope("/orchestrator")
                .route("/authorize/{app_token}/", web::get().to(authorize_app)),
        )
        .service(
            web::scope("") // escopo vazio herda o "/api/v1"
                .wrap(AuthMiddleware)
                // .service(get_logs)
                // Rotas privadas do user (com autenticação)
                .service(
                    web::scope("/users")
                        .route("/me/", web::get().to(get_me))
                        .route("/profile/", web::patch().to(update_profile))
                        .route("/", web::get().to(list_users))
                        .route("/", web::patch().to(update_user))
                        .route("/", web::delete().to(delete_user)),
                )
                // Rotas privadas do orchestrator (apenas super_admin)
                .service(
                    web::scope("/apps-orchestrator")
                        .route("/", web::get().to(list_orchestrators))
                        .route("/", web::post().to(create_orchestrator))
                        .route("/{id}/", web::get().to(get_orchestrator))
                        .route("/{id}/", web::delete().to(delete_orchestrator))
                        .route("/sync-users/", web::post().to(sync_all_users_with_app)),
                )
                .service(
                    web::scope("/products")
                        .route("/", web::get().to(list_products))
                        .route("/", web::post().to(create_product))
                        .route("/{id}/", web::get().to(get_product))
                        .route("/{id}/", web::put().to(update_product))
                        .route("/{id}/", web::delete().to(delete_product)),
                )
                .service(
                    web::scope("/carts")
                        .route("/", web::get().to(get_card_by_tenant))
                        .route("/all/", web::get().to(get_cards))
                        .route("/", web::post().to(create_cart))
                        .route("/{id}/", web::delete().to(delete_cart)),
                ),
        )
}
