use crate::messenger::domain::repositories::ChatRepository;
use crate::messenger::domain::repositories::MessageRepository;
use crate::messenger::dto::{
    CreateMessageDTO, DeliveryStatus, MessageResponseDTO, MessageRow, UserPreviewDTO, WsServerEvent,
};
use crate::messenger::ws::UserWsState;
use std::sync::Arc;
use uuid::Uuid;

pub struct MessageService {
    repo: Arc<dyn MessageRepository>,
    chat_repo: Arc<dyn ChatRepository>,
    ws: Option<Arc<UserWsState>>,
    media_base_url: String,
}

impl MessageService {
    pub fn new(
        repo: Arc<dyn MessageRepository>,
        chat_repo: Arc<dyn ChatRepository>,
        ws: Option<Arc<UserWsState>>,
        media_base_url: String,
    ) -> Self {
        Self {
            repo,
            chat_repo,
            ws,
            media_base_url,
        }
    }

    // ─── WebRTC Signaling ───

    pub async fn handle_call_offer(
        &self,
        chat_uuid: Uuid,
        caller_uuid: Uuid,
        sdp: String,
    ) -> Result<(), sqlx::Error> {
        if let Some(ws) = &self.ws {
            if let Ok(members) = self.chat_repo.get_member_uuids(chat_uuid).await {
                let recipients: Vec<Uuid> = members
                    .into_iter()
                    .filter(|&uid| uid != caller_uuid)
                    .collect();
                ws.broadcast_to_users(
                    &recipients,
                    WsServerEvent::CallOffer {
                        chat_uuid,
                        caller_uuid,
                        sdp,
                    },
                )
                .await;
            }
        }
        Ok(())
    }

    pub async fn handle_call_answer(
        &self,
        chat_uuid: Uuid,
        responder_uuid: Uuid,
        sdp: String,
    ) -> Result<(), sqlx::Error> {
        if let Some(ws) = &self.ws {
            if let Ok(members) = self.chat_repo.get_member_uuids(chat_uuid).await {
                let recipients: Vec<Uuid> = members
                    .into_iter()
                    .filter(|&uid| uid != responder_uuid)
                    .collect();
                ws.broadcast_to_users(
                    &recipients,
                    WsServerEvent::CallAnswer {
                        chat_uuid,
                        responder_uuid,
                        sdp,
                    },
                )
                .await;
            }
        }
        Ok(())
    }

    pub async fn handle_ice_candidate(
        &self,
        chat_uuid: Uuid,
        sender_uuid: Uuid,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<i32>,
    ) -> Result<(), sqlx::Error> {
        if let Some(ws) = &self.ws {
            if let Ok(members) = self.chat_repo.get_member_uuids(chat_uuid).await {
                let recipients: Vec<Uuid> = members
                    .into_iter()
                    .filter(|&uid| uid != sender_uuid)
                    .collect();
                ws.broadcast_to_users(
                    &recipients,
                    WsServerEvent::IceCandidate {
                        chat_uuid,
                        sender_uuid,
                        candidate,
                        sdp_mid,
                        sdp_m_line_index,
                    },
                )
                .await;
            }
        }
        Ok(())
    }

    pub async fn handle_call_reject(
        &self,
        chat_uuid: Uuid,
        user_uuid: Uuid,
    ) -> Result<(), sqlx::Error> {
        if let Some(ws) = &self.ws {
            if let Ok(members) = self.chat_repo.get_member_uuids(chat_uuid).await {
                let recipients: Vec<Uuid> = members
                    .into_iter()
                    .filter(|&uid| uid != user_uuid)
                    .collect();
                ws.broadcast_to_users(
                    &recipients,
                    WsServerEvent::CallReject {
                        chat_uuid,
                        user_uuid,
                    },
                )
                .await;
            }
        }
        Ok(())
    }

    pub async fn handle_call_end(
        &self,
        chat_uuid: Uuid,
        user_uuid: Uuid,
    ) -> Result<(), sqlx::Error> {
        if let Some(ws) = &self.ws {
            if let Ok(members) = self.chat_repo.get_member_uuids(chat_uuid).await {
                let recipients: Vec<Uuid> = members
                    .into_iter()
                    .filter(|&uid| uid != user_uuid)
                    .collect();
                ws.broadcast_to_users(
                    &recipients,
                    WsServerEvent::CallEnd {
                        chat_uuid,
                        user_uuid,
                    },
                )
                .await;
            }
        }
        Ok(())
    }

    pub async fn get_messages(
        &self,
        chat_uuid: Uuid,
        limit: i64,
        page: i64,
        before: Option<Uuid>,
    ) -> Result<(Vec<MessageResponseDTO>, i64), sqlx::Error> {
        let rows = if let Some(cursor) = before {
            self.repo
                .find_all_before_cursor(chat_uuid, cursor, limit)
                .await?
        } else {
            let offset = (page - 1) * limit;
            self.repo.find_all_in_chat(chat_uuid, limit, offset).await?
        };
        let total = self.repo.count_all_in_chat(chat_uuid).await?;

        let mut dtos = Vec::new();
        for row in rows {
            dtos.push(self.map_message_row_to_dto_async(row).await);
        }
        Ok((dtos, total))
    }

    pub async fn get_messages_before(
        &self,
        chat_uuid: Uuid,
        before_uuid: Uuid,
        limit: i64,
    ) -> Result<Vec<MessageResponseDTO>, sqlx::Error> {
        let rows = self
            .repo
            .find_all_before_cursor(chat_uuid, before_uuid, limit)
            .await?;
        let mut dtos = Vec::new();
        for row in rows {
            dtos.push(self.map_message_row_to_dto_async(row).await);
        }
        Ok(dtos)
    }

    pub async fn send_message(
        &self,
        chat_uuid: Uuid,
        sender_uuid: Uuid,
        dto: CreateMessageDTO,
    ) -> Result<MessageResponseDTO, sqlx::Error> {
        let row = self
            .repo
            .create(
                chat_uuid,
                sender_uuid,
                dto.reply_to_uuid,
                &dto.body,
                dto.media_uuid,
            )
            .await?;
        let dto = self.map_message_row_to_dto_async(row).await;

        if let Some(ws) = &self.ws {
            if let Ok(members) = self.chat_repo.get_member_uuids(chat_uuid).await {
                ws.broadcast_to_users(&members, WsServerEvent::NewMessage(dto.clone()))
                    .await;
            }
        }

        Ok(dto)
    }

    pub async fn edit_message(
        &self,
        chat_uuid: Uuid,
        uuid: Uuid,
        body: &str,
    ) -> Result<Option<MessageResponseDTO>, sqlx::Error> {
        let row = self.repo.update_body(uuid, body).await?;
        if let Some(row) = row {
            let dto = self.map_message_row_to_dto_async(row).await;
            if let Some(ws) = &self.ws {
                if let Ok(members) = self.chat_repo.get_member_uuids(chat_uuid).await {
                    ws.broadcast_to_users(&members, WsServerEvent::MessageEdited(dto.clone()))
                        .await;
                }
            }
            Ok(Some(dto))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_message(&self, chat_uuid: Uuid, uuid: Uuid) -> Result<bool, sqlx::Error> {
        let affected = self.repo.delete(uuid).await?;
        if affected > 0 {
            if let Some(ws) = &self.ws {
                if let Ok(members) = self.chat_repo.get_member_uuids(chat_uuid).await {
                    ws.broadcast_to_users(
                        &members,
                        WsServerEvent::MessageDeleted { uuid, chat_uuid },
                    )
                    .await;
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn mark_delivered(
        &self,
        chat_uuid: Uuid,
        user_uuid: Uuid,
    ) -> Result<(), sqlx::Error> {
        self.repo
            .upsert_statuses(chat_uuid, user_uuid, DeliveryStatus::Delivered)
            .await?;
        // Broadcast status update (this might be too noisy if done for every message,
        // usually we sending it per chat or for last message)
        // For now, let's just stick to the basic implementation.
        Ok(())
    }

    pub async fn mark_read(&self, chat_uuid: Uuid, user_uuid: Uuid) -> Result<(), sqlx::Error> {
        self.repo
            .upsert_statuses(chat_uuid, user_uuid, DeliveryStatus::Read)
            .await?;
        if let Some(ws) = &self.ws {
            if let Ok(members) = self.chat_repo.get_member_uuids(chat_uuid).await {
                // Simplified: we send a general status update.
                // Ideally we'd need to know which messages were marked read.
                // But the WsServerEvent::StatusUpdated requires message_uuid.
                // For direct DDD port, let's see what WsServerEvent expects.
            }
        }
        Ok(())
    }

    pub async fn toggle_reaction(
        &self,
        chat_uuid: Uuid,
        message_uuid: Uuid,
        user_uuid: Uuid,
        emoji: &str,
    ) -> Result<bool, sqlx::Error> {
        let is_added = self
            .repo
            .add_reaction(message_uuid, user_uuid, emoji)
            .await?;
        if let Some(ws) = &self.ws {
            if let Ok(members) = self.chat_repo.get_member_uuids(chat_uuid).await {
                ws.broadcast_to_users(
                    &members,
                    WsServerEvent::MessageReactionUpdated {
                        chat_uuid,
                        message_uuid,
                        user_uuid,
                        emoji: emoji.to_string(),
                        is_added,
                    },
                )
                .await;
            }
        }
        Ok(is_added)
    }

    pub async fn search_messages(
        &self,
        user_uuid: Uuid,
        chat_uuid: Option<Uuid>,
        query: &str,
        limit: i64,
    ) -> Result<Vec<MessageResponseDTO>, sqlx::Error> {
        let rows = self.repo.search(user_uuid, chat_uuid, query, limit).await?;
        let mut dtos = Vec::new();
        for row in rows {
            dtos.push(self.map_message_row_to_dto_async(row).await);
        }
        Ok(dtos)
    }

    pub async fn get_media(
        &self,
        chat_uuid: Uuid,
        media_type: Option<String>,
        limit: i64,
        page: i64,
    ) -> Result<(Vec<MessageResponseDTO>, i64), sqlx::Error> {
        let offset = (page - 1) * limit;
        let rows = self
            .repo
            .find_media(chat_uuid, media_type, limit, offset)
            .await?;
        // For simplicity, we can reuse count_all_in_chat or add a specific count_media if needed.
        // But shared media view usually doesn't need a strict total count if it's infinite scroll.
        // However, find_media in repo doesn't return total.
        // Let's just return the rows for now.
        let mut dtos = Vec::new();
        for row in rows {
            dtos.push(self.map_message_row_to_dto_async(row).await);
        }
        Ok((dtos, 0)) // 0 as total for now, or we could implement count_media
    }

    pub async fn get_message_receipts(
        &self,
        message_uuid: Uuid,
    ) -> Result<Vec<crate::messenger::dto::MessageReceiptDTO>, sqlx::Error> {
        let rows = self.repo.get_receipts(message_uuid).await?;
        let mut dtos = Vec::new();
        for row in rows {
            let mut is_online = false;
            if let Some(ws) = &self.ws {
                is_online = ws.is_online(row.user_uuid).await;
            }
            dtos.push(crate::messenger::dto::MessageReceiptDTO {
                user: crate::messenger::dto::UserPreviewDTO {
                    uuid: row.user_uuid,
                    first_name: row.first_name,
                    second_name: row.second_name,
                    avatar: row
                        .avatar_uuid
                        .map(|avatar_uuid| crate::core::dto::MediaDTO {
                            uuid: avatar_uuid,
                            url: format!("{}/media/image/{}.png", self.media_base_url, avatar_uuid),
                            alt: None,
                            title: None,
                            media_type: Some("image".to_string()),
                        }),
                    is_online,
                    last_seen_at: Some(row.created_at),
                },
                status: row.status,
                created_at: row.created_at,
            });
        }
        Ok(dtos)
    }

    pub async fn typing(
        &self,
        chat_uuid: Uuid,
        user_uuid: Uuid,
        is_typing: bool,
    ) -> Result<(), sqlx::Error> {
        if let Some(ws) = &self.ws {
            if let Ok(members) = self.chat_repo.get_member_uuids(chat_uuid).await {
                ws.broadcast_to_users(
                    &members,
                    WsServerEvent::Typing {
                        chat_uuid,
                        user_uuid,
                        is_typing,
                    },
                )
                .await;
            }
        }
        Ok(())
    }

    pub async fn toggle_pin(
        &self,
        chat_uuid: Uuid,
        uuid: Uuid,
        is_pinned: bool,
    ) -> Result<(), sqlx::Error> {
        self.repo.update_pin(uuid, is_pinned).await?;
        if let Some(ws) = &self.ws {
            if let Ok(members) = self.chat_repo.get_member_uuids(chat_uuid).await {
                ws.broadcast_to_users(
                    &members,
                    WsServerEvent::MessagePinned {
                        uuid,
                        chat_uuid,
                        is_pinned,
                    },
                )
                .await;
            }
        }
        Ok(())
    }

    async fn map_message_row_to_dto_async(&self, row: MessageRow) -> MessageResponseDTO {
        let mut is_online = false;
        if let Some(ws) = &self.ws {
            is_online = ws.is_online(row.sender_uuid).await;
        }

        MessageResponseDTO {
            uuid: row.uuid,
            chat_uuid: row.chat_uuid,
            sender_uuid: row.sender_uuid,
            reply_to_uuid: row.reply_to_uuid,
            body: row.body,
            is_edited: row.is_edited,
            is_deleted: row.is_deleted,
            is_pinned: row.is_pinned,
            created_at: row.created_at,
            updated_at: row.updated_at,
            sender: Some(UserPreviewDTO {
                uuid: row.sender_uuid,
                first_name: row.sender_first_name,
                second_name: row.sender_second_name,
                avatar: row
                    .sender_avatar_uuid
                    .map(|avatar_uuid| crate::core::dto::MediaDTO {
                        uuid: avatar_uuid,
                        url: format!("{}/media/image/{}.png", self.media_base_url, avatar_uuid),
                        alt: None,
                        title: None,
                        media_type: Some("image".to_string()),
                    }),
                is_online,
                last_seen_at: row.sender_last_seen_at,
            }),
            delivered_count: row.delivered_count,
            read_count: row.read_count,
            my_status: row.my_status,
            reply_body_preview: row.reply_body_preview,
            media: row.media_uuid.map(|media_uuid| crate::core::dto::MediaDTO {
                uuid: media_uuid,
                url: row.media_url.clone().unwrap_or_else(|| {
                    let ext = if row.media_type.as_deref() == Some("image") { ".png" } else { "" };
                    format!(
                        "{}/media/{}/{}{}",
                        self.media_base_url,
                        row.media_type.as_deref().unwrap_or("image"),
                        media_uuid,
                        ext
                    )
                }),
                alt: row.media_alt,
                title: row.media_title,
                media_type: row.media_type,
            }),
            reactions: vec![], // Reactions fetched separately if needed or joined
        }
    }
}
