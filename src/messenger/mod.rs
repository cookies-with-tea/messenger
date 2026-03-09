//! # messenger
//!
//! Плоский модуль мессенджера.
//!
//! ## Файлы
//! - `dto.rs`      — все типы запросов/ответов, WS-события
//! - `ws.rs`       — [`WsState`] broadcast-менеджер, [`handle_socket`]
//! - `handlers.rs` — REST + WS хендлеры, [`messenger_router`]
//!
//! ## Подключение
//!
//! **`src/main.rs`** (или `src/server.rs`):
//! ```rust
//! mod messenger; // если src/messenger/mod.rs
//!
//! use messenger::handlers::messenger_router;
//!
//! let app = Router::new()
//!     .merge(messenger_router())
//!     .with_state(Arc::new(app_state));
//! ```
//!
//! **`AppState`** — добавить поле:
//! ```rust
//! use crate::messenger::ws::WsState;
//!
//! pub struct AppState {
//!     pub pool:     sqlx::PgPool,
//!     pub i18n:     ...,
//!     pub ws_state: Option<WsState>,
//! }
//!
//! // Инициализация:
//! ws_state: Some(WsState::new()),
//! ```
//!
//! **`Cargo.toml`** — добавить фичу `ws`:
//! ```toml
//! axum = { version = "0.8.1", features = ["multipart", "macros", "ws"] }
//! ```
//!
//! ## i18n ключи
//! ```
//! messenger.not_member                     = "You are not a member of this chat"
//! messenger.not_admin                      = "Only admins can perform this action"
//! messenger.chat_not_found                 = "Chat not found"
//! messenger.message_not_found_or_forbidden = "Message not found or access denied"
//! messenger.direct_needs_one_member        = "Direct chat requires exactly one other participant"
//! messenger.group_needs_name               = "Group chat requires a name"
//! ```
//!
//! ## WebSocket протокол (клиент → сервер)
//! ```json
//! { "action": "send_message",  "payload": { "body": "Hi!", "reply_to_uuid": null } }
//! { "action": "edit_message",  "payload": { "uuid": "...", "body": "Fixed" } }
//! { "action": "delete_message","payload": { "uuid": "..." } }
//! { "action": "mark_delivered","payload": { "chat_uuid": "..." } }
//! { "action": "mark_read",     "payload": { "chat_uuid": "..." } }
//! { "action": "typing",        "payload": { "is_typing": true } }
//! ```
//!
//! ## WebSocket протокол (сервер → клиент)
//! ```json
//! { "event": "new_message",    "payload": { ...MessageResponseDTO } }
//! { "event": "message_edited", "payload": { ...MessageResponseDTO } }
//! { "event": "message_deleted","payload": { "uuid": "...", "chat_uuid": "..." } }
//! { "event": "status_updated", "payload": { "chat_uuid": "...", "user_uuid": "...", "status": "read" } }
//! { "event": "typing",         "payload": { "chat_uuid": "...", "user_uuid": "...", "is_typing": true } }
//! { "event": "member_joined",  "payload": { ...ChatMemberDTO } }
//! { "event": "member_left",    "payload": { "chat_uuid": "...", "user_uuid": "..." } }
//! { "event": "error",          "payload": { "message": "..." } }
//! ```

pub mod dto;
pub mod handlers;
pub mod ws;
pub mod ws_echo;

pub use handlers::{router, ws_router};
pub use ws::WsState;
