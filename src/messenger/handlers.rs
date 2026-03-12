use std::{collections::HashMap, sync::Arc};
use chrono::Utc;

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing,
};
use sqlx::{PgPool, query_as, Row};
use uuid::Uuid;

use crate::{AppState, auth::utils::get_user_from_token, core::{dto::{ApiPaginationDTO, ApiResponse, ApiResponseWithPagination, MediaDTO, PaginationDTO, PaginationQuery}, handlers::{get_media_by_uuid, get_media_by_uuids}, response::{error_map, into_api_response, into_api_response_with_pagination}}, messenger::dto::{ChatRow, MessageRow, UserPreviewDTO, WsQuery}};

use super::{
    dto::{
        AddMemberDTO, ChatMemberDTO, ChatResponseDTO, ChatType, CreateChatDTO, CreateMessageDTO,
        DeliveryStatus, MessageQuery, MessageResponseDTO, UpdateMessageDTO, SetAliasDTO,
        ChatMediaCountsDTO,
        WsClientAction, WsServerEvent,
    },
    ws::{WsState, handle_socket, handle_user_socket},
};

// ════════════════════════════════════════════════════════════════
// Helpers
// ════════════════════════════════════════════════════════════════

async fn get_chat_response_dto(
    pool: &PgPool,
    chat_uuid: Uuid,
) -> Result<Json<ApiResponse<ChatResponseDTO>>, (StatusCode, Json<ApiResponse<ChatResponseDTO>>)> {
    let chat_row = query_as::<_, ChatRow>(
        r#"
        SELECT
            c.uuid, c.name, c.description, c.chat_type, c.created_by,
            c.avatar AS avatar_uuid, c.is_archived, c.created_at, c.updated_at,
            NULL::text AS last_message_body,
            NULL::timestamptz AS last_message_at,
            NULL::bigint AS unread_count,
            (SELECT COUNT(*) FROM chat_member cm WHERE cm.chat_uuid = c.uuid AND cm.left_at IS NULL) AS member_count,
            gu.uuid AS sender_uuid,
            gu.first_name AS sender_first_name,
            gu.first_name AS sender_first_name,
            gu.second_name AS sender_second_name,
            gu.avatar_uuid AS sender_avatar_uuid,
            gu.last_seen_at AS sender_last_seen_at
        FROM chat c
        LEFT JOIN guest_user gu ON gu.uuid = c.created_by
        WHERE c.uuid = $1
        "#,
    )
    .bind(chat_uuid)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    match chat_row {
        Some(row) => {
            // Собираем UUID аватарок для пакетной загрузки
            let mut uuids = Vec::new();
            if let Some(u) = row.sender_avatar_uuid { uuids.push(u); }

            let media_map = get_media_by_uuids(pool, uuids).await.unwrap_or_default();
            let dto = map_chat_row(row, &media_map);
            into_api_response(StatusCode::OK, Some(dto), None, None)
        }
        None => {
                   // ✅ Конструируем вручную или используем into_api_response
                   let empty_response = ApiResponse {
                       data: None,
                       messages: None,
                       errors: None,
                   };
                   Err((StatusCode::NOT_FOUND, Json(empty_response)))
               }
    }
}

async fn map_message_row(
    state: &Arc<AppState>,
    row: MessageRow,
) -> MessageResponseDTO {
    // Загружаем аватар через вашу функцию
    let sender_avatar = get_media_by_uuid(state, row.sender_avatar_uuid)
        .await
        .ok()
        .flatten();

    // Загружаем медиа сообщения
    let media = get_media_by_uuid(state, row.media_uuid)
        .await
        .ok()
        .flatten();

    // Загружаем реакции
    let reactions = get_message_reactions(&state.pool, row.uuid).await.unwrap_or_default();

    // Фильтруем пустые строки
    let first_name = row.sender_first_name.filter(|s| !s.is_empty());
    let second_name = row.sender_second_name.filter(|s| !s.is_empty());

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

        // 👇 Вложенный sender
        sender: Some(UserPreviewDTO {
            uuid: row.sender_uuid,
            first_name,
            second_name,
            avatar: sender_avatar,
            is_online: if let Some(ws) = &state.user_ws_state { ws.is_online(row.sender_uuid).await } else { false },
            last_seen_at: Utc::now(), // В MessageRow этого поля пока нет, но для превью пойдёт
        }),

        delivered_count: row.delivered_count,
        read_count: row.read_count,
        my_status: row.my_status,
        reply_body_preview: row.reply_body_preview,
        media,
        reactions,
    }
}

async fn get_message_reactions(pool: &sqlx::PgPool, message_uuid: Uuid) -> sqlx::Result<Vec<super::dto::ReactionDTO>> {
    sqlx::query_as::<_, super::dto::ReactionDTO>(
        "SELECT user_uuid, emoji FROM message_reaction WHERE message_uuid = $1"
    )
    .bind(message_uuid)
    .fetch_all(pool)
    .await
}

async fn handle_react_to_message(pool: &sqlx::PgPool, user_uuid: Uuid, message_uuid: Uuid, emoji: String) -> sqlx::Result<bool> {
    // Пробуем удалить (toggle off)
    let deleted = sqlx::query(
        "DELETE FROM message_reaction WHERE message_uuid = $1 AND user_uuid = $2 AND emoji = $3"
    )
    .bind(message_uuid)
    .bind(user_uuid)
    .bind(&emoji)
    .execute(pool)
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
    .bind(&emoji)
    .execute(pool)
    .await?;

    Ok(true) // Добавлено
}

async fn assert_member(
    pool: &sqlx::PgPool,
    chat_uuid: Uuid,
    user_uuid: Uuid,
) -> bool {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
             SELECT 1 FROM chat_member
             WHERE chat_uuid = $1 AND user_uuid = $2 AND left_at IS NULL
         )",
    )
    .bind(chat_uuid)
    .bind(user_uuid)
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

async fn assert_admin(
    pool: &sqlx::PgPool,
    chat_uuid: Uuid,
    user_uuid: Uuid,
) -> bool {
    sqlx::query_scalar::<_, bool>(
        "SELECT COALESCE(
             (SELECT is_admin FROM chat_member
              WHERE chat_uuid = $1 AND user_uuid = $2 AND left_at IS NULL),
             FALSE
         )",
    )
    .bind(chat_uuid)
    .bind(user_uuid)
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

fn map_chat_row(
    row: ChatRow,
    media_map: &HashMap<Uuid, MediaDTO>,
) -> ChatResponseDTO {
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
            avatar: row.sender_avatar_uuid.and_then(|u| media_map.get(&u).cloned()),
            is_online: false, // Проставим в get_chats или get_chat где есть доступ к ws_state
            last_seen_at: row.sender_last_seen_at.unwrap_or_else(Utc::now),
        }),
        alias: row.alias,
    }
}

// ════════════════════════════════════════════════════════════════
// Chat handlers
// ════════════════════════════════════════════════════════════════

/// Список чатов текущего пользователя
#[utoipa::path(
    get,
    path = "/",
    params(
        ("page"  = Option<i64>, Query, description = "Page number (default 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default 20)"),
    ),
    responses(
        (status = 200, body = ApiResponseWithPagination<ChatResponseDTO>),
        (status = 500, body = ApiResponseWithPagination<ChatResponseDTO>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = [])),
    operation_id = "get_chats",
)]
pub async fn get_chats(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Query(p): Query<PaginationQuery>,
) -> Result<
    Json<ApiResponseWithPagination<ChatResponseDTO>>,
    (StatusCode, Json<ApiResponseWithPagination<ChatResponseDTO>>),
> {
    let page = p.page.unwrap_or(1);
    let limit = p.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    // 🔹 Подсчёт общего количества
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT c.uuid) FROM chat c
         JOIN chat_member cm ON cm.chat_uuid = c.uuid
             AND cm.user_uuid = $1 AND cm.left_at IS NULL",
    )
    .bind(me)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    // 🔹 Загружаем плоские строки
    let rows = query_as::<_, ChatRow>(
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

            -- 👇 Собеседник
            cm_other.user_uuid              AS sender_uuid,
            gu.first_name                   AS sender_first_name,
            gu.second_name                  AS sender_second_name,
            gu.avatar_uuid                  AS sender_avatar_uuid,
            cm.alias                        AS alias

        FROM chat c
        JOIN chat_member cm ON cm.chat_uuid = c.uuid
            AND cm.user_uuid = $1 AND cm.left_at IS NULL
        LEFT JOIN chat_member cm_other ON cm_other.chat_uuid = c.uuid
            AND cm_other.user_uuid != $1 AND cm_other.left_at IS NULL
        LEFT JOIN guest_user gu ON gu.uuid = cm_other.user_uuid
        LEFT JOIN LATERAL (
            SELECT body, created_at FROM message
            WHERE chat_uuid = c.uuid AND is_deleted = FALSE
            ORDER BY created_at DESC LIMIT 1
        ) lm ON TRUE
        ORDER BY COALESCE(lm.created_at, c.created_at) DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(me)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.pool)
    .await;

    let rows = match rows {
        Ok(r) => r,
        Err(_e) => {
            let msg = state.i18n.t("general.db_error", &locale).await;
            return into_api_response_with_pagination(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                Some(error_map("database", &msg)),
                Some(vec![msg]),
            );
        }
    };

    // 🔹 Конвертируем в финальные DTO
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        // Загружаем аватар собеседника через вашу функцию
        let sender_avatar = get_media_by_uuid(&state, row.sender_avatar_uuid)
            .await
            .ok()
            .flatten();

        let is_online_val = if let (Some(ws), Some(uuid)) = (&state.user_ws_state, row.sender_uuid) {
            ws.is_online(uuid).await
        } else {
            false
        };

        items.push(ChatResponseDTO {
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
                avatar: sender_avatar,
                is_online: is_online_val,
                last_seen_at: row.sender_last_seen_at.unwrap_or_else(Utc::now),
            }),
            alias: row.alias,
        });
    }

    into_api_response_with_pagination(
        StatusCode::OK,
        Some(ApiPaginationDTO {
            items,
            pagination: PaginationDTO {
                page: page as i32,
                total: Some(total as i32),
                total_pages: Some((total as f64 / limit as f64).ceil() as i32),
                limit: Some(limit as i32),
            },
        }),
        None,
        None,
    )
}

/// Один чат по UUID
#[utoipa::path(
    get,
    path = "/{chat_uuid}",
    params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
    responses(
        (status = 200, body = ApiResponse<ChatResponseDTO>),
        (status = 403, body = ApiResponse<ChatResponseDTO>),
        (status = 404, body = ApiResponse<ChatResponseDTO>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = [])),
    operation_id = "get_chat",
)]
pub async fn get_chat(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Path(chat_uuid): Path<Uuid>,
) -> Result<Json<ApiResponse<ChatResponseDTO>>, (StatusCode, Json<ApiResponse<ChatResponseDTO>>)> {
    if !assert_member(&state.pool, chat_uuid, me).await {
        let msg = state.i18n.t("messenger.not_member", &locale).await;
        return into_api_response(StatusCode::FORBIDDEN, None, None, Some(vec![msg]));
    }

    let row = query_as::<_, ChatRow>(
        r#"
        SELECT
            c.uuid, c.name, c.description, c.chat_type, c.created_by,
            c.is_archived, c.created_at, c.updated_at,
            NULL::text AS last_message_body,
            NULL::timestamptz AS last_message_at,
            (SELECT COUNT(*) FROM message m WHERE m.chat_uuid = c.uuid
                AND m.is_deleted = FALSE AND m.sender_uuid <> $2
                AND NOT EXISTS (
                    SELECT 1 FROM message_status ms
                    WHERE ms.message_uuid = m.uuid AND ms.user_uuid = $2 AND ms.status = 'read'
                )
            ) AS unread_count,
            (SELECT COUNT(*) FROM chat_member cm WHERE cm.chat_uuid = $1 AND cm.left_at IS NULL) AS member_count,

            -- 👇 Собеседник
            cm_other.user_uuid           AS sender_uuid,
            gu.first_name                AS sender_first_name,
            gu.second_name               AS sender_second_name,
            gu.avatar_uuid               AS sender_avatar_uuid,
            gu.last_seen_at              AS sender_last_seen_at,
            cm.alias                     AS alias
        FROM chat c
        JOIN chat_member cm ON cm.chat_uuid = c.uuid
            AND cm.user_uuid = $2
            AND cm.left_at IS NULL
        LEFT JOIN chat_member cm_other ON cm_other.chat_uuid = c.uuid
            AND cm_other.user_uuid != $2
            AND cm_other.left_at IS NULL
        LEFT JOIN guest_user gu ON gu.uuid = cm_other.user_uuid
        WHERE c.uuid = $1
        "#,
    )
    .bind(chat_uuid)
    .bind(me)
    .fetch_optional(&state.pool)
    .await;

    match row {
      Ok(Some(row)) => {
           let sender_avatar = get_media_by_uuid(&state, row.sender_avatar_uuid)
               .await
               .ok()
               .flatten();

           let is_online_val = if let (Some(ws), Some(uuid)) = (&state.user_ws_state, row.sender_uuid) {
               ws.is_online(uuid).await
           } else {
               false
           };

           let dto = ChatResponseDTO {
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
                   avatar: sender_avatar,
                   is_online: is_online_val,
                   last_seen_at: row.sender_last_seen_at.unwrap_or_else(Utc::now),
               }),
               alias: row.alias,
           };

           into_api_response(StatusCode::OK, Some(dto), None, None)
       }
        Ok(None) => {
            let msg = state.i18n.t("messenger.chat_not_found", &locale).await;
            into_api_response(StatusCode::NOT_FOUND, None, None, Some(vec![msg]))
        }
        Err(_) => {
            let msg = state.i18n.t("general.db_error", &locale).await;
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, Some(error_map("database", &msg)), Some(vec![msg]))
        }
    }
}

/// Создать чат (DM или группу)
#[utoipa::path(
    post,
    path = "/",
    request_body = CreateChatDTO,
    responses(
        (status = 201, body = ApiResponse<ChatResponseDTO>),
        (status = 400, body = ApiResponse<ChatResponseDTO>),
        (status = 500, body = ApiResponse<ChatResponseDTO>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = [])),
    operation_id = "create_chat",
)]
pub async fn create_chat(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Json(body): Json<CreateChatDTO>,
) -> Result<Json<ApiResponse<ChatResponseDTO>>, (StatusCode, Json<ApiResponse<ChatResponseDTO>>)> {
    // ───────── Валидация ─────────
    if body.chat_type == ChatType::Direct && body.member_uuids.len() != 1 {
        let msg = state.i18n.t("messenger.direct_needs_one_member", &locale).await;
        return into_api_response(StatusCode::BAD_REQUEST, None, Some(error_map("members", &msg)), Some(vec![msg]));
    }
    if body.chat_type == ChatType::Group && body.name.as_deref().unwrap_or("").trim().is_empty() {
        let msg = state.i18n.t("messenger.group_needs_name", &locale).await;
        return into_api_response(StatusCode::BAD_REQUEST, None, Some(error_map("name", &msg)), Some(vec![msg]));
    }

    let mut tx = match state.pool.begin().await {
        Ok(t) => t,
        Err(_) => {
            let msg = state.i18n.t("general.db_error", &locale).await;
            return into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, Some(error_map("database", &msg)), Some(vec![msg]));
        }
    };

    // ───────── Для DM — вернуть уже существующий чат ─────────
    if body.chat_type == ChatType::Direct {
        let other = body.member_uuids[0];
        let existing: Option<Uuid> = sqlx::query_scalar(
            r#"
            SELECT c.uuid FROM chat c
            JOIN chat_member a ON a.chat_uuid = c.uuid AND a.user_uuid = $1 AND a.left_at IS NULL
            JOIN chat_member b ON b.chat_uuid = c.uuid AND b.user_uuid = $2 AND b.left_at IS NULL
            WHERE c.chat_type = 'direct'
            LIMIT 1
            "#,
        )
        .bind(me)
        .bind(other)
        .fetch_optional(&mut *tx)
        .await
        .unwrap_or(None);

        if let Some(uuid) = existing {
            // 👇 rollback() потребляет tx, поэтому сразу возвращаем
            let _ = tx.rollback().await;
            return get_chat_response_dto(&state.pool, uuid).await;
        }
    }

    // ───────── Создать чат ─────────
    let chat_uuid: Uuid = match sqlx::query_scalar(
        "INSERT INTO chat (name, description, chat_type, created_by, avatar)
         VALUES ($1, $2, $3, $4, $5) RETURNING uuid",
    )
    .bind(&body.name)
    .bind(&body.description)
    .bind(&body.chat_type)
    .bind(me)
    .bind(&body.avatar)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(id) => id,
        Err(_) => {
            let _ = tx.rollback().await; // tx потребляется здесь
            let msg = state.i18n.t("general.db_error", &locale).await;
            return into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, Some(error_map("database", &msg)), Some(vec![msg]));
        }
    };

    // ───────── Добавить участников ─────────
    let mut members = vec![me];
    for &uid in &body.member_uuids {
        if uid != me { members.push(uid); }
    }

    for &uid in &members {
        if sqlx::query(
            "INSERT INTO chat_member (chat_uuid, user_uuid, is_admin)
             VALUES ($1, $2, $3) ON CONFLICT (chat_uuid, user_uuid) DO NOTHING",
        )
        .bind(chat_uuid)
        .bind(uid)
        .bind(uid == me)
        .execute(&mut *tx)
        .await
        .is_err()
        {
            let _ = tx.rollback().await; // tx потребляется здесь
            let msg = state.i18n.t("general.db_error", &locale).await;
            return into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, Some(error_map("database", &msg)), Some(vec![msg]));
        }
    }

    // ───────── Коммит транзакции ─────────
    if let Err(_) = tx.commit().await { // tx потребляется здесь
        let msg = state.i18n.t("general.db_error", &locale).await;
        return into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, Some(error_map("database", &msg)), Some(vec![msg]));
    }

    // 👇 После коммита tx больше не существует — используем state.pool
    get_chat_response_dto(&state.pool, chat_uuid).await
}

// /// Обновить групповой чат
// #[utoipa::path(
//     put,
//     path = "/{chat_uuid}",
//     params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
//     request_body = UpdateChatDTO,
//     responses(
//         (status = 200, body = ApiResponse<ChatResponseDTO>),
//         (status = 403, body = ApiResponse<ChatResponseDTO>),
//         (status = 404, body = ApiResponse<ChatResponseDTO>),
//     ),
//     tag = "Messenger",
//     security(("bearer_auth" = [])),
//     operation_id = "update_chat",
// )]
// pub async fn update_chat(
//     State(state): State<Arc<AppState>>,
//     Extension(locale): Extension<String>,
//     Extension(me): Extension<Uuid>,
//     Path(chat_uuid): Path<Uuid>,
//     Json(body): Json<UpdateChatDTO>,
// ) -> Result<Json<ApiResponse<ChatResponseDTO>>, (StatusCode, Json<ApiResponse<ChatResponseDTO>>)> {
//     if !assert_admin(&state.pool, chat_uuid, me).await {
//         let msg = state.i18n.t("messenger.not_admin", &locale).await;
//         return into_api_response(StatusCode::FORBIDDEN, None, None, Some(vec![msg]));
//     }

//     let updated = query_as::<_, ChatResponseDTO>(
//         r#"
//         UPDATE chat
//         SET name        = COALESCE($2, name),
//             description = COALESCE($3, description),
//             avatar      = COALESCE($4, avatar),
//             is_archived = COALESCE($5, is_archived)
//         WHERE uuid = $1
//         RETURNING
//             uuid, name, description, chat_type, created_by, avatar, is_archived,
//             created_at, updated_at,
//             NULL::text        AS last_message_body,
//             NULL::timestamptz AS last_message_at,
//             NULL::bigint      AS unread_count,
//             NULL::bigint      AS member_count
//         "#,
//     )
//     .bind(chat_uuid)
//     .bind(&body.name)
//     .bind(&body.description)
//     .bind(&body.avatar)
//     .bind(body.is_archived)
//     .fetch_optional(&state.pool)
//     .await;

//     match updated {
//         Ok(Some(c)) => into_api_response(StatusCode::OK, Some(c), None, None),
//         Ok(None) => {
//             let msg = state.i18n.t("messenger.chat_not_found", &locale).await;
//             into_api_response(StatusCode::NOT_FOUND, None, None, Some(vec![msg]))
//         }
//         Err(_) => {
//             let msg = state.i18n.t("general.db_error", &locale).await;
//             into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, Some(error_map("database", &msg)), Some(vec![msg]))
//         }
//     }
// }

pub async fn leave_or_delete_chat(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Path(chat_uuid): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    if !assert_member(&state.pool, chat_uuid, me).await {
        let msg = state.i18n.t("messenger.not_member", &locale).await;
        return into_api_response(StatusCode::FORBIDDEN, None, None, Some(vec![msg]));
    }

    // Если создатель — удаляем чат целиком (CASCADE)
    let is_sender: bool = sqlx::query_scalar("SELECT created_by = $2 FROM chat WHERE uuid = $1")
        .bind(chat_uuid)
        .bind(me)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(false);

    if is_sender {
        let _ = sqlx::query("DELETE FROM chat WHERE uuid = $1").bind(chat_uuid).execute(&state.pool).await;
    } else {
        let _ = sqlx::query(
            "UPDATE chat_member SET left_at = NOW() WHERE chat_uuid = $1 AND user_uuid = $2",
        )
        .bind(chat_uuid)
        .bind(me)
        .execute(&state.pool)
        .await;
    }

    into_api_response::<()>(StatusCode::OK, None, None, None)
}

// ════════════════════════════════════════════════════════════════
// Member handlers
// ════════════════════════════════════════════════════════════════

/// Список участников
#[utoipa::path(
    get,
    path = "/{chat_uuid}/members",
    params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
    responses(
        (status = 200, body = ApiResponse<Vec<ChatMemberDTO>>),
        (status = 403, body = ApiResponse<Vec<ChatMemberDTO>>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = [])),
    operation_id = "get_members",
)]
pub async fn get_members(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Path(chat_uuid): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<ChatMemberDTO>>>, (StatusCode, Json<ApiResponse<Vec<ChatMemberDTO>>>)> {
    if !assert_member(&state.pool, chat_uuid, me).await {
        let msg = state.i18n.t("messenger.not_member", &locale).await;
        return into_api_response(StatusCode::FORBIDDEN, None, None, Some(vec![msg]));
    }

    let rows = query_as::<_, ChatMemberDTO>(
        r#"
        SELECT
            cm.uuid, cm.chat_uuid, cm.user_uuid, cm.is_admin, cm.joined_at, cm.left_at,
            u.first_name,
            u.last_name,
            u.avatar,
            u.last_seen_at
        FROM chat_member cm
        JOIN guest_user u ON u.uuid = cm.user_uuid
        WHERE cm.chat_uuid = $1 AND cm.left_at IS NULL
        ORDER BY cm.is_admin DESC, cm.joined_at
        "#,
    )
    .bind(chat_uuid)
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(rows) => {
            let mut items = Vec::with_capacity(rows.len());
            for row in rows {
                let is_online = if let Some(ws) = &state.user_ws_state {
                    ws.is_online(row.user_uuid).await
                } else {
                    false
                };
                items.push(ChatMemberDTO {
                    uuid: row.uuid,
                    chat_uuid: row.chat_uuid,
                    user_uuid: row.user_uuid,
                    is_admin: row.is_admin,
                    joined_at: row.joined_at,
                    left_at: row.left_at,
                    first_name: row.first_name,
                    last_name: row.last_name,
                    avatar: row.avatar,
                    is_online,
                    last_seen_at: row.last_seen_at,
                });
            }
            into_api_response(StatusCode::OK, Some(items), None, None)
        }
        Err(_) => {
            let msg = state.i18n.t("general.db_error", &locale).await;
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, Some(error_map("database", &msg)), Some(vec![msg]))
        }
    }
}

pub async fn add_member(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Path(chat_uuid): Path<Uuid>,
    Json(body): Json<AddMemberDTO>,
) -> Result<Json<ApiResponse<ChatMemberDTO>>, (StatusCode, Json<ApiResponse<ChatMemberDTO>>)> {
    if !assert_admin(&state.pool, chat_uuid, me).await {
        let msg = state.i18n.t("messenger.not_admin", &locale).await;
        return into_api_response(StatusCode::FORBIDDEN, None, None, Some(vec![msg]));
    }

    let row = query_as::<_, ChatMemberDTO>(
        r#"
        INSERT INTO chat_member (chat_uuid, user_uuid, is_admin)
        VALUES ($1, $2, $3)
        ON CONFLICT (chat_uuid, user_uuid)
        DO UPDATE SET left_at = NULL, is_admin = EXCLUDED.is_admin
        RETURNING
            uuid, chat_uuid, user_uuid, is_admin, joined_at, left_at,
            NULL::text AS first_name,
            NULL::text AS last_name,
            NULL::text AS avatar
        "#,
    )
    .bind(chat_uuid)
    .bind(body.user_uuid)
    .bind(body.is_admin)
    .fetch_one(&state.pool)
    .await;

    match row {
        Ok(m) => {
            if let Some(ws) = &state.ws_state {
                let _ = ws.broadcast(chat_uuid, WsServerEvent::MemberJoined(m.clone()));
            }
            into_api_response(StatusCode::CREATED, Some(m), None, None)
        }
        Err(_) => {
            let msg = state.i18n.t("general.db_error", &locale).await;
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, Some(error_map("database", &msg)), Some(vec![msg]))
        }
    }
}

pub async fn remove_member(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Path((chat_uuid, target)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    if target != me && !assert_admin(&state.pool, chat_uuid, me).await {
        let msg = state.i18n.t("messenger.not_admin", &locale).await;
        return into_api_response(StatusCode::FORBIDDEN, None, None, Some(vec![msg]));
    }

    let _ = sqlx::query(
        "UPDATE chat_member SET left_at = NOW() WHERE chat_uuid = $1 AND user_uuid = $2",
    )
    .bind(chat_uuid)
    .bind(target)
    .execute(&state.pool)
    .await;

    if let Some(ws) = &state.ws_state {
        let _ = ws.broadcast(chat_uuid, WsServerEvent::MemberLeft { chat_uuid, user_uuid: target });
    }

    into_api_response::<()>(StatusCode::OK, None, None, None)
}

// ════════════════════════════════════════════════════════════════
// Message handlers
// ════════════════════════════════════════════════════════════════

const MSG_SELECT: &str = r#"
    SELECT
        m.uuid, m.chat_uuid, m.sender_uuid, m.reply_to_uuid, m.body,
        m.is_edited, m.is_deleted, m.is_pinned, m.created_at, m.updated_at,
        gu.first_name  AS sender_first_name,
        gu.second_name AS sender_second_name,
        gu.avatar_uuid      AS sender_avatar_uuid,
        gu.last_seen_at     AS sender_last_seen_at,
        (SELECT COUNT(*) FROM message_status ms1
            WHERE ms1.message_uuid = m.uuid AND ms1.status = 'delivered') AS delivered_count,
        (SELECT COUNT(*) FROM message_status ms2
            WHERE ms2.message_uuid = m.uuid AND ms2.status = 'read') AS read_count,
        (SELECT status FROM message_status ms3
            WHERE ms3.message_uuid = m.uuid AND ms3.user_uuid = $2 LIMIT 1) AS my_status,
        (SELECT body FROM message m2 WHERE m2.uuid = m.reply_to_uuid LIMIT 1) AS reply_body_preview,
        m.media_uuid
    FROM message m
    LEFT JOIN guest_user gu ON gu.uuid = m.sender_uuid
"#;

/// История сообщений (offset или курсорная пагинация)
#[utoipa::path(
    get,
    path = "/{chat_uuid}/messages",
    params(
        ("chat_uuid"   = Uuid,         Path,  description = "Chat UUID"),
        ("page"        = Option<i64>,  Query, description = "Page (offset pagination)"),
        ("limit"       = Option<i64>,  Query, description = "Limit per page (max 100)"),
        ("before_uuid" = Option<Uuid>, Query, description = "Cursor: load messages older than this UUID"),
    ),
    responses(
        (status = 200, body = ApiResponseWithPagination<MessageResponseDTO>),
        (status = 403, body = ApiResponseWithPagination<MessageResponseDTO>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = [])),
    operation_id = "get_messages",
)]
pub async fn get_messages(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Path(chat_uuid): Path<Uuid>,
    Query(q): Query<MessageQuery>,
) -> Result<
    Json<ApiResponseWithPagination<MessageResponseDTO>>,
    (StatusCode, Json<ApiResponseWithPagination<MessageResponseDTO>>),
> {
    if !assert_member(&state.pool, chat_uuid, me).await {
        let msg = state.i18n.t("messenger.not_member", &locale).await;
        return into_api_response_with_pagination(StatusCode::FORBIDDEN, None, None, Some(vec![msg]));
    }

    let limit  = q.limit.unwrap_or(30).min(100);
    let page   = q.page.unwrap_or(1);
    let offset = (page - 1) * limit;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM message WHERE chat_uuid = $1 AND is_deleted = FALSE",
    )
    .bind(chat_uuid)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    // 👇 Загружаем плоские строки (MessageRow)
    let rows = if let Some(before) = q.before_uuid {
        query_as::<_, MessageRow>(&format!(
            "{MSG_SELECT}
             WHERE m.chat_uuid = $1 AND m.is_deleted = FALSE
               AND m.created_at < (SELECT created_at FROM message WHERE uuid = $3)
             ORDER BY m.created_at DESC
             LIMIT $4"
        ))
        .bind(chat_uuid)
        .bind(me)
        .bind(before)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
    } else {
        query_as::<_, MessageRow>(&format!(
            "{MSG_SELECT}
             WHERE m.chat_uuid = $1 AND m.is_deleted = FALSE
             ORDER BY m.created_at DESC
             LIMIT $3 OFFSET $4"
        ))
        .bind(chat_uuid)
        .bind(me)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    };

    let rows = match rows {
        Ok(r) => r,
        Err(e) => {
          println!("error: {:?}", e);
            let msg = state.i18n.t("general.db_error", &locale).await;
            return into_api_response_with_pagination(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                Some(error_map("database", &msg)),
                Some(vec![msg]),
            );
        }
    };

    // 👇 Конвертируем MessageRow → MessageResponseDTO
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(map_message_row(&state, row).await);
    }

    items.reverse();

    into_api_response_with_pagination(
        StatusCode::OK,
        Some(ApiPaginationDTO {
            items,
            pagination: PaginationDTO {
                page: page as i32,
                total: Some(total as i32),
                total_pages: Some((total as f64 / limit as f64).ceil() as i32),
                limit: Some(limit as i32),
            },
        }),
        None,
        None,
    )
}

/// Отправить сообщение (REST fallback; основной путь — WebSocket)
#[utoipa::path(
    post,
    path = "/{chat_uuid}/messages",
    params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
    request_body = CreateMessageDTO,
    responses(
        (status = 201, body = ApiResponse<MessageResponseDTO>),
        (status = 403, body = ApiResponse<MessageResponseDTO>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = [])),
    operation_id = "send_message",
)]
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Path(chat_uuid): Path<Uuid>,
    Json(body): Json<CreateMessageDTO>,
) -> Result<Json<ApiResponse<MessageResponseDTO>>, (StatusCode, Json<ApiResponse<MessageResponseDTO>>)> {
    if !assert_member(&state.pool, chat_uuid, me).await {
        let msg = state.i18n.t("messenger.not_member", &locale).await;
        return into_api_response(StatusCode::FORBIDDEN, None, None, Some(vec![msg]));
    }

    // 👇 Загружаем как MessageRow (плоская структура с #[derive(FromRow)])
    let row = sqlx::query_as::<_, MessageRow>(
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
            0::bigint      AS delivered_count,
            0::bigint      AS read_count,
            NULL::delivery_status AS my_status,
            reply.body     AS reply_body_preview,
            m.media_uuid
        FROM ins m
        JOIN guest_user gu ON gu.uuid = m.sender_uuid
        LEFT JOIN message reply ON reply.uuid = m.reply_to_uuid"#
    )
    .bind(chat_uuid)
    .bind(me)
    .bind(body.reply_to_uuid)
    .bind(&body.body)
    .bind(body.media_uuid)
    .fetch_one(&state.pool)
    .await;

    match row {
        Ok(row) => {
            // 👇 Маппим в финальный DTO (загружаем аватар через вашу функцию)
            let msg = map_message_row(&state, row).await;

            // Broadcast по WebSocket (глобальный канал)
            if let Some(user_ws) = &state.user_ws_state {
                let members = get_chat_member_uuids(&state.pool, chat_uuid).await;
                user_ws.broadcast_to_users(&members, WsServerEvent::NewMessage(msg.clone())).await;
            }

            // Legacy broadcast (для совместимости)
            if let Some(ws) = &state.ws_state {
                let _ = ws.broadcast(chat_uuid, WsServerEvent::NewMessage(msg.clone()));
            }

            into_api_response(StatusCode::CREATED, Some(msg), None, None)
        }
        Err(_) => {
            let msg = state.i18n.t("general.db_error", &locale).await;
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, Some(error_map("database", &msg)), Some(vec![msg]))
        }
    }
}

pub async fn edit_message(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Path((chat_uuid, msg_uuid)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateMessageDTO>,
) -> Result<Json<ApiResponse<MessageResponseDTO>>, (StatusCode, Json<ApiResponse<MessageResponseDTO>>)> {

    // 👇 Загружаем как MessageRow (плоская структура)
    let row = sqlx::query_as::<_, MessageRow>(
        r#"WITH upd AS (
            UPDATE message SET body = $1, is_edited = TRUE
            WHERE uuid = $2 AND sender_uuid = $3 AND chat_uuid = $4 AND is_deleted = FALSE
            RETURNING *
        )
        SELECT
            m.uuid, m.chat_uuid, m.sender_uuid, m.reply_to_uuid,
            m.body, m.is_edited, m.is_deleted, m.is_pinned, m.created_at, m.updated_at,
            gu.first_name  AS sender_first_name,
            gu.second_name AS sender_second_name,
            gu.avatar_uuid      AS sender_avatar_uuid,
            (SELECT COUNT(*) FROM message_status ms WHERE ms.message_uuid = m.uuid AND ms.status = 'delivered') AS delivered_count,
            (SELECT COUNT(*) FROM message_status ms WHERE ms.message_uuid = m.uuid AND ms.status = 'read')      AS read_count,
            NULL::delivery_status AS my_status,
            NULL::text            AS reply_body_preview,
            m.media_uuid
        FROM upd m
        JOIN guest_user gu ON gu.uuid = m.sender_uuid"#,
    )
    .bind(&body.body)
    .bind(msg_uuid)
    .bind(me)
    .bind(chat_uuid)
    .fetch_optional(&state.pool)
    .await;

    match row {
        Ok(Some(row)) => {
            // 👇 Маппим в финальный DTO
            let msg = map_message_row(&state, row).await;

            // Broadcast по WebSocket (глобальный)
            if let Some(user_ws) = &state.user_ws_state {
                let members = get_chat_member_uuids(&state.pool, chat_uuid).await;
                user_ws.broadcast_to_users(&members, WsServerEvent::MessageEdited(msg.clone())).await;
            }

            // Legacy broadcast
            if let Some(ws) = &state.ws_state {
                let _ = ws.broadcast(chat_uuid, WsServerEvent::MessageEdited(msg.clone()));
            }

            into_api_response(StatusCode::OK, Some(msg), None, None)
        }
        Ok(None) => {
            let msg = state.i18n.t("messenger.message_not_found_or_forbidden", &locale).await;
            into_api_response(StatusCode::NOT_FOUND, None, None, Some(vec![msg]))
        }
        Err(_) => {
            let msg = state.i18n.t("general.db_error", &locale).await;
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, Some(error_map("database", &msg)), Some(vec![msg]))
        }
    }
}

pub async fn delete_message(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Path((chat_uuid, msg_uuid)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let affected = sqlx::query(
        "UPDATE message SET is_deleted = TRUE, body = ''
         WHERE uuid = $1 AND sender_uuid = $2 AND chat_uuid = $3",
    )
    .bind(msg_uuid)
    .bind(me)
    .bind(chat_uuid)
    .execute(&state.pool)
    .await
    .map(|r| r.rows_affected())
    .unwrap_or(0);

    if affected == 0 {
        let msg = state.i18n.t("messenger.message_not_found_or_forbidden", &locale).await;
        return into_api_response::<()>(StatusCode::NOT_FOUND, None, None, Some(vec![msg]));
    }

    if let Some(user_ws) = &state.user_ws_state {
        let members = get_chat_member_uuids(&state.pool, chat_uuid).await;
        user_ws.broadcast_to_users(&members, WsServerEvent::MessageDeleted { uuid: msg_uuid, chat_uuid }).await;
    }

    if let Some(ws) = &state.ws_state {
        let _ = ws.broadcast(chat_uuid, WsServerEvent::MessageDeleted { uuid: msg_uuid, chat_uuid });
    }
    into_api_response::<()>(StatusCode::OK, None, None, None)
}

pub async fn mark_delivered(
    State(state): State<Arc<AppState>>,
    Extension(_locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Path(chat_uuid): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    upsert_statuses(&state.pool, chat_uuid, me, DeliveryStatus::Delivered).await;

    if let Some(ws) = &state.ws_state {
        let _ = ws.broadcast(chat_uuid, WsServerEvent::StatusUpdated {
            chat_uuid,
            message_uuid: Uuid::nil(),
            user_uuid: me,
            status: DeliveryStatus::Delivered,
        });
    }
    into_api_response::<()>(StatusCode::OK, None, None, None)
}

pub async fn mark_read(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Path(chat_uuid): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    if !assert_member(&state.pool, chat_uuid, me).await {
        let msg = state.i18n.t("messenger.not_member", &locale).await;
        return into_api_response(StatusCode::FORBIDDEN, None, None, Some(vec![msg]));
    }

    upsert_statuses(&state.pool, chat_uuid, me, DeliveryStatus::Read).await;

    if let Some(ws) = &state.ws_state {
        let _ = ws.broadcast(chat_uuid, WsServerEvent::StatusUpdated {
            chat_uuid,
            message_uuid: Uuid::nil(),
            user_uuid: me,
            status: DeliveryStatus::Read,
        });
    }
    into_api_response::<()>(StatusCode::OK, None, None, None)
}

async fn upsert_statuses(
    pool: &sqlx::PgPool,
    chat_uuid: Uuid,
    user_uuid: Uuid,
    status: DeliveryStatus,
) {
    let _ = sqlx::query(
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
    .execute(pool)
    .await;
}

// ════════════════════════════════════════════════════════════════
// WebSocket handler
// ════════════════════════════════════════════════════════════════

/// WebSocket upgrade для real-time чата
#[utoipa::path(
    get,
    path = "/{chat_uuid}",
    params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
    responses(
        (status = 101, description = "Switching protocols — WebSocket connection established"),
        (status = 403, description = "Not a member of this chat"),
    ),
    tag = "Messenger",
    security(("bearer_auth" = [])),
    operation_id = "ws_chat",
)]
pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(chat_uuid): Path<Uuid>,      // Из пути: /ws/chats/:chat_id
    Query(query): Query<WsQuery>,     // Из query: ?token=...
) -> impl IntoResponse {
    let pool     = state.pool.clone();
    let ws_state = state.ws_state.clone().unwrap_or_else(WsState::new);

    let user_uuid = match get_user_from_token(&query.token) {
         Ok(id) => id,
         Err(_) => {
             return StatusCode::UNAUTHORIZED.into_response();
         }
     };

    ws.on_upgrade(move |socket| {
      // 👇 Клонируем Arc перед использованием во вложенных замыканиях
      let state_inner = state.clone();
      let pool_inner = pool.clone();

        handle_socket(socket, chat_uuid, user_uuid, ws_state, move |_chat_uuid, user_uuid, raw| {
            let state_async = state_inner.clone();
            let pool_async = pool_inner.clone();

            let pool = pool.clone();
            async move {
                let action: WsClientAction = match serde_json::from_str(&raw) {
                    Ok(a) => a,
                    Err(_) => {
                        return Some(WsServerEvent::Error {
                            message: "Invalid JSON. Expected {\"action\":\"...\",\"payload\":{...}}".into(),
                        });
                    }
                };

                match action {
                    // ── Отправить сообщение ──────────────────────────────────
                    WsClientAction::SendMessage { body, reply_to_uuid, chat_uuid: msg_chat_uuid, media_uuid } => {
                        let row = sqlx::query_as::<_, MessageRow>(
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
                                0::bigint      AS delivered_count,
                                0::bigint      AS read_count,
                                NULL::delivery_status AS my_status,
                                reply.body     AS reply_body_preview,
                                m.media_uuid
                            FROM ins m
                            JOIN guest_user gu ON gu.uuid = m.sender_uuid
                            LEFT JOIN message reply ON reply.uuid = m.reply_to_uuid"#,
                        )
                        .bind(msg_chat_uuid)
                        .bind(user_uuid)
                        .bind(reply_to_uuid)
                        .bind(&body)
                        .bind(media_uuid)
                        .fetch_one(&pool)
                        .await
                        .ok()?;

                        let msg = map_message_row(&state_async, row).await;
                        
                        // Broadcast to other members
                        if let Some(user_ws) = &state_async.user_ws_state {
                            let members = get_chat_member_uuids(&pool_async, msg_chat_uuid).await;
                            user_ws.broadcast_to_users(&members, WsServerEvent::NewMessage(msg.clone())).await;
                        }

                        Some(WsServerEvent::NewMessage(msg))
                    }

                    WsClientAction::Ping => Some(WsServerEvent::Pong),

                    // ── Реакция на сообщение ──────────────────────────────────
                    WsClientAction::ReactToMessage { chat_uuid: msg_chat_uuid, message_uuid, emoji } => {
                        let is_added = handle_react_to_message(&pool_async, user_uuid, message_uuid, emoji.clone()).await.ok()?;
                        let event = WsServerEvent::MessageReactionUpdated { 
                            chat_uuid: msg_chat_uuid, 
                            message_uuid, 
                            user_uuid, 
                            emoji, 
                            is_added 
                        };
                        if let Some(user_ws) = &state_async.user_ws_state {
                            let members = get_chat_member_uuids(&pool_async, msg_chat_uuid).await;
                            user_ws.broadcast_to_users(&members, event.clone()).await;
                        }

                        Some(event)
                    }

                    // ── Редактировать сообщение ──────────────────────────────
                    WsClientAction::EditMessage { uuid, body, chat_uuid: msg_chat_uuid } => {
                        let row = sqlx::query_as::<_, MessageRow>(
                            r#"WITH upd AS (
                                UPDATE message SET body = $1, is_edited = TRUE
                                WHERE uuid = $2 AND sender_uuid = $3 AND is_deleted = FALSE
                                RETURNING *
                            )
                            SELECT
                                m.uuid, m.chat_uuid, m.sender_uuid, m.reply_to_uuid,
                                m.body, m.is_edited, m.is_deleted, m.is_pinned, m.created_at, m.updated_at,
                                gu.first_name  AS sender_first_name,
                                gu.second_name AS sender_second_name,
                                gu.avatar_uuid      AS sender_avatar_uuid,
                                0::bigint      AS delivered_count,
                                0::bigint      AS read_count,
                                NULL::delivery_status AS my_status,
                                NULL::text     AS reply_body_preview,
                                m.media_uuid
                            FROM upd m JOIN guest_user gu ON gu.uuid = m.sender_uuid"#,
                        )
                        .bind(&body)
                        .bind(uuid)
                        .bind(user_uuid)
                        .fetch_optional(&pool)
                        .await
                        .ok()
                        .flatten()?;

                        let _ = msg_chat_uuid; // используется в DTO через row.chat_uuid
                        let dto = map_message_row(&state_async, row).await;
                                  Some(WsServerEvent::MessageEdited(dto))
                    }

                    // ── Удалить сообщение ────────────────────────────────────
                    WsClientAction::DeleteMessage { uuid, chat_uuid: msg_chat_uuid } => {
                        sqlx::query(
                            "UPDATE message SET is_deleted = TRUE, body = ''
                             WHERE uuid = $1 AND sender_uuid = $2",
                        )
                        .bind(uuid)
                        .bind(user_uuid)
                        .execute(&pool)
                        .await
                        .ok()
                        .filter(|r| r.rows_affected() > 0)
                        .map(|_| WsServerEvent::MessageDeleted { uuid, chat_uuid: msg_chat_uuid })
                    }

                    // ── Отметить доставленным ────────────────────────────────
                    WsClientAction::MarkDelivered { chat_uuid: msg_chat_uuid } => {
                        upsert_statuses(&pool, msg_chat_uuid, user_uuid, DeliveryStatus::Delivered).await;
                        Some(WsServerEvent::StatusUpdated {
                            chat_uuid: msg_chat_uuid,
                            message_uuid: Uuid::nil(),
                            user_uuid,
                            status: DeliveryStatus::Delivered,
                        })
                    }

                    // ── Отметить прочитанным ─────────────────────────────────
                    WsClientAction::MarkRead { chat_uuid: msg_chat_uuid } => {
                        upsert_statuses(&pool, msg_chat_uuid, user_uuid, DeliveryStatus::Read).await;
                        Some(WsServerEvent::StatusUpdated {
                            chat_uuid: msg_chat_uuid,
                            message_uuid: Uuid::nil(),
                            user_uuid,
                            status: DeliveryStatus::Read,
                        })
                    }

                    // ── Typing ───────────────────────────────────────────────
                    WsClientAction::Typing { is_typing, chat_uuid: msg_chat_uuid } => {
                        Some(WsServerEvent::Typing { chat_uuid: msg_chat_uuid, user_uuid, is_typing })
                    }

                    // ── WebRTC (Игнор в старом per-chat сокете) ─────────────────
                    WsClientAction::CallOffer { .. } |
                    WsClientAction::CallAnswer { .. } |
                    WsClientAction::IceCandidate { .. } |
                    WsClientAction::CallReject { .. } |
                    WsClientAction::CallEnd { .. } |
                    WsClientAction::TogglePinMessage { .. } => None,
                }
            }
        })
    })
}

// ════════════════════════════════════════════════════════════════
// Global WS handler (per-user)
// ════════════════════════════════════════════════════════════════

/// Helper: найти всех участников чата
async fn get_chat_member_uuids(pool: &PgPool, chat_uuid: Uuid) -> Vec<Uuid> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT user_uuid FROM chat_member WHERE chat_uuid = $1 AND left_at IS NULL",
    )
    .bind(chat_uuid)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

/// Разослать статус пользователя всем его контактам (участникам общих чатов)
async fn broadcast_user_status(state: &Arc<AppState>, user_uuid: Uuid, is_online: bool) {
    let pool = &state.pool;
    let ws = match &state.user_ws_state {
        Some(s) => s,
        None => return,
    };

    let last_seen_at = Utc::now();

    // 1. Обновляем в БД
    let _ = sqlx::query("UPDATE guest_user SET last_seen_at = $1 WHERE uuid = $2")
        .bind(last_seen_at)
        .bind(user_uuid)
        .execute(pool)
        .await;

    // 2. Находим контакты (люди из общих чатов)
    let contacts: Vec<Uuid> = sqlx::query_scalar(
        r#"
        SELECT DISTINCT cm2.user_uuid
        FROM chat_member cm1
        JOIN chat_member cm2 ON cm2.chat_uuid = cm1.chat_uuid
        WHERE cm1.user_uuid = $1 AND cm1.left_at IS NULL
          AND cm2.user_uuid != $1 AND cm2.left_at IS NULL
        "#
    )
    .bind(user_uuid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // 3. Рассылаем событие
    let event = WsServerEvent::UserStatusChanged {
        user_uuid,
        is_online,
        last_seen_at,
    };

    ws.broadcast_to_users(&contacts, event).await;
}

/// WebSocket upgrade для глобального real-time канала пользователя
pub async fn ws_user_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(query): Query<WsQuery>,
) -> impl IntoResponse {
    let user_uuid = match get_user_from_token(&query.token) {
        Ok(id) => id,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let user_ws_state = match &state.user_ws_state {
        Some(s) => s.clone(),
        None => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };

    ws.on_upgrade(move |socket| {
        let state_inner = state.clone();
        let user_ws_state_for_handler = user_ws_state.clone();

        async move {
            broadcast_user_status(&state_inner, user_uuid, true).await;

            let state_for_closure = state_inner.clone();
            handle_user_socket(socket, user_uuid, user_ws_state_for_handler, move |user_uuid, raw| {
                let state_async = state_for_closure.clone();

                async move {
                    let action: WsClientAction = match serde_json::from_str(&raw) {
                        Ok(a) => a,
                        Err(_) => {
                            // Отправляем ошибку обратно пользователю
                            state_async.user_ws_state.as_ref().map(|ws| {
                                let ws = ws.clone();
                                tokio::spawn(async move {
                                    ws.send_to_user(user_uuid, WsServerEvent::Error {
                                        message: "Invalid JSON. Expected {\"action\":\"...\",\"payload\":{...}}".into(),
                                    }).await;
                                });
                            });
                            return;
                        }
                    };

                    let pool = &state_async.pool;
                    let user_ws = state_async.user_ws_state.as_ref().unwrap();

                    match action {
                        // ── Отправить сообщение ──────────────────────────────────
                        WsClientAction::SendMessage { chat_uuid, body, reply_to_uuid, media_uuid } => {
                            if !assert_member(pool, chat_uuid, user_uuid).await {
                                let _ = user_ws.send_to_user(user_uuid, WsServerEvent::Error { 
                                    message: "You are not a member of this chat".into() 
                                }).await;
                                return;
                            }

                            let row = sqlx::query_as::<_, MessageRow>(
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
                                    gu.last_seen_at     AS sender_last_seen_at,
                                    0::bigint      AS delivered_count,
                                    0::bigint      AS read_count,
                                    NULL::delivery_status AS my_status,
                                    reply.body     AS reply_body_preview,
                                    m.media_uuid
                                FROM ins m
                                JOIN guest_user gu ON gu.uuid = m.sender_uuid
                                LEFT JOIN message reply ON reply.uuid = m.reply_to_uuid"#,
                            )
                            .bind(chat_uuid)
                            .bind(user_uuid)
                            .bind(reply_to_uuid)
                            .bind(&body)
                            .bind(media_uuid)
                            .fetch_one(pool)
                            .await;

                            match row {
                                Ok(row) => {
                                    let dto = map_message_row(&state_async, row).await;
                                    let event = WsServerEvent::NewMessage(dto);
                                    let members = get_chat_member_uuids(pool, chat_uuid).await;
                                    user_ws.broadcast_to_users(&members, event).await;
                                }
                                Err(_e) => {
                                    let _ = user_ws.send_to_user(user_uuid, WsServerEvent::Error { 
                                        message: "Database error while sending message".into() 
                                    }).await;
                                }
                            }
                        }

                        // ── Редактировать сообщение ──────────────────────────────
                        WsClientAction::EditMessage { chat_uuid, uuid, body } => {
                            if !assert_member(pool, chat_uuid, user_uuid).await { return; }

                            let row = sqlx::query_as::<_, MessageRow>(
                                r#"WITH upd AS (
                                    UPDATE message SET body = $1, is_edited = TRUE
                                    WHERE uuid = $2 AND sender_uuid = $3 AND is_deleted = FALSE
                                    RETURNING *
                                )
                                SELECT
                                    m.uuid, m.chat_uuid, m.sender_uuid, m.reply_to_uuid,
                                    m.body, m.is_edited, m.is_deleted, m.created_at, m.updated_at,
                                    gu.first_name  AS sender_first_name,
                                    gu.second_name AS sender_second_name,
                                    gu.avatar_uuid      AS sender_avatar_uuid,
                                    gu.last_seen_at     AS sender_last_seen_at,
                                    0::bigint      AS delivered_count,
                                    0::bigint      AS read_count,
                                    NULL::delivery_status AS my_status,
                                    NULL::text     AS reply_body_preview,
                                    m.media_uuid
                                FROM upd m JOIN guest_user gu ON gu.uuid = m.sender_uuid"#,
                            )
                            .bind(&body)
                            .bind(uuid)
                            .bind(user_uuid)
                            .fetch_optional(pool)
                            .await;

                            if let Ok(Some(row)) = row {
                                let dto = map_message_row(&state_async, row).await;
                                let event = WsServerEvent::MessageEdited(dto);
                                let members = get_chat_member_uuids(pool, chat_uuid).await;
                                user_ws.broadcast_to_users(&members, event).await;
                            }
                        }

                        // ── Удалить сообщение ────────────────────────────────────
                        WsClientAction::DeleteMessage { chat_uuid, uuid } => {
                            if !assert_member(pool, chat_uuid, user_uuid).await { return; }

                            let affected = sqlx::query(
                                "UPDATE message SET is_deleted = TRUE, body = '' WHERE uuid = $1 AND sender_uuid = $2",
                            )
                            .bind(uuid)
                            .bind(user_uuid)
                            .execute(pool)
                            .await
                            .map(|r| r.rows_affected())
                            .unwrap_or(0);

                            if affected > 0 {
                                let event = WsServerEvent::MessageDeleted { uuid, chat_uuid };
                                let members = get_chat_member_uuids(pool, chat_uuid).await;
                                user_ws.broadcast_to_users(&members, event).await;
                            }
                        }

                        // ── Отметить доставленным ────────────────────────────────
                        WsClientAction::MarkDelivered { chat_uuid } => {
                            upsert_statuses(pool, chat_uuid, user_uuid, DeliveryStatus::Delivered).await;
                            let event = WsServerEvent::StatusUpdated {
                                chat_uuid,
                                message_uuid: Uuid::nil(),
                                user_uuid,
                                status: DeliveryStatus::Delivered,
                            };
                            let members = get_chat_member_uuids(pool, chat_uuid).await;
                            user_ws.broadcast_to_users(&members, event).await;
                        }

                        // ── Отметить прочитанным ─────────────────────────────────
                        WsClientAction::MarkRead { chat_uuid } => {
                            upsert_statuses(pool, chat_uuid, user_uuid, DeliveryStatus::Read).await;
                            let event = WsServerEvent::StatusUpdated {
                                chat_uuid,
                                message_uuid: Uuid::nil(),
                                user_uuid,
                                status: DeliveryStatus::Read,
                            };
                            let members = get_chat_member_uuids(pool, chat_uuid).await;
                            user_ws.broadcast_to_users(&members, event).await;
                        }

                        // ── Typing ───────────────────────────────────────────────
                        WsClientAction::Typing { chat_uuid, is_typing } => {
                            let event = WsServerEvent::Typing { chat_uuid, user_uuid, is_typing };
                            let members = get_chat_member_uuids(pool, chat_uuid).await;
                            // Typing шлём всем кроме себя — фильтрация на клиенте
                            user_ws.broadcast_to_users(&members, event).await;
                        }

                        // ── WebRTC ──────────────────────────────────────────────
                        WsClientAction::CallOffer { chat_uuid, sdp } => {
                            // Проверяем членство в чате для безопасности
                            if !assert_member(pool, chat_uuid, user_uuid).await { return; }
                            let event = WsServerEvent::CallOffer { chat_uuid, caller_uuid: user_uuid, sdp };
                            let mut members = get_chat_member_uuids(pool, chat_uuid).await;
                            members.retain(|&m| m != user_uuid);
                            user_ws.broadcast_to_users(&members, event).await;
                        }
                        
                        WsClientAction::CallAnswer { chat_uuid, sdp } => {
                            if !assert_member(pool, chat_uuid, user_uuid).await { return; }
                            let event = WsServerEvent::CallAnswer { chat_uuid, responder_uuid: user_uuid, sdp };
                            let mut members = get_chat_member_uuids(pool, chat_uuid).await;
                            members.retain(|&m| m != user_uuid);
                            user_ws.broadcast_to_users(&members, event).await;
                        }

                        WsClientAction::ReactToMessage { chat_uuid, message_uuid, emoji } => {
                            if !assert_member(pool, chat_uuid, user_uuid).await { return; }

                            if let Ok(is_added) = handle_react_to_message(pool, user_uuid, message_uuid, emoji.clone()).await {
                                let event = WsServerEvent::MessageReactionUpdated { 
                                    chat_uuid, 
                                    message_uuid, 
                                    user_uuid, 
                                    emoji, 
                                    is_added 
                                };
                                let members = get_chat_member_uuids(pool, chat_uuid).await;
                                user_ws.broadcast_to_users(&members, event).await;
                            }
                        }

                        WsClientAction::IceCandidate { chat_uuid, candidate, sdp_mid, sdp_m_line_index } => {
                            if !assert_member(pool, chat_uuid, user_uuid).await { return; }
                            let event = WsServerEvent::IceCandidate { chat_uuid, sender_uuid: user_uuid, candidate, sdp_mid, sdp_m_line_index };
                            let mut members = get_chat_member_uuids(pool, chat_uuid).await;
                            members.retain(|&m| m != user_uuid);
                            user_ws.broadcast_to_users(&members, event).await;
                        }

                        WsClientAction::CallReject { chat_uuid } => {
                            if !assert_member(pool, chat_uuid, user_uuid).await { return; }
                            let event = WsServerEvent::CallReject { chat_uuid, user_uuid };
                            let mut members = get_chat_member_uuids(pool, chat_uuid).await;
                            members.retain(|&m| m != user_uuid);
                            user_ws.broadcast_to_users(&members, event).await;
                        }

                        WsClientAction::CallEnd { chat_uuid } => {
                            if !assert_member(pool, chat_uuid, user_uuid).await { return; }
                            let event = WsServerEvent::CallEnd { chat_uuid, user_uuid };
                            let mut members = get_chat_member_uuids(pool, chat_uuid).await;
                            members.retain(|&m| m != user_uuid);
                            user_ws.broadcast_to_users(&members, event).await;
                        }

                        WsClientAction::TogglePinMessage { chat_uuid, uuid, is_pinned } => {
                            if !assert_member(pool, chat_uuid, user_uuid).await { return; }

                            let affected = sqlx::query(
                                "UPDATE message SET is_pinned = $1 WHERE uuid = $2 AND chat_uuid = $3"
                            )
                            .bind(is_pinned)
                            .bind(uuid)
                            .bind(chat_uuid)
                            .execute(pool)
                            .await
                            .map(|r| r.rows_affected())
                            .unwrap_or(0);

                            if affected > 0 {
                                let event = WsServerEvent::MessagePinned { uuid, chat_uuid, is_pinned };
                                let members = get_chat_member_uuids(pool, chat_uuid).await;
                                user_ws.broadcast_to_users(&members, event).await;
                            }
                        }

                        WsClientAction::Ping => {
                            let _ = user_ws.send_to_user(user_uuid, WsServerEvent::Pong).await;
                        }
                    }
                }
            }).await;

            // 👇 Уведомляем об офлайне
            broadcast_user_status(&state_inner, user_uuid, false).await;
        }
    })
}

/// Поиск по сообщениям
#[utoipa::path(
    get,
    path = "/{chat_uuid}/search",
    params(
        ("chat_uuid" = Uuid, Path, description = "Chat UUID"),
        ("q"         = String, Query, description = "Search query"),
        ("limit"     = Option<i64>, Query, description = "Limit results"),
    ),
    responses(
        (status = 200, body = ApiResponse<Vec<MessageResponseDTO>>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = [])),
    operation_id = "search_messages",
)]
pub async fn search_messages(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Path(chat_uuid): Path<Uuid>,
    Query(q): Query<super::dto::SearchQuery>,
) -> Result<Json<ApiResponse<Vec<MessageResponseDTO>>>, (StatusCode, Json<ApiResponse<Vec<MessageResponseDTO>>>)> {
    if !assert_member(&state.pool, chat_uuid, me).await {
        let msg = state.i18n.t("messenger.not_member", &locale).await;
        return into_api_response(StatusCode::FORBIDDEN, None, None, Some(vec![msg]));
    }

    let limit = q.limit.unwrap_or(50).min(100);
    let search_pattern = format!("%{}%", q.q);

    let rows = sqlx::query_as::<_, MessageRow>(&format!(
        "{} WHERE m.chat_uuid = $1 AND m.is_deleted = FALSE AND m.body ILIKE $3 ORDER BY m.created_at DESC LIMIT $4",
        MSG_SELECT
    ))
    .bind(chat_uuid)
    .bind(me)
    .bind(search_pattern)
    .bind(limit)
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(rows) => {
            let mut items = Vec::with_capacity(rows.len());
            for row in rows {
                items.push(map_message_row(&state, row).await);
            }
            into_api_response(StatusCode::OK, Some(items), None, None)
        }
        Err(_) => {
            let msg = state.i18n.t("general.db_error", &locale).await;
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, Some(error_map("database", &msg)), Some(vec![msg]))
        }
    }
}

/// Установить алиас для чата (собеседника)
#[utoipa::path(
    patch,
    path = "/{chat_uuid}/alias",
    params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
    request_body = SetAliasDTO,
    responses(
        (status = 200, description = "Alias updated"),
        (status = 403, description = "Not member"),
        (status = 404, description = "Not found"),
    ),
    tag = "Messenger",
    security(("bearer_auth" = [])),
    operation_id = "set_chat_alias",
)]
pub async fn set_chat_alias(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Path(chat_uuid): Path<Uuid>,
    Json(body): Json<SetAliasDTO>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    if !assert_member(&state.pool, chat_uuid, me).await {
        let msg = state.i18n.t("messenger.not_member", &locale).await;
        return into_api_response(StatusCode::FORBIDDEN, None, None, Some(vec![msg]));
    }

    let result = sqlx::query(
        "UPDATE chat_member SET alias = $1 WHERE chat_uuid = $2 AND user_uuid = $3"
    )
    .bind(&body.alias)
    .bind(chat_uuid)
    .bind(me)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => into_api_response(StatusCode::OK, None, None, None),
        Err(_) => {
            let msg = state.i18n.t("general.db_error", &locale).await;
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, Some(error_map("database", &msg)), Some(vec![msg]))
        }
    }
}

/// Получить количество вложений по типам
#[utoipa::path(
    get,
    path = "/{chat_uuid}/media/counts",
    params(("chat_uuid" = Uuid, Path, description = "Chat UUID")),
    responses(
        (status = 200, body = ApiResponse<ChatMediaCountsDTO>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = [])),
    operation_id = "get_chat_media_counts",
)]
pub async fn get_chat_media_counts(
    State(state): State<Arc<AppState>>,
    Extension(me): Extension<Uuid>,
    Path(chat_uuid): Path<Uuid>,
) -> Result<Json<ApiResponse<ChatMediaCountsDTO>>, (StatusCode, Json<ApiResponse<ChatMediaCountsDTO>>)> {
    if !assert_member(&state.pool, chat_uuid, me).await {
        return into_api_response(StatusCode::FORBIDDEN, None, None, None);
    }

    let result = sqlx::query(
        r#"
        SELECT 
            COUNT(CASE WHEN m.media_type = 'image' THEN 1 END) as images,
            COUNT(CASE WHEN m.media_type = 'video' THEN 1 END) as videos,
            COUNT(CASE WHEN m.media_type = 'audio' THEN 1 END) as audio
        FROM message msg
        JOIN media m ON msg.media_uuid = m.uuid
        WHERE msg.chat_uuid = $1 AND msg.is_deleted = FALSE
        "#,
    )
    .bind(chat_uuid)
    .fetch_one(&state.pool)
    .await;

    let counts = match result {
        Ok(r) => ChatMediaCountsDTO {
            images: r.get::<Option<i64>, _>("images").unwrap_or(0),
            videos: r.get::<Option<i64>, _>("videos").unwrap_or(0),
            audio: r.get::<Option<i64>, _>("audio").unwrap_or(0),
        },
        Err(_) => ChatMediaCountsDTO { images: 0, videos: 0, audio: 0 }
    };

    into_api_response(StatusCode::OK, Some(counts), None, None)
}

/// Получить список вложений чата
#[utoipa::path(
    get,
    path = "/{chat_uuid}/media",
    params(
        ("chat_uuid" = Uuid, Path, description = "Chat UUID"),
        ("media_type" = Option<String>, Query, description = "Filter by media type"),
        ("page" = Option<i64>, Query, description = "Page number"),
        ("limit" = Option<i64>, Query, description = "Limit"),
    ),
    responses(
        (status = 200, body = ApiResponse<Vec<MessageResponseDTO>>),
    ),
    tag = "Messenger",
    security(("bearer_auth" = [])),
    operation_id = "get_chat_media",
)]
pub async fn get_chat_media(
    State(state): State<Arc<AppState>>,
    Extension(me): Extension<Uuid>,
    Path(chat_uuid): Path<Uuid>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<MessageResponseDTO>>>, (StatusCode, Json<ApiResponse<Vec<MessageResponseDTO>>>)> {
    if !assert_member(&state.pool, chat_uuid, me).await {
        return into_api_response(StatusCode::FORBIDDEN, None, None, None);
    }

    let limit = q.get("limit").and_then(|s| s.parse::<i64>().ok()).unwrap_or(50);
    let page = q.get("page").and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
    let offset = (page - 1) * limit;

    let mut sql = MSG_SELECT.to_string();
    sql.push_str(" WHERE m.chat_uuid = $1 AND m.is_deleted = FALSE AND m.media_uuid IS NOT NULL");
    
    let rows = if let Some(mt) = q.get("media_type") {
        sql.push_str(" AND EXISTS (SELECT 1 FROM media med WHERE med.uuid = m.media_uuid AND med.media_type = $3::media_type)");
        sql.push_str(" ORDER BY m.created_at DESC LIMIT $4 OFFSET $5");
        sqlx::query_as::<_, MessageRow>(&sql)
            .bind(chat_uuid)
            .bind(me)
            .bind(mt)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.pool)
            .await
    } else {
        sql.push_str(" ORDER BY m.created_at DESC LIMIT $3 OFFSET $4");
        sqlx::query_as::<_, MessageRow>(&sql)
            .bind(chat_uuid)
            .bind(me)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.pool)
            .await
    };

    match rows {
        Ok(rows) => {
            let mut items = Vec::with_capacity(rows.len());
            for row in rows {
                items.push(map_message_row(&state, row).await);
            }
            into_api_response(StatusCode::OK, Some(items), None, None)
        }
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            into_api_response(StatusCode::INTERNAL_SERVER_ERROR, None, None, None)
        }
    }
}


// ════════════════════════════════════════════════════════════════
// Router
// ════════════════════════════════════════════════════════════════

/// REST-роутер для подключения через .nest("/api/v1/chats", messenger::handlers::router())
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // ── Chats ──────────────────────────────────────────────
        .route("/",            routing::get(get_chats).post(create_chat))
        .route("/{chat_uuid}", routing::get(get_chat).delete(leave_or_delete_chat))
        .route("/{chat_uuid}/alias", routing::patch(set_chat_alias))
        // ── Members ────────────────────────────────────────────
        .route("/{chat_uuid}/members",            routing::get(get_members).post(add_member))
        .route("/{chat_uuid}/members/{user_uuid}", routing::delete(remove_member))
        // ── Messages ───────────────────────────────────────────
        .route("/{chat_uuid}/messages",            routing::get(get_messages).post(send_message))
        .route("/{chat_uuid}/messages/{msg_uuid}", routing::put(edit_message).delete(delete_message))
        .route("/{chat_uuid}/messages/delivered",  routing::post(mark_delivered))
        .route("/{chat_uuid}/messages/read",       routing::post(mark_read))
        .route("/{chat_uuid}/search",             routing::get(search_messages))
        .route("/{chat_uuid}/media/counts",       routing::get(get_chat_media_counts))
        .route("/{chat_uuid}/media",               routing::get(get_chat_media))
}

pub fn ws_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{chat_uuid}", routing::get(ws_upgrade))
}

/// Глобальный WS-роутер — один канал на пользователя
pub fn ws_user_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", routing::get(ws_user_upgrade))
}
