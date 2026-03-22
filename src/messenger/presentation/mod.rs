use std::sync::Arc;
use axum::{routing, Router};
use crate::AppState;
use crate::messenger::presentation::handlers::{chat_handlers, message_handlers, ws_handlers};

pub mod handlers;

pub fn messenger_router() -> Router<Arc<AppState>> {
    Router::new()
        // Chats
        .route("/", routing::get(chat_handlers::get_chats).post(chat_handlers::create_chat))
        .route("/search", routing::get(message_handlers::global_search))
        .route("/{chat_uuid}", routing::get(chat_handlers::get_chat).put(chat_handlers::update_chat).delete(chat_handlers::delete_chat))
        .route("/{chat_uuid}/alias", routing::patch(chat_handlers::set_alias))
        .route("/{chat_uuid}/members", routing::get(chat_handlers::get_members))
        .route("/{chat_uuid}/media", routing::get(message_handlers::get_media))
        .route("/{chat_uuid}/media/counts", routing::get(chat_handlers::get_media_counts))
        
        // Messages
        .route("/{chat_uuid}/messages", routing::get(message_handlers::get_messages).post(message_handlers::send_message))
        .route("/{chat_uuid}/messages/{message_uuid}", routing::put(message_handlers::edit_message).delete(message_handlers::delete_message))
        .route("/{chat_uuid}/messages/delivered", routing::post(message_handlers::mark_delivered))
        .route("/{chat_uuid}/messages/read", routing::post(message_handlers::mark_read))
        .route("/{chat_uuid}/messages/{message_uuid}/receipts", routing::get(message_handlers::get_message_receipts))
        .route("/{chat_uuid}/search", routing::get(message_handlers::search_in_chat))
}

pub fn ws_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{chat_uuid}", routing::get(ws_handlers::ws_upgrade))
}

pub fn ws_user_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", routing::get(ws_handlers::ws_user_upgrade))
}
