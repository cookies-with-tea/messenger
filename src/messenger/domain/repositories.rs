use async_trait::async_trait;
use uuid::Uuid;
use crate::messenger::dto::{ChatRow, MessageRow, ReactionDTO, DeliveryStatus, ChatMemberDTO};

#[async_trait]
pub trait ChatRepository: Send + Sync {
    async fn find_all_for_user(&self, user_uuid: Uuid, limit: i64, offset: i64) -> Result<Vec<ChatRow>, sqlx::Error>;
    async fn count_for_user(&self, user_uuid: Uuid) -> Result<i64, sqlx::Error>;
    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<ChatRow>, sqlx::Error>;
    async fn create(&self, name: &Option<String>, description: &Option<String>, chat_type: &str, creator_uuid: Uuid, avatar: &Option<Uuid>) -> Result<Uuid, sqlx::Error>;
    async fn update_metadata(&self, uuid: Uuid, name: Option<String>, description: Option<String>, avatar: Option<Uuid>, is_archived: Option<bool>) -> Result<(), sqlx::Error>;
    async fn set_alias(&self, chat_uuid: Uuid, user_uuid: Uuid, alias: Option<String>) -> Result<(), sqlx::Error>;
    async fn add_member(&self, chat_uuid: Uuid, user_uuid: Uuid, is_admin: bool) -> Result<(), sqlx::Error>;
    async fn remove_member(&self, chat_uuid: Uuid, user_uuid: Uuid) -> Result<(), sqlx::Error>;
    async fn get_member_uuids(&self, chat_uuid: Uuid) -> Result<Vec<Uuid>, sqlx::Error>;
    async fn find_members(&self, chat_uuid: Uuid) -> Result<Vec<ChatMemberDTO>, sqlx::Error>;
    async fn is_member(&self, chat_uuid: Uuid, user_uuid: Uuid) -> Result<bool, sqlx::Error>;
    async fn is_admin(&self, chat_uuid: Uuid, user_uuid: Uuid) -> Result<bool, sqlx::Error>;
    async fn delete(&self, uuid: Uuid) -> Result<(), sqlx::Error>;
    async fn get_media_counts(&self, chat_uuid: Uuid) -> Result<crate::messenger::dto::ChatMediaCountsDTO, sqlx::Error>;
    async fn find_direct_chat(&self, user1: Uuid, user2: Uuid) -> Result<Option<Uuid>, sqlx::Error>;
}

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn find_all_in_chat(&self, chat_uuid: Uuid, limit: i64, offset: i64) -> Result<Vec<MessageRow>, sqlx::Error>;
    async fn find_all_before_cursor(&self, chat_uuid: Uuid, before_uuid: Uuid, limit: i64) -> Result<Vec<MessageRow>, sqlx::Error>;
    async fn count_all_in_chat(&self, chat_uuid: Uuid) -> Result<i64, sqlx::Error>;
    async fn create(&self, chat_uuid: Uuid, sender_uuid: Uuid, reply_to_uuid: Option<Uuid>, body: &str, media_uuid: Option<Uuid>) -> Result<MessageRow, sqlx::Error>;
    async fn update_body(&self, uuid: Uuid, body: &str) -> Result<Option<MessageRow>, sqlx::Error>;
    async fn delete(&self, uuid: Uuid) -> Result<u64, sqlx::Error>;
    async fn upsert_statuses(&self, chat_uuid: Uuid, user_uuid: Uuid, status: DeliveryStatus) -> Result<(), sqlx::Error>;
    async fn add_reaction(&self, message_uuid: Uuid, user_uuid: Uuid, emoji: &str) -> Result<bool, sqlx::Error>;
    async fn get_reactions(&self, message_uuid: Uuid) -> Result<Vec<ReactionDTO>, sqlx::Error>;
    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<MessageRow>, sqlx::Error>;
    async fn search(&self, user_uuid: Uuid, chat_uuid: Option<Uuid>, query: &str, limit: i64) -> Result<Vec<MessageRow>, sqlx::Error>;
    async fn get_receipts(&self, message_uuid: Uuid) -> Result<Vec<crate::messenger::dto::MessageReceiptRow>, sqlx::Error>;
    async fn update_pin(&self, uuid: Uuid, is_pinned: bool) -> Result<(), sqlx::Error>;
    async fn find_media(&self, chat_uuid: Uuid, media_type: Option<String>, limit: i64, offset: i64) -> Result<Vec<MessageRow>, sqlx::Error>;
}
