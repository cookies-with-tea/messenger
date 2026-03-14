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
use crate::messenger::dto::{CreateMessageDTO, MessageResponseDTO, UpdateMessageDTO, SearchQuery, MessageReceiptDTO};
use crate::core::dto::{ApiResponseWithPagination, PaginationDTO, ApiPaginationDTO, ApiResponse};
use crate::messenger::domain::repositories::{MessageRepository, ChatRepository};
use crate::messenger::infrastructure::postgres_repository::PostgresMessengerRepository;
use crate::messenger::application::message_service::MessageService;

#[utoipa::path(
    get,
    path = "/api/v1/messenger/chats/{chat_uuid}/messages",
    params(
        ("chat_uuid" = Uuid, Path, description = "Chat UUID"),
        ("page" = Option<i64>, Query, description = "Page number"),
        ("limit" = Option<i64>, Query, description = "Items per page"),
        ("before" = Option<Uuid>, Query, description = "Cursor for pagination")
    ),
    responses(
        (status = 200, body = ApiResponseWithPagination<MessageResponseDTO>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn get_messages(
    State(state): State<Arc<AppState>>,
    Path(chat_uuid): Path<Uuid>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let page = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(20);
    let before = params.get("before").and_then(|b| Uuid::parse_str(b).ok());

    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn MessageRepository>;
    let chat_repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = MessageService::new(repo, chat_repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.get_messages(chat_uuid, limit, page, before).await {
        Ok((messages, total)) => {
            let total_pages = (total as f64 / limit as f64).ceil() as i32;
            let pagination = PaginationDTO {
                page: page as i32,
                total: Some(total as i32),
                total_pages: Some(total_pages),
                limit: Some(limit as i32),
            };
            let data = ApiPaginationDTO { items: messages, pagination };
            into_api_response(StatusCode::OK, Some(data), None, None)
        }
        Err(e) => {
            eprintln!("Error fetching messages: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to fetch messages".into()]))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/messenger/chats/{chat_uuid}/messages",
    params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
    request_body = CreateMessageDTO,
    responses(
        (status = 201, body = ApiResponse<MessageResponseDTO>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Path(chat_uuid): Path<Uuid>,
    Extension(user_uuid): Extension<Uuid>,
    Json(dto): Json<CreateMessageDTO>,
) -> impl IntoResponse {
    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn MessageRepository>;
    let chat_repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = MessageService::new(repo, chat_repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.send_message(chat_uuid, user_uuid, dto).await {
        Ok(msg) => into_api_response(StatusCode::CREATED, Some(msg), None, None),
        Err(e) => {
            eprintln!("Error sending message: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to send message".into()]))
        }
    }
}
#[utoipa::path(
    put,
    path = "/api/v1/messenger/chats/{chat_uuid}/messages/{message_uuid}",
    params(
        ("chat_uuid" = Uuid, Path, description = "Chat UUID"),
        ("message_uuid" = Uuid, Path, description = "Message UUID")
    ),
    request_body = UpdateMessageDTO,
    responses(
        (status = 200, body = ApiResponse<MessageResponseDTO>),
        (status = 404, body = ApiResponse<bool>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn edit_message(
    State(state): State<Arc<AppState>>,
    Path((chat_uuid, message_uuid)): Path<(Uuid, Uuid)>,
    Json(dto): Json<UpdateMessageDTO>,
) -> impl IntoResponse {
    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn MessageRepository>;
    let chat_repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = MessageService::new(repo, chat_repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.edit_message(chat_uuid, message_uuid, &dto.body).await {
        Ok(Some(msg)) => into_api_response(StatusCode::OK, Some(msg), None, None),
        Ok(None) => into_api_response(StatusCode::NOT_FOUND, None, None, Some(vec!["Message not found".into()])),
        Err(e) => {
            eprintln!("Error editing message: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to edit message".into()]))
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/messenger/chats/{chat_uuid}/messages/{message_uuid}",
    params(
        ("chat_uuid" = Uuid, Path, description = "Chat UUID"),
        ("message_uuid" = Uuid, Path, description = "Message UUID")
    ),
    responses(
        (status = 200, body = ApiResponse<bool>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn delete_message(
    State(state): State<Arc<AppState>>,
    Path((chat_uuid, message_uuid)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn MessageRepository>;
    let chat_repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = MessageService::new(repo, chat_repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.delete_message(chat_uuid, message_uuid).await {
        Ok(true) => into_api_response(StatusCode::OK, Some(true), None, None),
        Ok(false) | Err(_) => into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to delete message".into()])),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/messenger/chats/{chat_uuid}/messages/delivered",
    params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
    responses(
        (status = 200, body = ApiResponse<bool>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn mark_delivered(
    State(state): State<Arc<AppState>>,
    Path(chat_uuid): Path<Uuid>,
    Extension(user_uuid): Extension<Uuid>,
) -> impl IntoResponse {
    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn MessageRepository>;
    let chat_repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = MessageService::new(repo, chat_repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.mark_delivered(chat_uuid, user_uuid).await {
        Ok(_) => into_api_response(StatusCode::OK, Some(true), None, None),
        Err(e) => {
            eprintln!("Error marking delivered: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to mark delivered".into()]))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/messenger/chats/{chat_uuid}/messages/read",
    params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
    responses(
        (status = 200, body = ApiResponse<bool>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn mark_read(
    State(state): State<Arc<AppState>>,
    Path(chat_uuid): Path<Uuid>,
    Extension(user_uuid): Extension<Uuid>,
) -> impl IntoResponse {
    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn MessageRepository>;
    let chat_repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = MessageService::new(repo, chat_repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.mark_read(chat_uuid, user_uuid).await {
        Ok(_) => into_api_response(StatusCode::OK, Some(true), None, None),
        Err(e) => {
            eprintln!("Error marking read: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to mark read".into()]))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/messenger/chats/{chat_uuid}/search",
    params(
        ("chat_uuid" = Uuid, Path, description = "Chat UUID"),
        ("q" = String, Query, description = "Search query"),
        ("limit" = Option<i64>, Query, description = "Limit results")
    ),
    responses(
        (status = 200, body = ApiResponse<Vec<MessageResponseDTO>>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn search_in_chat(
    State(state): State<Arc<AppState>>,
    Path(chat_uuid): Path<Uuid>,
    Extension(user_uuid): Extension<Uuid>,
    axum::extract::Query(params): axum::extract::Query<SearchQuery>,
) -> impl IntoResponse {
    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn MessageRepository>;
    let chat_repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = MessageService::new(repo, chat_repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.search_messages(user_uuid, Some(chat_uuid), &params.q, params.limit.unwrap_or(50)).await {
        Ok(msgs) => into_api_response(StatusCode::OK, Some(msgs), None, None),
        Err(e) => {
            eprintln!("Error searching in chat: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to search messages".into()]))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/messenger/search",
    params(
        ("q" = String, Query, description = "Search query"),
        ("limit" = Option<i64>, Query, description = "Limit results")
    ),
    responses(
        (status = 200, body = ApiResponse<Vec<MessageResponseDTO>>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn global_search(
    State(state): State<Arc<AppState>>,
    Extension(user_uuid): Extension<Uuid>,
    axum::extract::Query(params): axum::extract::Query<SearchQuery>,
) -> impl IntoResponse {
    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn MessageRepository>;
    let chat_repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = MessageService::new(repo, chat_repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.search_messages(user_uuid, None, &params.q, params.limit.unwrap_or(50)).await {
        Ok(msgs) => into_api_response(StatusCode::OK, Some(msgs), None, None),
        Err(e) => {
            eprintln!("Error global searching: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to search messages".into()]))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/messenger/chats/{chat_uuid}/messages/{message_uuid}/receipts",
    params(
        ("chat_uuid" = Uuid, Path, description = "Chat UUID"),
        ("message_uuid" = Uuid, Path, description = "Message UUID")
    ),
    responses(
        (status = 200, body = ApiResponse<Vec<MessageReceiptDTO>>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn get_message_receipts(
    State(state): State<Arc<AppState>>,
    Path((_chat_uuid, message_uuid)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn MessageRepository>;
    let chat_repo = Arc::new(PostgresMessengerRepository::new(state.pool.clone())) as Arc<dyn ChatRepository>;
    let service = MessageService::new(repo, chat_repo, state.user_ws_state.clone().map(Arc::new), state.media_base_url.clone());

    match service.get_message_receipts(message_uuid).await {
        Ok(receipts) => into_api_response(StatusCode::OK, Some(receipts), None, None),
        Err(e) => {
            eprintln!("Error fetching message receipts: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, Some(vec!["Failed to fetch receipts".into()]))
        }
    }
}
