use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::core::dto::MediaDTO;

// ────────────────────────────────────────────────────────────────
// Enums
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct UserPreviewDTO {
    pub first_name: Option<String>,
    pub second_name: Option<String>,

    #[sqlx(default)]
    pub avatar: Option<MediaDTO>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ChatType {
    Direct,
    Group,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "delivery_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    Delivered,
    Read,
}

// ────────────────────────────────────────────────────────────────
// Chat
// ────────────────────────────────────────────────────────────────

#[derive(Debug, FromRow)]
pub struct ChatRow {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub chat_type: ChatType,
    pub created_by: Uuid,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    #[sqlx(default)]
    pub last_message_body: Option<String>,
    #[sqlx(default)]
    pub last_message_at: Option<DateTime<Utc>>,
    #[sqlx(default)]
    pub unread_count: Option<i64>,
    #[sqlx(default)]
    pub member_count: Option<i64>,

    // 👇 Данные собеседника (для Direct-чатов)
    #[sqlx(default)]
    pub sender_uuid: Option<Uuid>,
    #[sqlx(default)]
    pub sender_first_name: Option<String>,
    #[sqlx(default)]
    pub sender_second_name: Option<String>,
    #[sqlx(default)]
    pub sender_avatar_uuid: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct ChatResponseDTO {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub chat_type: ChatType,
    pub created_by: Uuid,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    #[sqlx(default)]
    pub last_message_body: Option<String>,
    #[sqlx(default)]
    pub last_message_at: Option<DateTime<Utc>>,
    #[sqlx(default)]
    pub unread_count: Option<i64>,
    #[sqlx(default)]
    pub member_count: Option<i64>,

    pub sender: Option<UserPreviewDTO>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateChatDTO {
    /// Только для group
    pub name: Option<String>,
    pub description: Option<String>,
    pub chat_type: ChatType,
    pub avatar: Option<String>,
    /// UUID других участников (без себя — добавляется автоматически)
    pub member_uuids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateChatDTO {
    pub name: Option<String>,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub is_archived: Option<bool>,
}

// ────────────────────────────────────────────────────────────────
// Chat member
// ────────────────────────────────────────────────────────────────

#[derive(Debug, FromRow)]
pub struct MessageRow {
    pub uuid: Uuid,
    pub chat_uuid: Uuid,
    pub sender_uuid: Uuid,
    pub reply_to_uuid: Option<Uuid>,
    pub body: String,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // 👇 Плоские поля из guest_user
    #[sqlx(default)]
    pub sender_first_name: Option<String>,
    #[sqlx(default)]
    pub sender_second_name: Option<String>,
    #[sqlx(default)]
    pub sender_avatar_uuid: Option<Uuid>, // 👈 UUID (ссылка на media)

    // Статусы
    #[sqlx(default)]
    pub delivered_count: Option<i64>,
    #[sqlx(default)]
    pub read_count: Option<i64>,
    #[sqlx(default)]
    pub my_status: Option<DeliveryStatus>,

    // Превью цитаты
    #[sqlx(default)]
    pub reply_body_preview: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct ChatMemberDTO {
    pub uuid: Uuid,
    pub chat_uuid: Uuid,
    pub user_uuid: Uuid,
    pub is_admin: bool,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    // из JOIN guest_user
    #[sqlx(default)]
    pub first_name: Option<String>,
    #[sqlx(default)]
    pub last_name: Option<String>,
    #[sqlx(default)]
    pub avatar: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddMemberDTO {
    pub user_uuid: Uuid,
    #[serde(default)]
    pub is_admin: bool,
}

// ────────────────────────────────────────────────────────────────
// Message
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct MessageResponseDTO {
  pub uuid: Uuid,
  pub chat_uuid: Uuid,
  pub sender_uuid: Uuid,
  pub reply_to_uuid: Option<Uuid>,
  pub body: String,
  pub is_edited: bool,
  pub is_deleted: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,

  // 👇 Вложенный sender вместо плоских полей
  pub sender: Option<UserPreviewDTO>,

  // статусы
  #[sqlx(default)]
  pub delivered_count: Option<i64>,
  #[sqlx(default)]
  pub read_count: Option<i64>,
  #[sqlx(default)]
  pub my_status: Option<DeliveryStatus>,
  // превью цитируемого
  #[sqlx(default)]
  pub reply_body_preview: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMessageDTO {
    pub body: String,
    pub reply_to_uuid: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMessageDTO {
    pub body: String,
}

// ────────────────────────────────────────────────────────────────
// Query params
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct MessageQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    /// Курсорная пагинация — загрузить сообщения старше этого UUID
    pub before_uuid: Option<Uuid>,
}

// ────────────────────────────────────────────────────────────────
// WebSocket events
// ────────────────────────────────────────────────────────────────

/// События, которые сервер шлёт клиенту по WS
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "event", content = "payload", rename_all = "snake_case")]
pub enum WsServerEvent {
    NewMessage(MessageResponseDTO),
    MessageEdited(MessageResponseDTO),
    MessageDeleted { uuid: Uuid, chat_uuid: Uuid },
    StatusUpdated { chat_uuid: Uuid, message_uuid: Uuid, user_uuid: Uuid, status: DeliveryStatus },
    Typing { chat_uuid: Uuid, user_uuid: Uuid, is_typing: bool },
    MemberJoined(ChatMemberDTO),
    MemberLeft { chat_uuid: Uuid, user_uuid: Uuid },
    Error { message: String },
}

/// Действия, которые клиент отправляет серверу по WS
#[derive(Debug, Deserialize, ToSchema)]
#[serde(tag = "action", content = "payload", rename_all = "snake_case")]
pub enum WsClientAction {
    SendMessage { body: String, reply_to_uuid: Option<Uuid> },
    EditMessage  { uuid: Uuid, body: String },
    DeleteMessage { uuid: Uuid },
    MarkDelivered { chat_uuid: Uuid },
    MarkRead      { chat_uuid: Uuid },
    Typing        { is_typing: bool },
}


#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: String,
}
