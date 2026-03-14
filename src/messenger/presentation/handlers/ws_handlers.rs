use std::sync::Arc;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::IntoResponse,
    Extension,
};
use uuid::Uuid;
use crate::AppState;
use crate::messenger::dto::WsClientAction;
use crate::messenger::ws::{handle_user_socket, handle_socket};
use crate::messenger::domain::repositories::{MessageRepository, ChatRepository};
use crate::messenger::infrastructure::postgres_repository::PostgresMessengerRepository;
use crate::messenger::application::message_service::MessageService;

#[utoipa::path(
    get,
    path = "/ws/chats/{chat_uuid}",
    params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
    responses(
        (status = 101, description = "WebSocket upgrade"),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(chat_uuid): Path<Uuid>,
    Extension(user_uuid): Extension<Uuid>,
) -> impl IntoResponse {
    let ws_state = state.ws_state.clone().expect("WsState must be set");

    ws.on_upgrade(move |socket| {
        let state_inner = state.clone();
        handle_socket(socket, chat_uuid, user_uuid, ws_state, move |_chat_uuid, _user_uuid, _raw| {
            let _state_async = state_inner.clone();
            async move {
                // Logic for handling chat-specific actions
                None
            }
        })
    })
}

#[utoipa::path(
    get,
    path = "/ws/user",
    responses(
        (status = 101, description = "WebSocket upgrade"),
    ),
    tag = "Messenger",
    security(("bearer_auth" = []))
)]
pub async fn ws_user_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Extension(user_uuid): Extension<Uuid>,
) -> impl IntoResponse {
    let user_ws_state = state.user_ws_state.clone().expect("UserWsState must be set");

    ws.on_upgrade(move |socket| {
        let state_inner = state.clone();
        handle_user_socket(socket, user_uuid, user_ws_state, state.pool.clone(), move |user_uuid, raw| {
            let state_async = state_inner.clone();
            async move {
                let action: WsClientAction = match serde_json::from_str(&raw) {
                    Ok(a) => a,
                    Err(_) => return,
                };

                let repo = Arc::new(PostgresMessengerRepository::new(state_async.pool.clone())) as Arc<dyn MessageRepository>;
                let chat_repo = Arc::new(PostgresMessengerRepository::new(state_async.pool.clone())) as Arc<dyn ChatRepository>;
                let service = MessageService::new(repo, chat_repo, state_async.user_ws_state.clone().map(Arc::new), state_async.media_base_url.clone());

                match action {
                    WsClientAction::SendMessage { chat_uuid, body, reply_to_uuid, media_uuid } => {
                        let dto = crate::messenger::dto::CreateMessageDTO { body, reply_to_uuid, media_uuid };
                        let _ = service.send_message(chat_uuid, user_uuid, dto).await;
                    }
                    WsClientAction::EditMessage { chat_uuid, uuid, body } => {
                        let _ = service.edit_message(chat_uuid, uuid, &body).await;
                    }
                    WsClientAction::DeleteMessage { chat_uuid, uuid } => {
                        let _ = service.delete_message(chat_uuid, uuid).await;
                    }
                    WsClientAction::Typing { chat_uuid, is_typing } => {
                        let _ = service.typing(chat_uuid, user_uuid, is_typing).await;
                    }
                    WsClientAction::MarkDelivered { chat_uuid } => {
                        let _ = service.mark_delivered(chat_uuid, user_uuid).await;
                    }
                    WsClientAction::MarkRead { chat_uuid } => {
                        let _ = service.mark_read(chat_uuid, user_uuid).await;
                    }
                    WsClientAction::ReactToMessage { chat_uuid, message_uuid, emoji } => {
                        let _ = service.toggle_reaction(chat_uuid, message_uuid, user_uuid, &emoji).await;
                    }
                    WsClientAction::TogglePinMessage { chat_uuid, uuid, is_pinned } => {
                        let _ = service.toggle_pin(chat_uuid, uuid, is_pinned).await;
                    }
                    _ => {}
                }
            }
        })
    })
}
