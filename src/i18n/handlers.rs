use crate::{AppState, core::{dto::ApiResponse, response::into_api_response}, i18n::dto::{CreateTranslationDTO, TranslationDTO}};
use axum::{
    Extension, Json, Router, extract::{Path, State}, http::StatusCode, routing::{delete, get, post}
};
use std::{collections::HashMap, sync::Arc};

#[utoipa::path(
    post,
    path = "/api/v1/i18n",
    request_body = CreateTranslationDTO,
    responses(
        (status = 201, description = "Translation created or updated"),
        (status = 500, description = "Database error")
    ),
    tag = "I18n"
)]
pub async fn create_or_update(
    State(state): State<Arc<AppState>>,
    Json(dto): Json<CreateTranslationDTO>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let result = sqlx::query(
        r#"
        INSERT INTO i18n_translations (key, locale, value)
        VALUES ($1, $2, $3)
        ON CONFLICT (key, locale)
        DO UPDATE SET value = $3, updated_at = NOW()
        "#,
    )
    .bind(&dto.key)
    .bind(&dto.locale)
    .bind(&dto.value)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            state.i18n.clear_cache();
            into_api_response(
                StatusCode::CREATED,
                None,
                None,
                Some(vec!["Translation saved".to_string()]),
            )
        }
        Err(e) => {
            eprintln!("DB error: {}", e);
            into_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                None,
                Some(vec!["Failed to save translation".to_string()]),
            )
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/i18n",
    params(
        ("locale" = String, Query, description = "Locale code, e.g. 'en', 'ru'")
    ),
    responses(
        (status = 200, description = "Translations loaded", body = ApiResponse<Vec<TranslationDTO>>),
        (status = 500, description = "Database error", body = ApiResponse<Vec<TranslationDTO>>)
    ),
    tag = "I18n"
)]
pub async fn get_all(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<
    Json<ApiResponse<Vec<TranslationDTO>>>,
    (StatusCode, Json<ApiResponse<Vec<TranslationDTO>>>),
> {
    let default_locale = "en".to_string();
    let locale = params.get("locale").unwrap_or(&default_locale);

    match sqlx::query_as::<_, TranslationDTO>(
        "SELECT id, key, locale, value, created_at, updated_at FROM i18n_translations WHERE locale = $1"
    )
    .bind(locale)
    .fetch_all(&state.pool)
    .await
    {
        Ok(translations) => into_api_response(StatusCode::OK, Some(translations), None, None),
        Err(e) => {
            eprintln!("DB error: {}", e);
            into_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                None,
                Some(vec!["Failed to load translations".to_string()]),
            )
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/i18n/{key}/{locale}",
    params(
        ("key" = String, Path, description = "Translation key"),
        ("locale" = String, Path, description = "Locale code")
    ),
    responses(
        (status = 200, description = "Translation deleted"),
        (status = 500, description = "Database error")
    ),
    tag = "I18n"
)]
pub async fn delete_one(
    State(state): State<Arc<AppState>>,
    Path((key, locale)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let result = sqlx::query("DELETE FROM i18n_translations WHERE key = $1 AND locale = $2")
        .bind(&key)
        .bind(&locale)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => {
            state.i18n.clear_cache();
            into_api_response(
                StatusCode::OK,
                None,
                None,
                Some(vec!["Translation deleted".to_string()]),
            )
        }
        Err(e) => {
            eprintln!("DB error: {}", e);
            into_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                None,
                Some(vec!["Failed to delete translation".to_string()]),
            )
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/i18n/{dict_key}",
    params(
        ("dict_key" = String, Path, description = "Dictionary key prefix, e.g. 'user', 'general'")
    ),
    responses(
        (status = 200, description = "Translations loaded", body = Object),
        (status = 500, description = "Database error")
    ),
    tag = "I18n"
)]
pub async fn get_by_dict_key(
    State(state): State<Arc<AppState>>,
    Extension(locale): Extension<String>,
    Path(dict_key): Path<String>,
) -> Result<Json<HashMap<String, String>>, (StatusCode, Json<ApiResponse<()>>)> {
    let pattern = format!("{}.%", dict_key);

    let rows: Result<Vec<(String, String)>, _> = sqlx::query_as::<_, (String, String)>(
        "SELECT key, value FROM i18n_translations WHERE locale = $1 AND key LIKE $2",
    )
    .bind(&locale)
    .bind(&pattern)
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(translations) => {
            let map: HashMap<String, String> = translations.into_iter().collect();
            Ok(Json(map))
        }
        Err(e) => {
            eprintln!("DB error in get_by_dict_key: {}", e);
            let msg = state.i18n.t("general.db_error", &locale).await;
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                into_api_response_internal(msg),
            ))
        }
    }
}

fn into_api_response_internal(message: String) -> Json<ApiResponse<()>> {
    Json(ApiResponse {
        data: None,
        errors: None,
        messages: Some(vec![message]),
    })
}

pub fn public_router() -> Router<Arc<AppState>> {
    Router::new().route("/{dict_key}", get(get_by_dict_key))
}

pub fn protected_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_or_update))
        .route("/", get(get_all))
        .route("/{key}/{locale}", delete(delete_one))
}
