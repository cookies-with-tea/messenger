use crate::core::dto::{ApiPaginationDTO, ApiResponse, ApiResponseWithPagination, PaginationDTO};
use crate::core::response::{error_map, into_api_response, into_api_response_with_pagination};
use crate::user::dto::{
    CreateUserDTO, UpdateUserDTO, User, UserResponseDTO, UserRole, UserSearchQuery, UserStatus,
};
use crate::user::utils::{validate_email, validate_phone};
use crate::AppState;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use axum::Router;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::get,
    Json,
};
use sqlx::query_as;
use std::sync::Arc;
use uuid::Uuid;

fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

#[utoipa::path(
    post,
    path = "/api/v1/user",
    request_body = CreateUserDTO,
    responses(
        (status = 201, description = "Пользователь создан"),
        (status = 409, description = "Пользователь существует"),
        (status = 500, description = "Внутренняя ошибка")
    ),
    tag = "User",
    operation_id = "create_user",
)]
pub async fn create(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Json(payload): Json<CreateUserDTO>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let is_email_valid = validate_email(&payload.email.clone());

    if !is_email_valid {
        let msg = state.i18n.t("user.email_invalid", &locale).await;
        return into_api_response(
            StatusCode::BAD_REQUEST,
            None,
            Some(error_map(&"email".to_string(), &msg.clone())),
            Some(vec![msg]),
        );
    }

    let mut is_phone_valid = true;

    if payload.phone.is_some() {
        is_phone_valid = validate_phone(&payload.phone.clone().unwrap_or_else(|| "".to_string()));
    }

    if !is_phone_valid {
        let msg = state.i18n.t("user.phone_invalid", &locale).await;
        return into_api_response(
            StatusCode::BAD_REQUEST,
            None,
            Some(error_map(&"phone".to_string(), &msg.clone())),
            Some(vec![msg]),
        );
    }

    if let Some(phone) = &payload.phone {
        let existing_user =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM guest_user WHERE phone = $1")
                .bind(phone)
                .fetch_one(&state.pool)
                .await;

        match existing_user {
            Ok(count) if count > 0 => {
                let msg = state.i18n.t("user.phone_exists", &locale).await;
                return into_api_response(
                    StatusCode::CONFLICT,
                    None,
                    Some(error_map(&"phone".to_string(), &msg.clone())),
                    Some(vec![msg]),
                );
            }
            Err(_) => {
                let msg = state.i18n.t("user.check_exists_error", &locale).await;
                return into_api_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    None,
                    Some(error_map(&"database".to_string(), &msg.clone())),
                    Some(vec![msg]),
                );
            }
            _ => {}
        }
    }

    let existing_user =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM guest_user WHERE email = $1")
            .bind(&payload.email.clone())
            .fetch_one(&state.pool)
            .await;

    match existing_user {
        Ok(count) if count > 0 => {
            let msg = state.i18n.t("user.email_exists", &locale).await;
            return into_api_response(
                StatusCode::CONFLICT,
                None,
                Some(error_map(&"email".to_string(), &msg.clone())),
                Some(vec![msg]),
            );
        }
        Err(_) => {
            let msg = state.i18n.t("user.check_exists_error", &locale).await;
            return into_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                Some(error_map(&"database".to_string(), &msg.clone())),
                Some(vec![msg]),
            );
        }
        _ => {}
    }

    let password_hash = match hash_password(&payload.password) {
        Ok(hash) => hash,
        Err(_) => {
            let msg = state.i18n.t("user.password_hash_error", &locale).await;
            return into_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                Some(error_map(&"password".to_string(), &msg.clone())),
                Some(vec![msg]),
            );
        }
    };

    let role = match payload.role {
        Some(role) => role,
        None => UserRole::User,
    };

    let status = match payload.status {
        Some(status) => status,
        None => UserStatus::Active,
    };

    let result = sqlx::query(
        "INSERT INTO guest_user (
            first_name, second_name, last_name, phone, birth_date, password_hash, role, status, email, avatar, avatar_uuid, street, gender, city
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
    )
    .bind(&payload.first_name.unwrap_or_else(|| "".to_string()))
    .bind(&payload.second_name.unwrap_or_else(|| "".to_string()))
    .bind(&payload.last_name.unwrap_or_else(|| "".to_string()))
    .bind(&payload.phone.unwrap_or_else(|| "".to_string()))
    .bind(payload.birth_date)
    .bind(&password_hash)
    .bind(&role)
    .bind(&status)
    .bind(&payload.email)
    .bind(&payload.avatar.unwrap_or_else(|| "".to_string()))
    .bind(&payload.avatar_uuid)
    .bind(&payload.street.unwrap_or_else(|| "".to_string()))
    .bind(&payload.gender.unwrap_or_else(|| "".to_string()))
    .bind(&payload.city.unwrap_or_else(|| "".to_string()))
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let msg = state.i18n.t("user.created", &locale).await;
            into_api_response(StatusCode::CREATED, None, None, Some(vec![msg]))
        }
        Err(_e) => {
            println!("{:?}", _e);
            let msg = state.i18n.t("general.db_error", &locale).await;
            into_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                Some(error_map(&"database".to_string(), &msg.clone())),
                Some(vec![msg]),
            )
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/user",
    params(
        ("page" = Option<i32>, Query, description = "Page number"),
        ("limit" = Option<i32>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, body = ApiResponseWithPagination<UserResponseDTO>),
        (status = 500, body = ApiResponseWithPagination<UserResponseDTO>)
    ),
    tag = "User",
    operation_id = "get_all_users",
)]
async fn get_all(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Query(query): Query<UserSearchQuery>,
) -> Result<
    Json<ApiResponseWithPagination<UserResponseDTO>>,
    (StatusCode, Json<ApiResponseWithPagination<UserResponseDTO>>),
> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let offset = (page - 1) * limit;

    let search_pattern = query
        .search
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s));

    let total_query = match &search_pattern {
        Some(pattern) => {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM guest_user
                 WHERE email ILIKE $1
                    OR phone ILIKE $1
                    OR first_name ILIKE $1
                    OR second_name ILIKE $1
                    OR last_name ILIKE $1",
            )
            .bind(pattern)
            .fetch_one(&state.pool)
            .await
        }
        None => {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM guest_user")
                .fetch_one(&state.pool)
                .await
        }
    };

    let total = match total_query {
        Ok(count) => count,
        Err(_) => {
            let msg = state.i18n.t("general.db_error", &locale).await;
            return into_api_response_with_pagination(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                Some(error_map(&"database".to_string(), &msg.clone())),
                Some(vec![msg]),
            );
        }
    };

    let total_pages = (total as f64 / limit as f64).ceil() as i32;

    let users_query = match &search_pattern {
        Some(pattern) => {
            query_as::<_, UserResponseDTO>(
                "SELECT
                    uuid, first_name, second_name, last_name,
                    phone, email, birth_date, avatar,
                    street, gender, city, role, status,
                    created_at, updated_at
                 FROM guest_user
                 WHERE email ILIKE $1
                    OR phone ILIKE $1
                    OR first_name ILIKE $1
                    OR second_name ILIKE $1
                    OR last_name ILIKE $1
                 ORDER BY created_at
                 LIMIT $2 OFFSET $3",
            )
            .bind(pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.pool)
            .await
        }
        None => {
            query_as::<_, UserResponseDTO>(
                "SELECT
                    uuid, first_name, second_name, last_name,
                    phone, email, birth_date, avatar,
                    street, gender, city, role, status,
                    created_at, updated_at
                 FROM guest_user
                 ORDER BY created_at
                 LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.pool)
            .await
        }
    };

    match users_query {
        Ok(users) => {
            let pagination = PaginationDTO {
                page,
                total: Some(total as i32),
                total_pages: Some(total_pages),
                limit: Some(limit),
            };

            let api_pagination = ApiPaginationDTO {
                items: users,
                pagination,
            };

            into_api_response_with_pagination(StatusCode::OK, Some(api_pagination), None, None)
        }
        Err(_) => {
            let msg = state.i18n.t("general.db_error", &locale).await;
            return into_api_response_with_pagination(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(ApiPaginationDTO {
                    items: vec![],
                    pagination: PaginationDTO {
                        page,
                        total: Some(0),
                        total_pages: Some(0),
                        limit: Some(limit),
                    },
                }),
                Some(error_map("database", &msg)),
                Some(vec![msg]),
            );
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/user/{uuid}",
    params(("uuid" = Uuid, Path, description = "User UUID")),
    responses(
        (status = 200, body = ApiResponse<UserResponseDTO>),
        (status = 404, body = ApiResponse<UserResponseDTO>),
        (status = 500, body = ApiResponse<UserResponseDTO>)
    ),
    tag = "User",
    operation_id = "get_user_by_uuid",
)]
async fn get_one(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Path(uuid): Path<Uuid>,
) -> Result<Json<ApiResponse<UserResponseDTO>>, (StatusCode, Json<ApiResponse<UserResponseDTO>>)> {
    let result = sqlx::query_as::<_, User>("SELECT * FROM guest_user WHERE uuid = $1")
        .bind(uuid)
        .fetch_optional(&state.pool)
        .await;

    match result {
        Ok(Some(user)) => {
            let user_response = UserResponseDTO {
                uuid: user.uuid,
                first_name: user.first_name,
                second_name: user.second_name,
                last_name: user.last_name,
                phone: user.phone,
                email: user.email,
                avatar: user
                    .avatar_uuid
                    .map(|uuid| format!("{}/media/image/{}.png", state.media_base_url, uuid)),
                role: user.role,
                birth_date: user.birth_date,
                created_at: user.created_at,
                updated_at: user.updated_at,
                street: user.street,
                city: user.city,
                status: user.status,
                gender: user.gender,
            };

            into_api_response(StatusCode::OK, Some(user_response), None, None)
        }
        Ok(None) => {
            let msg = state.i18n.t("user.not_found", &locale).await;
            into_api_response(
                StatusCode::NOT_FOUND,
                None,
                Some(error_map(&"user".to_string(), &msg.clone())),
                Some(vec![msg]),
            )
        }
        Err(_) => {
            let msg = state.i18n.t("general.db_error", &locale).await;
            into_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                Some(error_map(&"database".to_string(), &msg.clone())),
                Some(vec![msg]),
            )
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/user/{uuid}",
    params(("uuid" = Uuid, Path, description = "User UUID")),
    responses(
        (status = 200, description = "Пользователь удален"),
        (status = 404, description = "Не найден"),
        (status = 500, description = "Ошибка базы данных")
    ),
    tag = "User",
    operation_id = "delete_user_by_uuid",
)]
async fn delete_one(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Path(uuid): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    if me != uuid {
        let msg = state.i18n.t("user.not_allowed_to_delete", &locale).await;
        return into_api_response(StatusCode::FORBIDDEN, None, None, Some(vec![msg]));
    }
    let exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM guest_user WHERE uuid = $1)")
            .bind(uuid)
            .fetch_one(&state.pool)
            .await;

    match exists {
        Ok(true) => {
            let result = sqlx::query("DELETE FROM guest_user WHERE uuid = $1")
                .bind(uuid)
                .execute(&state.pool)
                .await;
            match result {
                Ok(_) => {
                    let msg = state.i18n.t("user.deleted", &locale).await;
                    into_api_response(StatusCode::OK, None, None, Some(vec![msg]))
                }
                Err(_) => {
                    let msg = state.i18n.t("general.db_error", &locale).await;
                    into_api_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        None,
                        Some(error_map(&"database".to_string(), &msg.clone())),
                        Some(vec![msg]),
                    )
                }
            }
        }
        Ok(false) => {
            let msg = state.i18n.t("user.not_found", &locale).await;
            into_api_response(
                StatusCode::NOT_FOUND,
                None,
                Some(error_map(&"user".to_string(), &msg.clone())),
                Some(vec![msg]),
            )
        }
        Err(_) => {
            let msg = state.i18n.t("general.db_error", &locale).await;
            into_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                Some(error_map(&"database".to_string(), &msg.clone())),
                Some(vec![msg]),
            )
        }
    }
}

#[utoipa::path(
    patch,
    path = "/api/v1/user/{uuid}",
    params(("uuid" = Uuid, Path, description = "User UUID")),
    request_body = UpdateUserDTO,
    responses(
        (status = 200, body = ApiResponse<UserResponseDTO>),
        (status = 404, body = ApiResponse<UserResponseDTO>),
        (status = 500, body = ApiResponse<UserResponseDTO>)
    ),
    tag = "User",
    operation_id = "update_user_by_uuid",
)]
async fn update(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Extension(me): Extension<Uuid>,
    Path(uuid): Path<Uuid>,
    Json(payload): Json<UpdateUserDTO>,
) -> Result<Json<ApiResponse<UserResponseDTO>>, (StatusCode, Json<ApiResponse<UserResponseDTO>>)> {
    if me != uuid {
        let msg = state.i18n.t("user.not_allowed_to_update", &locale).await;
        return into_api_response(StatusCode::FORBIDDEN, None, None, Some(vec![msg]));
    }

    let existing_user = sqlx::query_as::<_, User>("SELECT * FROM guest_user WHERE uuid = $1")
        .bind(uuid)
        .fetch_optional(&state.pool)
        .await;

    match existing_user {
        Ok(Some(_)) => {
            let mut update_query = "UPDATE guest_user SET ".to_string();
            let mut query_param_index = 1;

            if let Some(_email) = &payload.email {
                update_query.push_str(&format!("email = ${}, ", query_param_index));
                query_param_index += 1;
            }

            if let Some(_phone) = &payload.phone {
                update_query.push_str(&format!("phone = ${}, ", query_param_index));
                query_param_index += 1;
            }

            if let Some(_first_name) = &payload.first_name {
                update_query.push_str(&format!("first_name = ${}, ", query_param_index));
                query_param_index += 1;
            }

            if let Some(_second_name) = &payload.second_name {
                update_query.push_str(&format!("second_name = ${}, ", query_param_index));
                query_param_index += 1;
            }

            if let Some(_last_name) = &payload.last_name {
                update_query.push_str(&format!("last_name = ${}, ", query_param_index));
                query_param_index += 1;
            }

            if let Some(_birth_date) = &payload.birth_date {
                update_query.push_str(&format!("birth_date = ${}, ", query_param_index));
                query_param_index += 1;
            }

            if let Some(_role) = &payload.role {
                update_query.push_str(&format!("role = ${}, ", query_param_index));
                query_param_index += 1;
            }

            if let Some(_status) = &payload.status {
                update_query.push_str(&format!("status = ${}, ", query_param_index));
                query_param_index += 1;
            }

            if let Some(_avatar) = &payload.avatar {
                update_query.push_str(&format!("avatar = ${}, ", query_param_index));
                query_param_index += 1;
            }

            if let Some(_street) = &payload.street {
                update_query.push_str(&format!("street = ${}, ", query_param_index));
                query_param_index += 1;
            }

            if let Some(_gender) = &payload.gender {
                update_query.push_str(&format!("gender = ${}, ", query_param_index));
                query_param_index += 1;
            }

            if let Some(_city) = &payload.city {
                update_query.push_str(&format!("city = ${}, ", query_param_index));
                query_param_index += 1;
            }

            if let Some(_avatar_uuid) = &payload.avatar_uuid {
                update_query.push_str(&format!("avatar_uuid = ${}::uuid, ", query_param_index));
                query_param_index += 1;
            }

            if query_param_index == 1 {
                let msg = state.i18n.t("user.no_fields_to_update", &locale).await;
                return into_api_response(
                    StatusCode::BAD_REQUEST,
                    None,
                    Some(error_map(&"update".to_string(), &msg.clone())),
                    Some(vec![msg]),
                );
            }

            update_query.push_str(&format!(
                "updated_at = NOW() WHERE uuid = ${}",
                query_param_index
            ));

            let mut query = sqlx::query(&update_query);

            let mut _bind_param_index = 1;
            if let Some(email) = &payload.email {
                query = query.bind(email.clone());
                _bind_param_index += 1;
            }

            if let Some(phone) = &payload.phone {
                query = query.bind(phone.clone());
                _bind_param_index += 1;
            }

            if let Some(first_name) = &payload.first_name {
                query = query.bind(first_name.clone());
                _bind_param_index += 1;
            }

            if let Some(second_name) = &payload.second_name {
                query = query.bind(second_name.clone());
                _bind_param_index += 1;
            }

            if let Some(last_name) = &payload.last_name {
                query = query.bind(last_name.clone());
                _bind_param_index += 1;
            }

            if let Some(birth_date) = &payload.birth_date {
                query = query.bind(*birth_date);
                _bind_param_index += 1;
            }

            if let Some(role) = &payload.role {
                query = query.bind(role);
                _bind_param_index += 1;
            }

            if let Some(status) = &payload.status {
                query = query.bind(status);
                _bind_param_index += 1;
            }

            if let Some(avatar) = &payload.avatar {
                query = query.bind(avatar.clone());
                _bind_param_index += 1;
            }

            if let Some(street) = &payload.street {
                query = query.bind(street.clone());
                _bind_param_index += 1;
            }

            if let Some(gender) = &payload.gender {
                query = query.bind(gender.clone());
                _bind_param_index += 1;
            }

            if let Some(city) = &payload.city {
                query = query.bind(city.clone());
                _bind_param_index += 1;
            }

            if let Some(avatar_uuid) = &payload.avatar_uuid {
                query = query.bind(avatar_uuid);
                _bind_param_index += 1;
            }

            query = query.bind(uuid);

            let result = query.execute(&state.pool).await;

            match result {
                Ok(_) => {
                    let updated_user =
                        sqlx::query_as::<_, User>("SELECT * FROM guest_user WHERE uuid = $1")
                            .bind(uuid)
                            .fetch_one(&state.pool)
                            .await;

                    match updated_user {
                        Ok(user) => {
                            let user_response = UserResponseDTO {
                                uuid: user.uuid,
                                first_name: user.first_name,
                                second_name: user.second_name,
                                last_name: user.last_name,
                                phone: user.phone,
                                email: user.email,
                                avatar: user.avatar_uuid.map(|uuid| {
                                    format!("{}/media/image/{}.png", state.media_base_url, uuid)
                                }),
                                role: user.role,
                                birth_date: user.birth_date,
                                created_at: user.created_at,
                                updated_at: user.updated_at,
                                street: user.street,
                                city: user.city,
                                status: user.status,
                                gender: user.gender,
                            };

                            let msg = state.i18n.t("user.updated", &locale).await;
                            into_api_response(
                                StatusCode::OK,
                                Some(user_response),
                                None,
                                Some(vec![msg]),
                            )
                        }
                        Err(e) => {
                            eprintln!("[UpdateUser] Failed to fetch updated user: {:?}. UUID: {}", e, uuid);
                            let msg = state.i18n.t("general.db_error", &locale).await;
                            into_api_response(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                None,
                                Some(error_map(&"database".to_string(), &msg.clone())),
                                Some(vec![msg]),
                            )
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[UpdateUser] Database error: {:?}. UUID: {}, Payload: {:?}", e, uuid, payload);
                    let msg = state.i18n.t("general.db_error", &locale).await;
                    into_api_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        None,
                        Some(error_map(&"database".to_string(), &msg.clone())),
                        Some(vec![msg]),
                    )
                }
            }
        }
        Ok(None) => {
            let msg = state.i18n.t("user.not_found", &locale).await;
            into_api_response(
                StatusCode::NOT_FOUND,
                None,
                Some(error_map(&"user".to_string(), &msg.clone())),
                Some(vec![msg]),
            )
        }
        Err(e) => {
            println!("Error: {:?}", e);
            let msg = state.i18n.t("general.db_error", &locale).await;
            into_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                Some(error_map(&"database".to_string(), &msg.clone())),
                Some(vec![msg]),
            )
        }
    }
}

// TODO: Add protected routes
pub fn public_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all).post(create))
        .route("/{uuid}", get(get_one))
}

pub fn protected_router() -> Router<Arc<AppState>> {
    Router::new().route("/{uuid}", axum::routing::delete(delete_one).patch(update))
}
