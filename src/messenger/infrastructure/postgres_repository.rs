use async_trait::async_trait;
use sqlx::{Pool, Postgres, query_as};
use uuid::Uuid;
use crate::messenger::dto::{ChatRow, MessageRow, ReactionDTO, DeliveryStatus, ChatMemberDTO};
use crate::messenger::domain::repositories::{ChatRepository, MessageRepository};

pub struct PostgresMessengerRepository {
    pool: Pool<Postgres>,
}

impl PostgresMessengerRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChatRepository for PostgresMessengerRepository {
    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<ChatRow>, sqlx::Error> {
        query_as::<_, ChatRow>(
            r#"
            SELECT
                c.uuid, c.name, c.description, c.chat_type, c.created_by,
                c.is_archived, c.created_at, c.updated_at,
                lm.body AS last_message_body,
                lm.created_at AS last_message_at,
                NULL::bigint AS unread_count,
                (SELECT COUNT(*) FROM chat_member cm WHERE cm.chat_uuid = c.uuid AND cm.left_at IS NULL) AS member_count,
                gu.uuid AS sender_uuid,
                gu.first_name AS sender_first_name,
                gu.second_name AS sender_second_name,
                gu.avatar_uuid AS sender_avatar_uuid,
                med_avatar.url AS sender_avatar_url,
                gu.last_seen_at AS sender_last_seen_at,
                gu.is_online AS sender_is_online,
                cm_other.alias AS alias
            FROM chat c
            LEFT JOIN chat_member cm_other ON cm_other.chat_uuid = c.uuid 
                AND c.chat_type = 'direct'
            LEFT JOIN guest_user gu ON gu.uuid = cm_other.user_uuid
            LEFT JOIN media med_avatar ON med_avatar.uuid = gu.avatar_uuid
            LEFT JOIN LATERAL (
                SELECT body, created_at FROM message
                WHERE chat_uuid = c.uuid AND is_deleted = FALSE
                ORDER BY created_at DESC LIMIT 1
            ) lm ON TRUE
            WHERE c.uuid = $1
            "#,
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_all_for_user(&self, user_uuid: Uuid, limit: i64, offset: i64) -> Result<Vec<ChatRow>, sqlx::Error> {
        query_as::<_, ChatRow>(
            r#"
            SELECT
                c.uuid, c.name, c.description, c.chat_type, c.created_by,
                c.is_archived, c.created_at, c.updated_at,
                lm.body AS last_message_body,
                lm.created_at AS last_message_at,
                (SELECT COUNT(*) FROM message m WHERE m.chat_uuid = c.uuid
                    AND m.is_deleted = FALSE AND m.sender_uuid <> $1
                    AND NOT EXISTS (
                        SELECT 1 FROM message_status ms
                        WHERE ms.message_uuid = m.uuid AND ms.user_uuid = $1 AND ms.status = 'read'
                    )
                ) AS unread_count,
                (SELECT COUNT(*) FROM chat_member cm2
                    WHERE cm2.chat_uuid = c.uuid AND cm2.left_at IS NULL
                ) AS member_count,

                gu.uuid                        AS sender_uuid,
                gu.first_name                   AS sender_first_name,
                gu.second_name                  AS sender_second_name,
                gu.avatar_uuid                  AS sender_avatar_uuid,
                med_avatar.url                  AS sender_avatar_url,
                gu.last_seen_at                 AS sender_last_seen_at,
                gu.is_online                   AS sender_is_online,
                cm.alias                        AS alias

            FROM chat c
            JOIN chat_member cm ON cm.chat_uuid = c.uuid
                AND cm.user_uuid = $1 AND cm.left_at IS NULL
            LEFT JOIN chat_member cm_other ON cm_other.chat_uuid = c.uuid
                AND cm_other.user_uuid != $1 AND cm_other.left_at IS NULL
            LEFT JOIN guest_user gu ON gu.uuid = cm_other.user_uuid
            LEFT JOIN media med_avatar ON med_avatar.uuid = gu.avatar_uuid
            LEFT JOIN LATERAL (
                SELECT body, created_at FROM message
                WHERE chat_uuid = c.uuid AND is_deleted = FALSE
                ORDER BY created_at DESC LIMIT 1
            ) lm ON TRUE
            ORDER BY COALESCE(lm.created_at, c.created_at) DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_uuid)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    async fn count_for_user(&self, user_uuid: Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT COUNT(DISTINCT c.uuid) FROM chat c
             JOIN chat_member cm ON cm.chat_uuid = c.uuid
                 AND cm.user_uuid = $1 AND cm.left_at IS NULL",
        )
        .bind(user_uuid)
        .fetch_one(&self.pool)
        .await
    }

    async fn create(&self, name: &Option<String>, description: &Option<String>, chat_type: &str, creator_uuid: Uuid, avatar: &Option<Uuid>) -> Result<Uuid, sqlx::Error> {
        sqlx::query_scalar(
            "INSERT INTO chat (name, description, chat_type, created_by, avatar)
             VALUES ($1, $2, $3::chat_type, $4, $5) RETURNING uuid",
        )
        .bind(name)
        .bind(description)
        .bind(chat_type)
        .bind(creator_uuid)
        .bind(avatar)
        .fetch_one(&self.pool)
        .await
    }

    async fn update_metadata(&self, uuid: Uuid, name: Option<String>, description: Option<String>, avatar: Option<Uuid>, is_archived: Option<bool>) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE chat SET 
                name = COALESCE($2, name), 
                description = COALESCE($3, description), 
                avatar = COALESCE($4, avatar),
                is_archived = COALESCE($5, is_archived),
                updated_at = NOW()
             WHERE uuid = $1"
        )
        .bind(uuid)
        .bind(name)
        .bind(description)
        .bind(avatar)
        .bind(is_archived)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn set_alias(&self, chat_uuid: Uuid, user_uuid: Uuid, alias: Option<String>) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE chat_member SET alias = $3 WHERE chat_uuid = $1 AND user_uuid = $2"
        )
        .bind(chat_uuid)
        .bind(user_uuid)
        .bind(alias)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn add_member(&self, chat_uuid: Uuid, user_uuid: Uuid, is_admin: bool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO chat_member (chat_uuid, user_uuid, is_admin)
             VALUES ($1, $2, $3) ON CONFLICT (chat_uuid, user_uuid) 
             DO UPDATE SET left_at = NULL, is_admin = EXCLUDED.is_admin",
        )
        .bind(chat_uuid)
        .bind(user_uuid)
        .bind(is_admin)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_member_uuids(&self, chat_uuid: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
        sqlx::query_scalar::<_, Uuid>(
            "SELECT user_uuid FROM chat_member WHERE chat_uuid = $1 AND left_at IS NULL",
        )
        .bind(chat_uuid)
        .fetch_all(&self.pool)
        .await
    }

    async fn remove_member(&self, chat_uuid: Uuid, user_uuid: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE chat_member SET left_at = NOW() WHERE chat_uuid = $1 AND user_uuid = $2",
        )
        .bind(chat_uuid)
        .bind(user_uuid)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_members(&self, chat_uuid: Uuid) -> Result<Vec<ChatMemberDTO>, sqlx::Error> {
        query_as::<_, ChatMemberDTO>(
            r#"
            SELECT
                cm.uuid, cm.chat_uuid, cm.user_uuid, cm.is_admin, cm.joined_at, cm.left_at,
                u.first_name,
                u.second_name AS last_name,
                u.avatar_uuid AS avatar,
                u.last_seen_at,
                u.is_online
            FROM chat_member cm
            JOIN guest_user u ON u.uuid = cm.user_uuid
            WHERE cm.chat_uuid = $1 AND cm.left_at IS NULL
            ORDER BY cm.is_admin DESC, cm.joined_at
            "#,
        )
        .bind(chat_uuid)
        .fetch_all(&self.pool)
        .await
    }

    async fn is_member(&self, chat_uuid: Uuid, user_uuid: Uuid) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                 SELECT 1 FROM chat_member
                 WHERE chat_uuid = $1 AND user_uuid = $2 AND left_at IS NULL
             )",
        )
        .bind(chat_uuid)
        .bind(user_uuid)
        .fetch_one(&self.pool)
        .await
    }

    async fn is_admin(&self, chat_uuid: Uuid, user_uuid: Uuid) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar::<_, bool>(
            "SELECT COALESCE(
                 (SELECT is_admin FROM chat_member
                  WHERE chat_uuid = $1 AND user_uuid = $2 AND left_at IS NULL),
                 FALSE
             )",
        )
        .bind(chat_uuid)
        .bind(user_uuid)
        .fetch_one(&self.pool)
        .await
    }

    async fn delete(&self, uuid: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM chat WHERE uuid = $1")
            .bind(uuid)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_media_counts(&self, chat_uuid: Uuid) -> Result<crate::messenger::dto::ChatMediaCountsDTO, sqlx::Error> {
        sqlx::query_as::<_, crate::messenger::dto::ChatMediaCountsDTO>(
            r#"
            SELECT 
                COUNT(*) FILTER (WHERE m.media_uuid IS NOT NULL AND med.media_type = 'image') as images,
                COUNT(*) FILTER (WHERE m.media_uuid IS NOT NULL AND med.media_type = 'video') as videos,
                COUNT(*) FILTER (WHERE m.media_uuid IS NOT NULL AND med.media_type = 'audio') as audio
            FROM message m
            JOIN media med ON med.uuid = m.media_uuid
            WHERE m.chat_uuid = $1 AND m.is_deleted = FALSE
            "#
        )
        .bind(chat_uuid)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_direct_chat(&self, user1: Uuid, user2: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
        sqlx::query_scalar(
            r#"
            SELECT c.uuid
            FROM chat c
            WHERE c.chat_type = 'direct'
              AND EXISTS (SELECT 1 FROM chat_member cm1 WHERE cm1.chat_uuid = c.uuid AND cm1.user_uuid = $1 AND cm1.left_at IS NULL)
              AND EXISTS (SELECT 1 FROM chat_member cm2 WHERE cm2.chat_uuid = c.uuid AND cm2.user_uuid = $2 AND cm2.left_at IS NULL)
            LIMIT 1
            "#,
        )
        .bind(user1)
        .bind(user2)
        .fetch_optional(&self.pool)
        .await
    }
}

const MSG_SELECT: &str = r#"
    SELECT
        m.uuid, m.chat_uuid, m.sender_uuid, m.reply_to_uuid, m.body,
        m.is_edited, m.is_deleted, m.is_pinned, m.created_at, m.updated_at,
        gu.first_name  AS sender_first_name,
        gu.second_name AS sender_second_name,
        gu.avatar_uuid      AS sender_avatar_uuid,
        gu.last_seen_at     AS sender_last_seen_at,
        gu.is_online        AS sender_is_online,
        (SELECT COUNT(*) FROM message_status ms1
            WHERE ms1.message_uuid = m.uuid AND ms1.status = 'delivered') AS delivered_count,
        (SELECT COUNT(*) FROM message_status ms2
            WHERE ms2.message_uuid = m.uuid AND ms2.status = 'read') AS read_count,
        (SELECT status FROM message_status ms3
            WHERE ms3.message_uuid = m.uuid AND ms3.user_uuid = $2 LIMIT 1) AS my_status,
        (SELECT body FROM message m2 WHERE m2.uuid = m.reply_to_uuid LIMIT 1) AS reply_body_preview,
        m.media_uuid,
        med.media_type::text,
        med.title      AS media_title,
        med.alt        AS media_alt,
        med.url        AS media_url
    FROM message m
    LEFT JOIN guest_user gu ON gu.uuid = m.sender_uuid
    LEFT JOIN media med ON med.uuid = m.media_uuid
"#;

#[async_trait]
impl MessageRepository for PostgresMessengerRepository {
    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<MessageRow>, sqlx::Error> {
        query_as::<_, MessageRow>(
            &format!("{MSG_SELECT} WHERE m.uuid = $1")
        )
        .bind(uuid)
        .bind(Uuid::nil()) // Placeholder
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_all_in_chat(&self, chat_uuid: Uuid, limit: i64, offset: i64) -> Result<Vec<MessageRow>, sqlx::Error> {
        query_as::<_, MessageRow>(
            &format!(
                "{MSG_SELECT}
                 WHERE m.chat_uuid = $1 AND m.is_deleted = FALSE
                 ORDER BY m.created_at DESC
                 LIMIT $3 OFFSET $4"
            )
        )
        .bind(chat_uuid)
        .bind(Uuid::nil()) // Placeholder for me_uuid if needed, currently repo doesn't take me_uuid for all methods
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_all_before_cursor(&self, chat_uuid: Uuid, before_uuid: Uuid, limit: i64) -> Result<Vec<MessageRow>, sqlx::Error> {
        query_as::<_, MessageRow>(
            &format!(
                "{MSG_SELECT}
                 WHERE m.chat_uuid = $1 AND m.is_deleted = FALSE
                   AND m.created_at < (SELECT created_at FROM message WHERE uuid = $3)
                 ORDER BY m.created_at DESC
                 LIMIT $4"
            )
        )
        .bind(chat_uuid)
        .bind(Uuid::nil()) // Placeholder
        .bind(before_uuid)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    async fn count_all_in_chat(&self, chat_uuid: Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM message WHERE chat_uuid = $1 AND is_deleted = FALSE",
        )
        .bind(chat_uuid)
        .fetch_one(&self.pool)
        .await
    }

    async fn create(&self, chat_uuid: Uuid, sender_uuid: Uuid, reply_to_uuid: Option<Uuid>, body: &str, media_uuid: Option<Uuid>) -> Result<MessageRow, sqlx::Error> {
        sqlx::query_as::<_, MessageRow>(
            r#"WITH ins AS (
                INSERT INTO message (chat_uuid, sender_uuid, reply_to_uuid, body, media_uuid)
                VALUES ($1, $2, $3, $4, $5) RETURNING *
            )
            SELECT
                m.uuid, m.chat_uuid, m.sender_uuid, m.reply_to_uuid,
                m.body, m.is_edited, m.is_deleted, m.is_pinned, m.created_at, m.updated_at,
                gu.first_name  AS sender_first_name,
                gu.second_name AS sender_second_name,
                gu.avatar_uuid      AS sender_avatar_uuid,
                gu.is_online        AS sender_is_online,
                0::bigint      AS delivered_count,
                0::bigint      AS read_count,
                NULL::delivery_status AS my_status,
                reply.body     AS reply_body_preview,
                m.media_uuid,
                med.media_type::text,
                med.title      AS media_title,
                med.alt        AS media_alt,
                med.url        AS media_url
            FROM ins m
            JOIN guest_user gu ON gu.uuid = m.sender_uuid
            LEFT JOIN message reply ON reply.uuid = m.reply_to_uuid
            LEFT JOIN media med ON med.uuid = m.media_uuid"#
        )
        .bind(chat_uuid)
        .bind(sender_uuid)
        .bind(reply_to_uuid)
        .bind(body)
        .bind(media_uuid)
        .fetch_one(&self.pool)
        .await
    }

    async fn update_body(&self, uuid: Uuid, body: &str) -> Result<Option<MessageRow>, sqlx::Error> {
        query_as::<_, MessageRow>(
            r#"WITH upd AS (
                UPDATE message SET body = $1, is_edited = TRUE
                WHERE uuid = $2 AND is_deleted = FALSE
                RETURNING *
            )
            SELECT
                m.uuid, m.chat_uuid, m.sender_uuid, m.reply_to_uuid,
                m.body, m.is_edited, m.is_deleted, m.is_pinned, m.created_at, m.updated_at,
                gu.first_name  AS sender_first_name,
                gu.second_name AS sender_second_name,
                gu.avatar_uuid      AS sender_avatar_uuid,
                gu.is_online        AS sender_is_online,
                (SELECT COUNT(*) FROM message_status ms WHERE ms.message_uuid = m.uuid AND ms.status = 'delivered') AS delivered_count,
                (SELECT COUNT(*) FROM message_status ms WHERE ms.message_uuid = m.uuid AND ms.status = 'read')      AS read_count,
                NULL::delivery_status AS my_status,
                NULL::text            AS reply_body_preview,
                m.media_uuid,
                med.media_type::text,
                med.title      AS media_title,
                med.alt        AS media_alt,
                med.url        AS media_url
            FROM upd m
            JOIN guest_user gu ON gu.uuid = m.sender_uuid
            LEFT JOIN media med ON med.uuid = m.media_uuid"#,
        )
        .bind(body)
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
    }

    async fn delete(&self, uuid: Uuid) -> Result<u64, sqlx::Error> {
        sqlx::query(
            "UPDATE message SET is_deleted = TRUE, body = ''
             WHERE uuid = $1",
        )
        .bind(uuid)
        .execute(&self.pool)
        .await
        .map(|r| r.rows_affected())
    }

    async fn upsert_statuses(&self, chat_uuid: Uuid, user_uuid: Uuid, status: DeliveryStatus) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO message_status (message_uuid, user_uuid, status)
            SELECT m.uuid, $2, $3
            FROM message m
            WHERE m.chat_uuid    = $1
              AND m.sender_uuid <> $2
              AND m.is_deleted   = FALSE
              AND NOT EXISTS (
                  SELECT 1 FROM message_status ms
                  WHERE ms.message_uuid = m.uuid
                    AND ms.user_uuid    = $2
                    AND ms.status       = $3
              )
            ON CONFLICT (message_uuid, user_uuid)
            DO UPDATE SET status = EXCLUDED.status
            "#,
        )
        .bind(chat_uuid)
        .bind(user_uuid)
        .bind(status)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn add_reaction(&self, message_uuid: Uuid, user_uuid: Uuid, emoji: &str) -> Result<bool, sqlx::Error> {
        // Пробуем удалить (toggle off)
        let deleted = sqlx::query(
            "DELETE FROM message_reaction WHERE message_uuid = $1 AND user_uuid = $2 AND emoji = $3"
        )
        .bind(message_uuid)
        .bind(user_uuid)
        .bind(emoji)
        .execute(&self.pool)
        .await?;

        if deleted.rows_affected() > 0 {
            return Ok(false); // Удалено
        }

        // Если не удалено, значит не было — добавляем
        sqlx::query(
            "INSERT INTO message_reaction (message_uuid, user_uuid, emoji) VALUES ($1, $2, $3)"
        )
        .bind(message_uuid)
        .bind(user_uuid)
        .bind(emoji)
        .execute(&self.pool)
        .await?;

        Ok(true) // Добавлено
    }

    async fn get_reactions(&self, message_uuid: Uuid) -> Result<Vec<ReactionDTO>, sqlx::Error> {
        sqlx::query_as::<_, ReactionDTO>(
            "SELECT user_uuid, emoji FROM message_reaction WHERE message_uuid = $1"
        )
        .bind(message_uuid)
        .fetch_all(&self.pool)
        .await
    }

    async fn search(&self, user_uuid: Uuid, chat_uuid: Option<Uuid>, query: &str, limit: i64) -> Result<Vec<MessageRow>, sqlx::Error> {
        let pattern = format!("%{}%", query);
        query_as::<_, MessageRow>(
            &format!(
                "{}
                 WHERE m.is_deleted = FALSE 
                   AND m.body ILIKE $3
                   AND ($4::uuid IS NULL OR m.chat_uuid = $4)
                   AND EXISTS (
                       SELECT 1 FROM chat_member cm 
                       WHERE cm.chat_uuid = m.chat_uuid AND cm.user_uuid = $1 AND cm.left_at IS NULL
                   )
                 ORDER BY m.created_at DESC
                 LIMIT $5",
                MSG_SELECT
            )
        )
        .bind(user_uuid)
        .bind(user_uuid)
        .bind(pattern)
        .bind(chat_uuid)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    async fn get_receipts(&self, message_uuid: Uuid) -> Result<Vec<crate::messenger::dto::MessageReceiptRow>, sqlx::Error> {
        sqlx::query_as::<_, crate::messenger::dto::MessageReceiptRow>(
            r#"
            SELECT 
                ms.user_uuid,
                gu.first_name,
                gu.second_name,
                gu.avatar_uuid,
                ms.status,
                ms.created_at
            FROM message_status ms
            JOIN guest_user gu ON gu.uuid = ms.user_uuid
            WHERE ms.message_uuid = $1
            ORDER BY ms.created_at DESC
            "#
        )
        .bind(message_uuid)
        .fetch_all(&self.pool)
        .await
    }

    async fn update_pin(&self, uuid: Uuid, is_pinned: bool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE message SET is_pinned = $2 WHERE uuid = $1")
            .bind(uuid)
            .bind(is_pinned)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_media(&self, chat_uuid: Uuid, media_type: Option<String>, limit: i64, offset: i64) -> Result<Vec<MessageRow>, sqlx::Error> {
        query_as::<_, MessageRow>(
            &format!(
                "{}
                 WHERE m.chat_uuid = $1 AND m.is_deleted = FALSE
                   AND m.media_uuid IS NOT NULL
                   AND ($3::text IS NULL OR med.media_type = $3::media_type)
                 ORDER BY m.created_at DESC
                 LIMIT $4 OFFSET $5",
                MSG_SELECT
            )
        )
        .bind(chat_uuid)
        .bind(Uuid::nil())
        .bind(media_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }
}
