use crate::core::app::AppConfig;
use crate::core::dto::{ApiResponse, ApiResponseWithPagination, ApiPaginationDTO, PaginationDTO, PaginationQuery};
use crate::media::dto::{
  CreateMediaDTO,
  MediaItemDTO,
  MediaItemFromDb,
  MediaUploadResponseDTO,
  UpdateMediaDTO,
};
use crate::core::response::{error_map, into_api_response, into_api_response_with_pagination};
use crate::AppState;
use axum::Router;
use axum::routing::{delete, get, post, put};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    Json,
};
use std::{fs, path::PathBuf, sync::Arc};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/v1/media",
    tag = "Media",
    request_body(content = CreateMediaDTO, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Media uploaded successfully", body = ApiResponse<MediaUploadResponseDTO>),
        (status = 400, description = "Invalid file or content type", body = ApiResponse<MediaUploadResponseDTO>),
        (status = 500, description = "Internal server error", body = ApiResponse<MediaUploadResponseDTO>)
    ),
    operation_id = "upload_media",
)]
pub async fn create(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<MediaUploadResponseDTO>>, (StatusCode, Json<ApiResponse<MediaUploadResponseDTO>>)> {
    let mut file_name = String::new();
    let mut data = Vec::new();
    let mut content_type = None;
    let mut title = None;
    let mut alt = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        if field.name() == Some("file") {
            file_name = field
                .file_name()
                .map(|f| f.to_string())
                .unwrap_or_else(|| "file".to_string());
            data = field.bytes().await.unwrap().to_vec();
            content_type = mime_guess::from_path(&file_name)
                .first()
                .map(|mime| mime.to_string());
        } else if field.name() == Some("title") {
            title = Some(field.text().await.unwrap());
        } else if field.name() == Some("alt") {
            alt = Some(field.text().await.unwrap());
        }
    }

    let content_type = match content_type {
        Some(ct) if !ct.is_empty() => ct,
        _ => {
            return into_api_response(
                StatusCode::BAD_REQUEST,
                None,
                Some(error_map("content_type", "Unable to determine content type from file")),
                Some(vec!["Failed to upload media".to_string()]),
            );
        }
    };

    let media_type = if content_type.starts_with("image/") {
        "image"
    } else if content_type.starts_with("video/") {
        "video"
    } else {
        return into_api_response(
            StatusCode::BAD_REQUEST,
            None,
            Some(error_map("content_type", "Unsupported media type")),
            Some(vec!["Only images and videos are allowed".to_string()]),
        );
    };

    let uuid = Uuid::new_v4();
    let path_buf = PathBuf::from(&file_name);
    let extension = path_buf
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("bin");

    let relative_path = format!("media/{}/{}.{}", media_type, uuid, extension);
    let save_path = PathBuf::from(&relative_path);

    if let Err(e) = fs::create_dir_all(save_path.parent().unwrap()) {
        eprintln!("FS error (mkdir): {:?}", e);
        return into_api_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            None,
            Some(error_map("filesystem", "Failed to create upload directory")),
            Some(vec!["Server configuration error".to_string()]),
        );
    }

    if let Err(e) = fs::write(&save_path, &data) {
        eprintln!("FS error (write): {:?}", e);
        return into_api_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            None,
            Some(error_map("filesystem", "Failed to save file")),
            Some(vec!["Could not write file to disk".to_string()]),
        );
    }

    let config = AppConfig::new();
    let full_url = format!(
        "{}/{}",
        config.public_url.trim_end_matches('/'),
        &relative_path
    );

    let db_result = sqlx::query(
        "INSERT INTO media (uuid, media_type, url, title, alt) VALUES ($1, $2::media_type, $3, $4, $5)",
    )
    .bind(uuid)
    .bind(media_type)
    .bind(&full_url)
    .bind(title)
    .bind(alt)
    .execute(&state.pool)
    .await;

    if let Err(e) = db_result {
        eprintln!("DB error: {:?}", e);
        // удалить файл, если запись в БД не удалась
        let _ = fs::remove_file(&save_path);
        return into_api_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            None,
            Some(error_map("database", "Failed to save media record")),
            Some(vec!["Could not register file in database".to_string()]),
        );
    }

    into_api_response(
        StatusCode::CREATED,
        Some(MediaUploadResponseDTO {
            uuid: uuid.to_string(),
            url: full_url,
        }),
        None,
        Some(vec!["Media uploaded successfully".to_string()]),
    )
}

#[utoipa::path(
    get,
    path = "/api/v1/media",
    tag = "Media",
    params(
        ("page" = i32, Query, description = "Page number"),
        ("limit" = i32, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List of media with pagination", body = ApiResponseWithPagination<MediaItemDTO>),
        (status = 500, description = "Internal server error", body = ApiResponseWithPagination<MediaItemDTO>)
    ),
    operation_id = "get_all_media"
)]
pub async fn get_all(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ApiResponseWithPagination<MediaItemDTO>>, (StatusCode, Json<ApiResponseWithPagination<MediaItemDTO>>)> {
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(10);
    let offset = (page - 1) * limit;

    let total_query = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM media")
        .fetch_one(&state.pool)
        .await;

    let total = match total_query {
        Ok(count) => count,
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            return into_api_response_with_pagination(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                Some(error_map("database", "Failed to fetch media count")),
                Some(vec!["Could not retrieve media count".to_string()]),
            );
        }
    };

    let total_pages = (total as f64 / limit as f64).ceil() as i32;

    let rows = sqlx::query_as::<_, MediaItemFromDb>(
        r#"
        SELECT uuid, media_type, url, title, alt
        FROM media
        ORDER BY uuid DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(db_items) => {
            let media_list = db_items
                .into_iter()
                .map(|row| MediaItemDTO {
                    uuid: row.uuid.to_string(),
                    url: row.url,
                    title: row.title,
                    alt: row.alt,
                    media_type: row.media_type,
                })
                .collect();

            let pagination = PaginationDTO {
                page,
                total: Some(total as i32),
                total_pages: Some(total_pages),
                limit: Some(limit),
            };

            let api_pagination = ApiPaginationDTO {
                items: media_list,
                pagination: pagination,
            };

            into_api_response_with_pagination(
                StatusCode::OK,
                Some(api_pagination),
                None,
                Some(vec!["Media fetched successfully".to_string()]),
            )
        }
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            into_api_response_with_pagination(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                Some(error_map("database", "Failed to fetch media list")),
                Some(vec!["Could not retrieve media records".to_string()]),
            )
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/media/{uuid}",
    tag = "Media",
    params(
        ("uuid" = String, Path, description = "Media UUID")
    ),
    responses(
        (status = 200, description = "Media item", body = ApiResponse<MediaItemDTO>),
        (status = 404, description = "Media not found", body = ApiResponse<MediaItemDTO>),
        (status = 500, description = "Internal server error", body = ApiResponse<MediaItemDTO>)
    ),
    operation_id = "get_one_media"
)]
pub async fn get_one(
    State(state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
) -> Result<Json<ApiResponse<MediaItemDTO>>, (StatusCode, Json<ApiResponse<MediaItemDTO>>)> {
    let result = sqlx::query_as::<_, MediaItemFromDb>(
        "SELECT uuid, media_type, url, title, alt FROM media WHERE uuid = $1"
    )
    .bind(uuid)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(media)) => {
            let media_response = MediaItemDTO {
                uuid: media.uuid.to_string(),
                url: media.url,
                title: media.title,
                alt: media.alt,
                media_type: media.media_type,
            };

            into_api_response(
                StatusCode::OK,
                Some(media_response),
                None,
                Some(vec!["Media fetched successfully".to_string()]),
            )
        }
        Ok(None) => {
            into_api_response(
                StatusCode::NOT_FOUND,
                None,
                Some(error_map("media", "Media not found")),
                Some(vec!["Media not found".to_string()]),
            )
        }
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            into_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                Some(error_map("database", "Failed to fetch media")),
                Some(vec!["Could not retrieve media".to_string()]),
            )
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/media/{uuid}",
    tag = "Media",
    params(
        ("uuid" = Uuid, Path, description = "Media UUID")
    ),
    request_body(content = UpdateMediaDTO, content_type = "application/json"),
    responses(
        (status = 200, description = "Media updated successfully", body = ApiResponse<MediaItemDTO>),
        (status = 400, description = "No fields to update", body = ApiResponse<MediaItemDTO>),
        (status = 404, description = "Media not found", body = ApiResponse<MediaItemDTO>),
        (status = 500, description = "Internal server error", body = ApiResponse<MediaItemDTO>)
    ),
    operation_id = "update_media"
)]
pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Json(payload): Json<UpdateMediaDTO>,
) -> Result<Json<ApiResponse<MediaItemDTO>>, (StatusCode, Json<ApiResponse<MediaItemDTO>>)> {
    let existing_media = sqlx::query_as::<_, MediaItemFromDb>(
        "SELECT uuid, media_type, url, title, alt FROM media WHERE uuid = $1"
    )
    .bind(uuid)
    .fetch_optional(&state.pool)
    .await;

    match existing_media {
        Ok(Some(_)) => {
            let mut update_query = "UPDATE media SET ".to_string();
            let mut query_param_index = 1;
            let mut has_updates = false;

            if let Some(_title) = &payload.title {
                update_query.push_str(&format!("title = ${}, ", query_param_index));
                query_param_index += 1;
                has_updates = true;
            }

            if let Some(_alt) = &payload.alt {
                update_query.push_str(&format!("alt = ${}, ", query_param_index));
                query_param_index += 1;
                has_updates = true;
            }

            if !has_updates {
                return into_api_response(
                    StatusCode::BAD_REQUEST,
                    None,
                    Some(error_map("update", "No fields to update")),
                    Some(vec!["No fields provided for update".to_string()]),
                );
            }

            update_query.push_str(&format!("WHERE uuid = ${}", query_param_index));

            let mut query = sqlx::query(&update_query);

            if let Some(title) = &payload.title {
                query = query.bind(title.clone());
            }

            if let Some(alt) = &payload.alt {
                query = query.bind(alt.clone());
            }

            query = query.bind(uuid);

            let result = query.execute(&state.pool).await;

            match result {
                Ok(_) => {
                    let updated_media = sqlx::query_as::<_, MediaItemFromDb>(
                        "SELECT uuid, media_type, url, title, alt FROM media WHERE uuid = $1"
                    )
                    .bind(uuid)
                    .fetch_one(&state.pool)
                    .await;

                    match updated_media {
                        Ok(media) => {
                            let media_response = MediaItemDTO {
                                uuid: media.uuid.to_string(),
                                url: media.url,
                                title: media.title,
                                alt: media.alt,
                                media_type: media.media_type,
                            };

                            into_api_response(
                                StatusCode::OK,
                                Some(media_response),
                                None,
                                Some(vec!["Media updated successfully".to_string()]),
                            )
                        }
                        Err(e) => {
                            eprintln!("DB error: {:?}", e);
                            into_api_response(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                None,
                                Some(error_map("database", "Failed to fetch updated media")),
                                Some(vec!["Could not retrieve updated media".to_string()]),
                            )
                        }
                    }
                }
                Err(e) => {
                    eprintln!("DB error: {:?}", e);
                    into_api_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        None,
                        Some(error_map("database", "Failed to update media")),
                        Some(vec!["Could not update media".to_string()]),
                    )
                }
            }
        }
        Ok(None) => {
            into_api_response(
                StatusCode::NOT_FOUND,
                None,
                Some(error_map("media", "Media not found")),
                Some(vec!["Media not found".to_string()]),
            )
        }
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            into_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                Some(error_map("database", "Failed to check media existence")),
                Some(vec!["Could not check if media exists".to_string()]),
            )
        }
    }
}


pub async fn delete_one(
    State(state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM media WHERE uuid = $1)")
        .bind(uuid)
        .fetch_one(&state.pool)
        .await;

    match exists {
        Ok(true) => {
            let result = sqlx::query("DELETE FROM media WHERE uuid = $1")
                .bind(uuid)
                .execute(&state.pool)
                .await;
            match result {
                Ok(_) => {
                    into_api_response(
                        StatusCode::OK,
                        None,
                        None,
                        Some(vec!["Media deleted successfully".to_string()]),
                    )
                }
                Err(e) => {
                    eprintln!("DB error: {:?}", e);
                    into_api_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        None,
                        Some(error_map("database", "Failed to delete media")),
                        Some(vec!["Could not delete media".to_string()]),
                    )
                }
            }
        }
        Ok(false) => {
            into_api_response(
                StatusCode::NOT_FOUND,
                None,
                Some(error_map("media", "Media not found")),
                Some(vec!["Media not found".to_string()]),
            )
        }
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            into_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                Some(error_map("database", "Failed to check media existence")),
                Some(vec!["Could not check if media exists".to_string()]),
            )
        }
    }
}


pub async fn delete_all(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let result = sqlx::query("DELETE FROM media")
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => {
            into_api_response(
                StatusCode::OK,
                None,
                None,
                Some(vec!["All media deleted successfully".to_string()]),
            )
        }
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            into_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                Some(error_map("database", "Failed to delete all media")),
                Some(vec!["Could not delete all media".to_string()]),
            )
        }
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create))
        .route("/", get(get_all))
        .route("/{uuid}", get(get_one))
        .route("/{uuid}", put(update))
        .route("/{uuid}", delete(delete_one))
        .route("/", delete(delete_all))
}
