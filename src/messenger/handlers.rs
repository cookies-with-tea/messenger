use std::{collections::HashMap, sync::Arc};

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing,
};
use sqlx::{PgPool, query_as};
use uuid::Uuid;

use crate::{AppState, auth::utils::get_user_from_token, core::{dto::{ApiPaginationDTO, ApiResponse, ApiResponseWithPagination, MediaDTO, PaginationDTO, PaginationQuery}, handlers::{get_media_by_uuid, get_media_by_uuids}, response::{error_map, into_api_response, into_api_response_with_pagination}}, messenger::dto::{ChatRow, UserPreviewDTO, WsQuery}};

use super::{
    dto::{
        AddMemberDTO, ChatMemberDTO, ChatResponseDTO, ChatType, CreateChatDTO, CreateMessageDTO,
        DeliveryStatus, MessageQuery, MessageResponseDTO, UpdateChatDTO, UpdateMessageDTO,
        WsClientAction, WsServerEvent,
    },
    ws::{WsState, handle_socket},
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
            gu.uuid AS creator_uuid,
            gu.first_name AS creator_first_name,
            gu.second_name AS creator_second_name,
            gu.avatar AS creator_avatar_uuid
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
            if let Some(u) = row.interlocutor_avatar_uuid { uuids.push(u); }

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

        interlocutor: row.interlocutor_uuid.map(|_| UserPreviewDTO {
            first_name: row.interlocutor_first_name,
            second_name: row.interlocutor_second_name,
            avatar: row.interlocutor_avatar_uuid.and_then(|uuid| media_map.get(&uuid).cloned()),
        }),
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

            -- 👇 Данные собеседника (второго участника)
            cm_other.user_uuid           AS interlocutor_uuid,
            gu.first_name                AS interlocutor_first_name,
            gu.second_name               AS interlocutor_second_name,
            gu.avatar                    AS interlocutor_avatar_uuid

        FROM chat c
        JOIN chat_member cm ON cm.chat_uuid = c.uuid
            AND cm.user_uuid = $1
            AND cm.left_at IS NULL

        -- 👈 Присоединяем ДРУГОГО участника (не $1)
        LEFT JOIN chat_member cm_other ON cm_other.chat_uuid = c.uuid
            AND cm_other.user_uuid != $1
            AND cm_other.left_at IS NULL

        -- 👈 Получаем данные пользователя из guest_user
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
        Err(_) => {
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
        let interlocutor_avatar = get_media_by_uuid(&state, row.interlocutor_avatar_uuid)
            .await
            .ok()
            .flatten();

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

            interlocutor: row.interlocutor_uuid.map(|_| UserPreviewDTO {
                first_name: row.interlocutor_first_name,
                second_name: row.interlocutor_second_name,
                avatar: interlocutor_avatar,
            }),
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
            NULL::bigint AS unread_count,
            (SELECT COUNT(*) FROM chat_member cm WHERE cm.chat_uuid = $1 AND cm.left_at IS NULL) AS member_count,

            -- 👇 Собеседник
            cm_other.user_uuid           AS interlocutor_uuid,
            gu.first_name                AS interlocutor_first_name,
            gu.second_name               AS interlocutor_second_name,
            gu.avatar                    AS interlocutor_avatar_uuid
        FROM chat c
        JOIN chat_member cm ON cm.chat_uuid = c.uuid
            AND cm.user_uuid = $1
            AND cm.left_at IS NULL
        LEFT JOIN chat_member cm_other ON cm_other.chat_uuid = c.uuid
            AND cm_other.user_uuid != $1
            AND cm_other.left_at IS NULL
        LEFT JOIN guest_user gu ON gu.uuid = cm_other.user_uuid
        WHERE c.uuid = $1
        "#,
    )
    .bind(chat_uuid)
    .fetch_optional(&state.pool)
    .await;

    match row {
      Ok(Some(row)) => {
           let interlocutor_avatar = get_media_by_uuid(&state, row.interlocutor_avatar_uuid)
               .await
               .ok()
               .flatten();

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
               interlocutor: row.interlocutor_uuid.map(|_| UserPreviewDTO {
                   first_name: row.interlocutor_first_name,
                   second_name: row.interlocutor_second_name,
                   avatar: interlocutor_avatar,
               }),
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
    let is_creator: bool = sqlx::query_scalar("SELECT created_by = $2 FROM chat WHERE uuid = $1")
        .bind(chat_uuid)
        .bind(me)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(false);

    if is_creator {
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
            u.avatar
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
        Ok(items) => into_api_response(StatusCode::OK, Some(items), None, None),
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
                ws.broadcast(chat_uuid, WsServerEvent::MemberJoined(m.clone()));
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
        ws.broadcast(chat_uuid, WsServerEvent::MemberLeft { chat_uuid, user_uuid: target });
    }

    into_api_response::<()>(StatusCode::OK, None, None, None)
}

// ════════════════════════════════════════════════════════════════
// Message handlers
// ════════════════════════════════════════════════════════════════

const MSG_SELECT: &str = r#"
    SELECT
        m.uuid,
        m.chat_uuid,
        m.sender_uuid,
        m.reply_to_uuid,
        m.body,
        m.is_edited,
        m.is_deleted,
        m.created_at,
        m.updated_at,
        u.first_name  AS sender_first_name,
        u.last_name   AS sender_last_name,
        u.avatar      AS sender_avatar,
        (SELECT COUNT(*) FROM message_status ms
         WHERE ms.message_uuid = m.uuid AND ms.status = 'delivered') AS delivered_count,
        (SELECT COUNT(*) FROM message_status ms
         WHERE ms.message_uuid = m.uuid AND ms.status = 'read')      AS read_count,
        (SELECT ms.status FROM message_status ms
         WHERE ms.message_uuid = m.uuid AND ms.user_uuid = $2
         LIMIT 1)                                                     AS my_status,
        reply.body    AS reply_body_preview
    FROM message m
    JOIN guest_user u    ON u.uuid    = m.sender_uuid
    LEFT JOIN message reply ON reply.uuid = m.reply_to_uuid
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

    let rows = if let Some(before) = q.before_uuid {
        // Курсорная пагинация
        query_as::<_, MessageResponseDTO>(&format!(
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
        // Offset пагинация
        query_as::<_, MessageResponseDTO>(&format!(
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

    match rows {
        Ok(mut items) => {
            items.reverse(); // хронологический порядок
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
        Err(_) => {
            let msg = state.i18n.t("general.db_error", &locale).await;
            into_api_response_with_pagination(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                Some(error_map("database", &msg)),
                Some(vec![msg]),
            )
        }
    }
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

    let row = query_as::<_, MessageResponseDTO>(&format!(
        "WITH ins AS (
             INSERT INTO message (chat_uuid, sender_uuid, reply_to_uuid, body)
             VALUES ($1, $2, $3, $4)
             RETURNING *
         )
         {MSG_SELECT_INS}",
        MSG_SELECT_INS = MSG_SELECT
            .replace("FROM message m", "FROM ins m")
            .replace("WHERE m.chat_uuid = $1 AND m.is_deleted = FALSE", ""),
    ))
    .bind(chat_uuid)
    .bind(me)
    .bind(body.reply_to_uuid)
    .bind(&body.body)
    .fetch_one(&state.pool)
    .await;

    // Упрощённый INSERT + SELECT
    let row = sqlx::query_as::<_, MessageResponseDTO>(&format!(
        r#"WITH ins AS (
            INSERT INTO message (chat_uuid, sender_uuid, reply_to_uuid, body)
            VALUES ($1, $2, $3, $4) RETURNING *
        )
        SELECT
            m.uuid, m.chat_uuid, m.sender_uuid, m.reply_to_uuid,
            m.body, m.is_edited, m.is_deleted, m.created_at, m.updated_at,
            u.first_name AS sender_first_name,
            u.last_name  AS sender_last_name,
            u.avatar     AS sender_avatar,
            0::bigint    AS delivered_count,
            0::bigint    AS read_count,
            NULL::delivery_status AS my_status,
            reply.body   AS reply_body_preview
        FROM ins m
        JOIN guest_user u    ON u.uuid    = m.sender_uuid
        LEFT JOIN message reply ON reply.uuid = m.reply_to_uuid"#
    ))
    .bind(chat_uuid)
    .bind(me)
    .bind(body.reply_to_uuid)
    .bind(&body.body)
    .fetch_one(&state.pool)
    .await;

    match row {
        Ok(msg) => {
            if let Some(ws) = &state.ws_state {
                ws.broadcast(chat_uuid, WsServerEvent::NewMessage(msg.clone()));
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
    let row = sqlx::query_as::<_, MessageResponseDTO>(
        r#"WITH upd AS (
            UPDATE message SET body = $1, is_edited = TRUE
            WHERE uuid = $2 AND sender_uuid = $3 AND chat_uuid = $4 AND is_deleted = FALSE
            RETURNING *
        )
        SELECT
            m.uuid, m.chat_uuid, m.sender_uuid, m.reply_to_uuid,
            m.body, m.is_edited, m.is_deleted, m.created_at, m.updated_at,
            u.first_name AS sender_first_name,
            u.last_name  AS sender_last_name,
            u.avatar     AS sender_avatar,
            (SELECT COUNT(*) FROM message_status ms WHERE ms.message_uuid = m.uuid AND ms.status = 'delivered') AS delivered_count,
            (SELECT COUNT(*) FROM message_status ms WHERE ms.message_uuid = m.uuid AND ms.status = 'read')      AS read_count,
            NULL::delivery_status AS my_status,
            NULL::text            AS reply_body_preview
        FROM upd m
        JOIN guest_user u ON u.uuid = m.sender_uuid"#,
    )
    .bind(&body.body)
    .bind(msg_uuid)
    .bind(me)
    .bind(chat_uuid)
    .fetch_optional(&state.pool)
    .await;

    match row {
        Ok(Some(msg)) => {
            if let Some(ws) = &state.ws_state {
                ws.broadcast(chat_uuid, WsServerEvent::MessageEdited(msg.clone()));
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

    if let Some(ws) = &state.ws_state {
        ws.broadcast(chat_uuid, WsServerEvent::MessageDeleted { uuid: msg_uuid, chat_uuid });
    }
    into_api_response::<()>(StatusCode::OK, None, None, None)
}

pub async fn mark_delivered(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Path(chat_uuid): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    upsert_statuses(&state.pool, chat_uuid, me, DeliveryStatus::Delivered).await;

    if let Some(ws) = &state.ws_state {
        ws.broadcast(chat_uuid, WsServerEvent::StatusUpdated {
            chat_uuid,
            message_uuid: Uuid::nil(), // bulk — клиент сам обновляет весь чат
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
        ws.broadcast(chat_uuid, WsServerEvent::StatusUpdated {
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
        handle_socket(socket, chat_uuid, user_uuid, ws_state, move |chat_uuid, user_uuid, raw| {
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
                    WsClientAction::SendMessage { body, reply_to_uuid } => {
                        sqlx::query_as::<_, MessageResponseDTO>(
                            r#"WITH ins AS (
                                INSERT INTO message (chat_uuid, sender_uuid, reply_to_uuid, body)
                                VALUES ($1, $2, $3, $4) RETURNING *
                            )
                            SELECT
                                m.uuid, m.chat_uuid, m.sender_uuid, m.reply_to_uuid,
                                m.body, m.is_edited, m.is_deleted, m.created_at, m.updated_at,
                                u.first_name AS sender_first_name,
                                u.last_name  AS sender_last_name,
                                u.avatar     AS sender_avatar,
                                0::bigint    AS delivered_count,
                                0::bigint    AS read_count,
                                NULL::delivery_status AS my_status,
                                reply.body   AS reply_body_preview
                            FROM ins m
                            JOIN guest_user u    ON u.uuid    = m.sender_uuid
                            LEFT JOIN message reply ON reply.uuid = m.reply_to_uuid"#,
                        )
                        .bind(chat_uuid)
                        .bind(user_uuid)
                        .bind(reply_to_uuid)
                        .bind(&body)
                        .fetch_one(&pool)
                        .await
                        .ok()
                        .map(WsServerEvent::NewMessage)
                    }

                    // ── Редактировать сообщение ──────────────────────────────
                    WsClientAction::EditMessage { uuid, body } => {
                        sqlx::query_as::<_, MessageResponseDTO>(
                            r#"WITH upd AS (
                                UPDATE message SET body = $1, is_edited = TRUE
                                WHERE uuid = $2 AND sender_uuid = $3 AND is_deleted = FALSE
                                RETURNING *
                            )
                            SELECT
                                m.uuid, m.chat_uuid, m.sender_uuid, m.reply_to_uuid,
                                m.body, m.is_edited, m.is_deleted, m.created_at, m.updated_at,
                                u.first_name AS sender_first_name,
                                u.last_name  AS sender_last_name,
                                u.avatar     AS sender_avatar,
                                0::bigint    AS delivered_count,
                                0::bigint    AS read_count,
                                NULL::delivery_status AS my_status,
                                NULL::text   AS reply_body_preview
                            FROM upd m JOIN guest_user u ON u.uuid = m.sender_uuid"#,
                        )
                        .bind(&body)
                        .bind(uuid)
                        .bind(user_uuid)
                        .fetch_optional(&pool)
                        .await
                        .ok()
                        .flatten()
                        .map(WsServerEvent::MessageEdited)
                    }

                    // ── Удалить сообщение ────────────────────────────────────
                    WsClientAction::DeleteMessage { uuid } => {
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
                        .map(|_| WsServerEvent::MessageDeleted { uuid, chat_uuid })
                    }

                    // ── Отметить доставленным ────────────────────────────────
                    WsClientAction::MarkDelivered { chat_uuid } => {
                        upsert_statuses(&pool, chat_uuid, user_uuid, DeliveryStatus::Delivered).await;
                        Some(WsServerEvent::StatusUpdated {
                            chat_uuid,
                            message_uuid: Uuid::nil(),
                            user_uuid,
                            status: DeliveryStatus::Delivered,
                        })
                    }

                    // ── Отметить прочитанным ─────────────────────────────────
                    WsClientAction::MarkRead { chat_uuid } => {
                        upsert_statuses(&pool, chat_uuid, user_uuid, DeliveryStatus::Read).await;
                        Some(WsServerEvent::StatusUpdated {
                            chat_uuid,
                            message_uuid: Uuid::nil(),
                            user_uuid,
                            status: DeliveryStatus::Read,
                        })
                    }

                    // ── Typing ───────────────────────────────────────────────
                    WsClientAction::Typing { is_typing } => {
                        Some(WsServerEvent::Typing { chat_uuid, user_uuid, is_typing })
                    }
                }
            }
        })
    })
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
        // ── Members ────────────────────────────────────────────
        .route("/{chat_uuid}/members",            routing::get(get_members).post(add_member))
        .route("/{chat_uuid}/members/{user_uuid}", routing::delete(remove_member))
        // ── Messages ───────────────────────────────────────────
        .route("/{chat_uuid}/messages",            routing::get(get_messages).post(send_message))
        .route("/{chat_uuid}/messages/{msg_uuid}", routing::put(edit_message).delete(delete_message))
        .route("/{chat_uuid}/messages/delivered",  routing::post(mark_delivered))
        .route("/{chat_uuid}/messages/read",       routing::post(mark_read))
}

pub fn ws_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{chat_uuid}", routing::get(ws_upgrade))
}
