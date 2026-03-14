use chrono::{DateTime, Utc};
use uuid::Uuid;
use super::super::dto::ChatType;

#[derive(Debug, Clone)]
pub struct Chat {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub chat_type: ChatType,
    pub created_by: Uuid,
    pub is_archived: bool,
    pub avatar: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub uuid: Uuid,
    pub chat_uuid: Uuid,
    pub sender_uuid: Uuid,
    pub reply_to_uuid: Option<Uuid>,
    pub body: String,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_pinned: bool,
    pub media_uuid: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ChatMember {
    pub uuid: Uuid,
    pub chat_uuid: Uuid,
    pub user_uuid: Uuid,
    pub is_admin: bool,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MessageReaction {
    pub message_uuid: Uuid,
    pub user_uuid: Uuid,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}
