mod auth;
mod core;
mod i18n;
mod mailer;
mod media;
mod messenger;
mod user;

use crate::auth::middlewares::auth_middleware;
use crate::core::app::AppConfig;
use crate::core::db::create_pool;
use crate::i18n::middlewares::locale_middleware;
use crate::i18n::I18nService;
use crate::messenger::ws::UserWsState;
use crate::messenger::WsState;
use axum::Router;
use axum::{http::HeaderValue, middleware};
use http::header;
use sqlx::{Pool, Postgres};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    services::ServeDir,
};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone, Debug)]
struct AppState {
    pool: Pool<Postgres>,
    i18n: I18nService,

    frontend_url: String,

    smtp_host: String,
    smtp_port: u16,
    smtp_username: String,
    smtp_password: String,
    smtp_from: String,

    ws_state: Option<WsState>,
    user_ws_state: Option<UserWsState>,
    media_base_url: String,
    is_registration_email: bool,
}

#[derive(OpenApi)]
#[openapi(
    security(("bearer_auth" = [])),
    paths(
        crate::auth::handlers::login,
        crate::auth::handlers::logout,
        crate::auth::handlers::refresh,
        crate::auth::handlers::register,
        crate::auth::handlers::check_register_key,
        crate::user::handlers::create,
        crate::user::handlers::get_all,
        crate::user::handlers::get_one,
        crate::user::handlers::delete_one,
        crate::user::handlers::update,
        crate::media::handlers::create,
        crate::media::handlers::get_all,
        crate::media::handlers::get_one,
        crate::media::handlers::update,
        crate::media::handlers::delete_one,
        crate::media::handlers::delete_all,
        crate::i18n::handlers::create_or_update,
        crate::i18n::handlers::get_all,
        crate::i18n::handlers::delete_one,
        crate::i18n::handlers::get_by_dict_key,
        crate::messenger::presentation::handlers::chat_handlers::get_chats,
        crate::messenger::presentation::handlers::chat_handlers::get_chat,
        crate::messenger::presentation::handlers::chat_handlers::create_chat,
        crate::messenger::presentation::handlers::chat_handlers::update_chat,
        crate::messenger::presentation::handlers::chat_handlers::delete_chat,
        crate::messenger::presentation::handlers::chat_handlers::set_alias,
        crate::messenger::presentation::handlers::chat_handlers::get_members,
        crate::messenger::presentation::handlers::chat_handlers::get_media_counts,
        crate::messenger::presentation::handlers::message_handlers::get_messages,
        crate::messenger::presentation::handlers::message_handlers::send_message,
        crate::messenger::presentation::handlers::message_handlers::edit_message,
        crate::messenger::presentation::handlers::message_handlers::delete_message,
        crate::messenger::presentation::handlers::message_handlers::mark_delivered,
        crate::messenger::presentation::handlers::message_handlers::mark_read,
        crate::messenger::presentation::handlers::message_handlers::search_in_chat,
        crate::messenger::presentation::handlers::message_handlers::global_search,
        crate::messenger::presentation::handlers::message_handlers::get_message_receipts,
        crate::messenger::presentation::handlers::ws_handlers::ws_upgrade,
        crate::messenger::presentation::handlers::ws_handlers::ws_user_upgrade,
    ),
    components(schemas(
        crate::messenger::dto::ChatResponseDTO,
        crate::messenger::dto::CreateChatDTO,
        crate::messenger::dto::UpdateChatDTO,
        crate::messenger::dto::ChatType,
        crate::messenger::dto::ChatMemberDTO,
        crate::messenger::dto::AddMemberDTO,
        crate::messenger::dto::MessageResponseDTO,
        crate::messenger::dto::CreateMessageDTO,
        crate::messenger::dto::UpdateMessageDTO,
        crate::messenger::dto::DeliveryStatus,
        crate::messenger::dto::WsServerEvent,
        crate::messenger::dto::WsClientAction,
        crate::messenger::dto::MessageQuery,
        crate::messenger::dto::SearchQuery,
        crate::messenger::dto::SetAliasDTO,
        crate::messenger::dto::MessageReceiptDTO,
        crate::messenger::dto::ChatMediaCountsDTO,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth",      description = "Auth"),
        (name = "Media",     description = "Media"),
        (name = "User",      description = "User"),
        (name = "I18n",      description = "Translations management"),
        (name = "Messenger", description = "Chats & real-time messaging"),
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("gp_apikey"))),
            );
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .build(),
                ),
            );
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    dotenv::dotenv().ok();

    let config = AppConfig::new();
    let pool = create_pool(&config).await;
    let app_host = config.app_host.clone();
    let app_port = config.app_port.clone();

    let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");
    let smtp_port: u16 = env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .expect("Invalid SMTP_PORT");
    let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
    let smtp_from = env::var("SMTP_FROM").expect("SMTP_FROM must be set");
    let is_registration_email = env::var("IS_REGISTRATION_EMAIL")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);

    let i18n = I18nService::new(pool.clone());

    let shared_state = Arc::new(AppState {
        pool: pool.clone(),
        i18n,

        frontend_url: env::var("FRONTEND_URL").expect("FRONTEND_URL must be set"),

        smtp_host,
        smtp_port,
        smtp_username,
        smtp_password,
        smtp_from,

        ws_state: Some(WsState::new()),
        user_ws_state: Some(UserWsState::new()),
        media_base_url: env::var("PUBLIC_URL")
            .unwrap_or_else(|_| "http://localhost:8000".to_string()),
        is_registration_email,
    });

    let cors = {
        let allowed_origins: Vec<String> = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect();

        CorsLayer::new()
            .allow_origin(AllowOrigin::predicate(move |origin: &HeaderValue, _| {
                origin
                    .to_str()
                    .map(|o| allowed_origins.iter().any(|a| a == &o.to_lowercase()))
                    .unwrap_or(false)
            }))
            .allow_methods(AllowMethods::list(vec![
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PUT,
                axum::http::Method::DELETE,
                axum::http::Method::OPTIONS,
            ]))
            .allow_headers(AllowHeaders::list(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT_LANGUAGE,
                // 🔥 WebSocket-специфичные заголовки — обязательны для браузера:
                header::HeaderName::from_static("sec-websocket-key"),
                header::HeaderName::from_static("sec-websocket-version"),
                header::HeaderName::from_static("sec-websocket-extensions"),
                header::HeaderName::from_static("connection"),
                header::HeaderName::from_static("upgrade"),
            ]))
            .expose_headers([header::HeaderName::from_static("sec-websocket-accept")])
            .allow_credentials(true)
            .max_age(Duration::from_secs(3600))
    };

    let openapi = ApiDoc::openapi();

    let public_router = Router::new()
        .nest("/api/v1/auth", auth::handlers::router())
        .nest("/api/v1/user", user::handlers::public_router())
        .nest("/api/v1/i18n", i18n::handlers::public_router())
        .nest("/api/v1/media", media::handlers::router())
        .nest_service(
            "/media",
            ServeDir::new("media").fallback(ServeDir::new("media/image")),
        )
        .with_state(shared_state.clone());

    let protected_router = Router::new()
        .nest("/api/v1/user", user::handlers::protected_router())
        .nest("/api/v1/i18n", i18n::handlers::protected_router())
        .nest("/api/v1/chats", messenger::presentation::messenger_router())
        .nest("/ws/chats", messenger::presentation::ws_router())
        .nest("/ws/user", messenger::presentation::ws_user_router())
        .with_state(shared_state.clone())
        .layer(middleware::from_fn_with_state(
            shared_state.clone(),
            auth_middleware,
        ));

    let app_router = public_router
        .merge(protected_router)
        .layer(middleware::from_fn(locale_middleware));

    let router = app_router
        .merge(SwaggerUi::new("/docs").url("/swagger/openapi.json", openapi))
        .layer(cors);

    let _ = sqlx::migrate!().run(&pool).await;

    let listener = tokio::net::TcpListener::bind(format!("{app_host}:{app_port}"))
        .await
        .unwrap();

    println!("Server is running at http://{app_host}:{app_port}/");
    println!("Swagger is running at http://{app_host}:{app_port}/docs");

    axum::serve(listener, router.into_make_service())
        .await
        .expect("Error serving");
}
