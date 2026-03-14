use crate::{AppState, core::response::{error_map, into_api_response}};
use axum::{
    body::Body,
    extract::{Request, State},
    http::{self, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let locale = request
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_else(|| "en".to_string());

    let auth_header = request
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = if let Some(header_value) = auth_header {
        if header_value.starts_with("Bearer ") {
            header_value[7..].to_string()
        } else {
            return Err(create_unauthorized_response(&state, &locale).await);
        }
    } else {
        // Try query param for WebSockets
        let query = request.uri().query().unwrap_or("");
        let token_param = query.split('&')
            .find(|part| part.starts_with("token="))
            .map(|part| part[6..].to_string());
        
        if let Some(t) = token_param {
            t
        } else {
            return Err(create_unauthorized_response(&state, &locale).await);
        }
    };

    let secret = match std::env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            eprintln!("JWT_SECRET environment variable not set");
            return Err(
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            );
        }
    };

    use jsonwebtoken::{decode, DecodingKey, Validation};
    let token_data = match decode::<crate::auth::dto::Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    ) {
        Ok(data) => data,
        Err(_) => return Err(create_unauthorized_response(&state, &locale).await),
    };

    let user_id = token_data.claims.sub;

    let pool = &state.pool;
    let result = match sqlx::query(
        "SELECT user_id FROM refresh_token WHERE user_id = $1 AND expires_at > NOW()",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Database error in auth_middleware: {}", e);
            return Err(
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            );
        }
    };

    if result.is_none() {
        return Err(create_unauthorized_response(&state, &locale).await);
    }

    request.extensions_mut().insert(user_id);

    let response = next.run(request).await;
    Ok(response)
}

async fn create_unauthorized_response(state: &Arc<AppState>, locale: &str) -> Response {
    let message = state.i18n.t("auth.unauthorized", locale).await;
    let error_detail = error_map("auth", &message);
    let api_response = into_api_response::<()>(
        StatusCode::UNAUTHORIZED,
        None,
        Some(error_detail),
        Some(vec![message]),
    );
    api_response.into_response()
}
