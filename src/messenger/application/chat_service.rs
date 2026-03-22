use std::sync::Arc;
use uuid::Uuid;
use crate::messenger::domain::repositories::ChatRepository;
use crate::messenger::dto::{ChatResponseDTO, CreateChatDTO, UserPreviewDTO, ChatRow};
use crate::messenger::ws::UserWsState;

pub struct ChatService {
    repo: Arc<dyn ChatRepository>,
    ws: Option<Arc<UserWsState>>,
    media_base_url: String,
}

impl ChatService {
    pub fn new(repo: Arc<dyn ChatRepository>, ws: Option<Arc<UserWsState>>, media_base_url: String) -> Self {
        Self { repo, ws, media_base_url }
    }

    pub async fn get_chats(&self, user_uuid: Uuid, limit: i64, page: i64) -> Result<(Vec<ChatResponseDTO>, i64), sqlx::Error> {
        let offset = (page - 1) * limit;
        let chats_rows = self.repo.find_all_for_user(user_uuid, limit, offset).await?;
        let total = self.repo.count_for_user(user_uuid).await?;

        let mut dtos = Vec::new();
        for row in chats_rows {
            dtos.push(self.map_chat_row_to_dto_async(row).await);
        }
        Ok((dtos, total))
    }

    pub async fn create_chat(&self, creator_uuid: Uuid, dto: CreateChatDTO) -> Result<Uuid, sqlx::Error> {
        // Business logic: check number of members for direct chat, etc.
        let chat_uuid = self.repo.create(
            &dto.name,
            &dto.description,
            match dto.chat_type {
                crate::messenger::dto::ChatType::Direct => "direct",
                crate::messenger::dto::ChatType::Group => "group",
            },
            creator_uuid,
            &dto.avatar
        ).await?;

        // Add creator
        self.repo.add_member(chat_uuid, creator_uuid, true).await?;

        // Add others
        for member_uuid in dto.member_uuids {
            self.repo.add_member(chat_uuid, member_uuid, false).await?;
        }

        Ok(chat_uuid)
    }

    pub async fn get_chat(&self, chat_uuid: Uuid) -> Result<Option<ChatResponseDTO>, sqlx::Error> {
        let row = self.repo.find_by_uuid(chat_uuid).await?;
        if let Some(r) = row {
            Ok(Some(self.map_chat_row_to_dto_async(r).await))
        } else {
            Ok(None)
        }
    }

    pub async fn update_chat(&self, uuid: Uuid, dto: crate::messenger::dto::UpdateChatDTO) -> Result<(), sqlx::Error> {
        let avatar_uuid = dto.avatar.and_then(|s| Uuid::parse_str(&s).ok());
        self.repo.update_metadata(uuid, dto.name, dto.description, avatar_uuid, dto.is_archived).await
    }

    pub async fn set_alias(&self, chat_uuid: Uuid, user_uuid: Uuid, alias: Option<String>) -> Result<(), sqlx::Error> {
        self.repo.set_alias(chat_uuid, user_uuid, alias).await
    }

    pub async fn delete_chat(&self, uuid: Uuid) -> Result<(), sqlx::Error> {
        self.repo.delete(uuid).await
    }

    pub async fn get_members(&self, chat_uuid: Uuid) -> Result<Vec<crate::messenger::dto::ChatMemberDTO>, sqlx::Error> {
        self.repo.find_members(chat_uuid).await
    }

    pub async fn get_media_counts(&self, chat_uuid: Uuid) -> Result<crate::messenger::dto::ChatMediaCountsDTO, sqlx::Error> {
        self.repo.get_media_counts(chat_uuid).await
    }

    async fn map_chat_row_to_dto_async(&self, row: ChatRow) -> ChatResponseDTO {
        let mut is_online = false;
        if let (Some(ws), Some(sender_uuid)) = (&self.ws, row.sender_uuid) {
            is_online = ws.is_online(sender_uuid).await;
        }

        ChatResponseDTO {
            uuid: row.uuid,
            name: row.name,
            description: row.description,
            chat_type: row.chat_type,
            created_by: row.created_by,
            is_archived: row.is_archived,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_message_body: row.last_message_body,
            last_message_at: row.last_message_at,
            unread_count: row.unread_count,
            member_count: row.member_count,
            sender: row.sender_uuid.map(|uuid| UserPreviewDTO {
                uuid,
                first_name: row.sender_first_name,
                second_name: row.sender_second_name,
                avatar: row.sender_avatar_uuid.map(|avatar_uuid| crate::core::dto::MediaDTO {
                    uuid: avatar_uuid,
                    url: row.sender_avatar_url.unwrap_or_else(|| format!("{}/media/image/{}", self.media_base_url, avatar_uuid)),
                    alt: None,
                    title: None,
                    media_type: Some("image".to_string()),
                }), 
                is_online,
                last_seen_at: row.sender_last_seen_at,
            }),
            alias: row.alias,
        }
    }
}
