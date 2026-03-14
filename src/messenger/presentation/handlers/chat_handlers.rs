use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use uuid::Uuid;
use crate::AppState;
use crate::core::response::into_api_response;
use crate::messenger::dto::{CreateChatDTO, ChatResponseDTO, UpdateChatDTO, SetAliasDTO, ChatMemberDTO, ChatMediaCountsDTO};
use crate::core::dto::{ApiResponseWithPagination, PaginationDTO, ApiPaginationDTO, ApiResponse};
use crate::messenger::domain::repositories::ChatRepository;
use crate::messenger::infrastructure::postgres_repository::PostgresMessengerRepository;
use crate::messenger::application::chat_service::ChatService;

#[utoipa::path(
    get,
    path = "/api/v1/messenger/chats",
    params(
        ("page" = Option<i64>, Query, description = "Page number"),
        ("limit" = Option<i64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, body = ApiResponseWithPagination<ChatResponseDTO>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn get_chats(
    State(state): State<Arc<AppState>>,
    Extension(user_uuid): Extension<Uuid>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let page = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(20);

    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = ChatService::new(repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.get_chats(user_uuid, limit, page).await {
        Ok((chats, total)) => {
            let total_pages = (total as f64 / limit as f64).ceil() as i32;
            let pagination = PaginationDTO {
                page: page as i32,
                total: Some(total as i32),
                total_pages: Some(total_pages),
                limit: Some(limit as i32),
            };
            let data = ApiPaginationDTO { items: chats, pagination };
            into_api_response(StatusCode::OK, Some(data), None, None)
        }
        Err(e) => {
            eprintln!("Error fetching chats: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to fetch chats".into()]))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/messenger/chats/{chat_uuid}",
    params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
    responses(
        (status = 200, body = ApiResponse<ChatResponseDTO>),
        (status = 404, body = ApiResponse<ChatResponseDTO>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn get_chat(
    State(state): State<Arc<AppState>>,
    Path(chat_uuid): Path<Uuid>,
) -> impl IntoResponse {
    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = ChatService::new(repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.get_chat(chat_uuid).await {
        Ok(Some(chat)) => into_api_response(StatusCode::OK, Some(chat), None, None),
        Ok(None) => into_api_response(StatusCode::NOT_FOUND, None, None, Some(vec!["Chat not found".into()])),
        Err(e) => {
            eprintln!("Error fetching chat: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to fetch chat".into()]))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/messenger/chats",
    request_body = CreateChatDTO,
    responses(
        (status = 201, body = ApiResponse<Uuid>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn create_chat(
    State(state): State<Arc<AppState>>,
    Extension(user_uuid): Extension<Uuid>,
    Json(dto): Json<CreateChatDTO>,
) -> impl IntoResponse {
    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = ChatService::new(repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.create_chat(user_uuid, dto).await {
        Ok(chat_uuid) => into_api_response(StatusCode::CREATED, Some(chat_uuid), None, None),
        Err(e) => {
            eprintln!("Error creating chat: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to create chat".into()]))
        }
    }
}
#[utoipa::path(
    put,
    path = "/api/v1/messenger/chats/{chat_uuid}",
    params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
    request_body = UpdateChatDTO,
    responses(
        (status = 200, body = ApiResponse<bool>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn update_chat(
    State(state): State<Arc<AppState>>,
    Path(chat_uuid): Path<Uuid>,
    Json(dto): Json<UpdateChatDTO>,
) -> impl IntoResponse {
    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = ChatService::new(repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.update_chat(chat_uuid, dto).await {
        Ok(_) => into_api_response(StatusCode::OK, Some(true), None, None),
        Err(e) => {
            eprintln!("Error updating chat: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to update chat".into()]))
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/messenger/chats/{chat_uuid}",
    params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
    responses(
        (status = 200, body = ApiResponse<bool>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn delete_chat(
    State(state): State<Arc<AppState>>,
    Path(chat_uuid): Path<Uuid>,
) -> impl IntoResponse {
    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = ChatService::new(repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.delete_chat(chat_uuid).await {
        Ok(_) => into_api_response(StatusCode::OK, Some(true), None, None),
        Err(e) => {
            eprintln!("Error deleting chat: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to delete chat".into()]))
        }
    }
}

#[utoipa::path(
    patch,
    path = "/api/v1/messenger/chats/{chat_uuid}/alias",
    params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
    request_body = SetAliasDTO,
    responses(
        (status = 200, body = ApiResponse<bool>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn set_alias(
    State(state): State<Arc<AppState>>,
    Extension(user_uuid): Extension<Uuid>,
    Path(chat_uuid): Path<Uuid>,
    Json(dto): Json<SetAliasDTO>,
) -> impl IntoResponse {
    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = ChatService::new(repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.set_alias(chat_uuid, user_uuid, dto.alias).await {
        Ok(_) => into_api_response(StatusCode::OK, Some(true), None, None),
        Err(e) => {
            eprintln!("Error setting alias: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to set alias".into()]))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/messenger/chats/{chat_uuid}/members",
    params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
    responses(
        (status = 200, body = ApiResponse<Vec<ChatMemberDTO>>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn get_members(
    State(state): State<Arc<AppState>>,
    Path(chat_uuid): Path<Uuid>,
) -> impl IntoResponse {
    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = ChatService::new(repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.get_members(chat_uuid).await {
        Ok(members) => into_api_response(StatusCode::OK, Some(members), None, None),
        Err(e) => {
            eprintln!("Error fetching members: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to fetch members".into()]))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/messenger/chats/{chat_uuid}/media/counts",
    params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
    responses(
        (status = 200, body = ApiResponse<ChatMediaCountsDTO>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn get_media_counts(
    State(state): State<Arc<AppState>>,
    Path(chat_uuid): Path<Uuid>,
) -> impl IntoResponse {
    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = ChatService::new(repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.get_media_counts(chat_uuid).await {
        Ok(counts) => into_api_response(StatusCode::OK, Some(counts), None, None),
        Err(e) => {
            eprintln!("Error fetching media counts: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to fetch media counts".into()]))
        }
    }
}
