use actix_web::{App, test, web};
use sqlx::PgPool;
use std::sync::Once;
use uuid::Uuid;

use rust_template::app_core::{
    app_routes::api_v1_scope, app_state::AppState, init_settings::init_settings,
};
use rust_template::apps::user::models::{
    LoginRequest, UpdateProfileRequest, UpdateUserRequest, UserRequest,
};

mod test_utils;
use test_utils::{clean_test_db, setup_test_db};

// ===== TEST SETUP =====

static INIT: Once = Once::new();

fn init() {
    INIT.call_once(|| {
        dotenvy::dotenv().ok();
        init_settings().expect("Falha ao inicializar settings");
    });
}

// ===== TEST HELPERS =====

fn create_test_app_state(pool: PgPool) -> AppState {
    init();
    AppState {
        db: pool,
        // Adicionar outros campos conforme necessário
    }
}

// ===== TEST DATA =====

fn create_test_user_request() -> UserRequest {
    UserRequest {
        email: format!("test_{}@example.com", Uuid::new_v4()),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        password: "password123".to_string(),
        profile: None,
        tenant_id: None,
    }
}

fn create_test_login_request(email: &str) -> LoginRequest {
    LoginRequest {
        email: email.to_string(),
        password: "password123".to_string(),
    }
}

// ===== TESTS =====

#[actix_web::test]
async fn test_health_check() {
    init();

    let pool = setup_test_db().await;
    let app_state = create_test_app_state(pool.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(api_v1_scope()),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/v1/health/").to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    clean_test_db(&pool).await;
}

#[actix_web::test]
async fn test_create_user() {
    init();

    let pool = setup_test_db().await;
    let app_state = create_test_app_state(pool.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(api_v1_scope()),
    )
    .await;

    let user_request = create_test_user_request();
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register/")
        .set_json(&user_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verificar se o usuário foi criado no banco
    let user = sqlx::query!(
        "SELECT email FROM users WHERE email = $1",
        user_request.email
    )
    .fetch_one(&pool)
    .await
    .expect("Usuário deveria existir no banco");

    assert_eq!(user.email, user_request.email);

    clean_test_db(&pool).await;
}

#[actix_web::test]
async fn test_login_user() {
    init();

    let pool = setup_test_db().await;
    let app_state = create_test_app_state(pool.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(api_v1_scope()),
    )
    .await;

    // Primeiro criar um usuário
    let user_request = create_test_user_request();
    println!("Criando usuário com email: {}", user_request.email);

    let create_req = test::TestRequest::post()
        .uri("/api/v1/auth/register/")
        .set_json(&user_request)
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    println!("Status da criação: {}", create_resp.status());

    if !create_resp.status().is_success() {
        let body = test::read_body(create_resp).await;
        println!("Erro na criação: {}", String::from_utf8_lossy(&body));
    }

    // Verificar se o usuário foi criado no banco
    let user_check = sqlx::query!(
        "SELECT email FROM users WHERE email = $1",
        user_request.email
    )
    .fetch_optional(&pool)
    .await
    .expect("Erro ao verificar usuário no banco");

    if let Some(user) = user_check {
        println!("Usuário encontrado no banco: {}", user.email);
    } else {
        println!("Usuário NÃO encontrado no banco!");
    }

    // Verificar se o perfil foi criado no banco
    let profile_check = sqlx::query!(
        "SELECT user_id FROM profiles WHERE user_id = (SELECT id FROM users WHERE email = $1)",
        user_request.email
    )
    .fetch_optional(&pool)
    .await
    .expect("Erro ao verificar perfil no banco");

    if let Some(profile) = profile_check {
        println!(
            "Perfil encontrado no banco para user_id: {}",
            profile.user_id
        );
    } else {
        println!("Perfil NÃO encontrado no banco!");
    }

    // Agora fazer login
    let login_request = create_test_login_request(&user_request.email);
    println!("Fazendo login com email: {}", login_request.email);

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login/")
        .set_json(&login_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    println!("Status do login: {}", status);

    if !status.is_success() {
        let body = test::read_body(resp).await;
        println!("Erro no login: {}", String::from_utf8_lossy(&body));
        assert!(status.is_success());
    } else {
        assert!(status.is_success());
    }

    clean_test_db(&pool).await;
}

#[actix_web::test]
async fn test_get_me_with_token() {
    init();

    let pool = setup_test_db().await;
    let app_state = create_test_app_state(pool.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(api_v1_scope()),
    )
    .await;

    // Criar usuário e fazer login para obter token
    let user_request = create_test_user_request();
    let create_req = test::TestRequest::post()
        .uri("/api/v1/auth/register/")
        .set_json(&user_request)
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let token = create_body["token"]
        .as_str()
        .expect("Token deveria existir");

    // Usar token para acessar /me
    let req = test::TestRequest::get()
        .uri("/api/v1/users/me/")
        .insert_header(("Authorization", format!("Token {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    clean_test_db(&pool).await;
}

#[actix_web::test]
async fn test_update_user() {
    init();

    let pool = setup_test_db().await;
    let app_state = create_test_app_state(pool.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(api_v1_scope()),
    )
    .await;

    // Criar usuário e obter token
    let user_request = create_test_user_request();
    let create_req = test::TestRequest::post()
        .uri("/api/v1/auth/register/")
        .set_json(&user_request)
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let token = create_body["token"]
        .as_str()
        .expect("Token deveria existir");

    // Atualizar dados do usuário
    let update_request = UpdateUserRequest {
        first_name: Some("Updated".to_string()),
        last_name: Some("Name".to_string()),
    };

    let req = test::TestRequest::patch()
        .uri("/api/v1/users/")
        .insert_header(("Authorization", format!("Token {}", token)))
        .set_json(&update_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    clean_test_db(&pool).await;
}

#[actix_web::test]
async fn test_update_profile() {
    init();

    let pool = setup_test_db().await;
    let app_state = create_test_app_state(pool.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(api_v1_scope()),
    )
    .await;

    // Criar usuário e obter token
    let user_request = create_test_user_request();
    let create_req = test::TestRequest::post()
        .uri("/api/v1/auth/register/")
        .set_json(&user_request)
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let token = create_body["token"]
        .as_str()
        .expect("Token deveria existir");

    // Atualizar perfil
    let update_request = UpdateProfileRequest {
        bio: Some("Nova biografia de teste".to_string()),
        phone: Some("11999999999".to_string()),
        birth_date: None,
        profession: Some("Desenvolvedor".to_string()),
        document: None,
        avatar: None,
    };

    let req = test::TestRequest::patch()
        .uri("/api/v1/users/profile/")
        .insert_header(("Authorization", format!("Token {}", token)))
        .set_json(&update_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    clean_test_db(&pool).await;
}

#[actix_web::test]
async fn test_delete_user() {
    init();

    let pool = setup_test_db().await;
    let app_state = create_test_app_state(pool.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(api_v1_scope()),
    )
    .await;

    // Criar usuário e obter token
    let user_request = create_test_user_request();
    let create_req = test::TestRequest::post()
        .uri("/api/v1/auth/register/")
        .set_json(&user_request)
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let token = create_body["token"]
        .as_str()
        .expect("Token deveria existir");

    // Deletar usuário
    let req = test::TestRequest::delete()
        .uri("/api/v1/users/")
        .insert_header(("Authorization", format!("Token {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    clean_test_db(&pool).await;
}

#[actix_web::test]
async fn test_unauthorized_access() {
    init();

    let pool = setup_test_db().await;
    let app_state = create_test_app_state(pool.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(api_v1_scope()),
    )
    .await;

    // Tentar acessar rota protegida sem token
    let req = test::TestRequest::get()
        .uri("/api/v1/users/me/")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error()); // 401 Unauthorized

    clean_test_db(&pool).await;
}

#[actix_web::test]
async fn test_invalid_token() {
    init();

    let pool = setup_test_db().await;
    let app_state = create_test_app_state(pool.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(api_v1_scope()),
    )
    .await;

    // Tentar acessar rota protegida com token inválido
    let req = test::TestRequest::get()
        .uri("/api/v1/users/me/")
        .insert_header(("Authorization", "Token invalid_token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error()); // 401 Unauthorized

    clean_test_db(&pool).await;
}
