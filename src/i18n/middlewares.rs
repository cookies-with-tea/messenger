use axum::http::header::ACCEPT_LANGUAGE;
use axum::extract::Request;
use axum::middleware::{Next};
use axum::response::Response;

pub async fn locale_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, Response> {
    let accept_language = req
        .headers()
        .get(ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .map(|s| {
            s.split(|c| c == ',' || c == ';')
                .next()
                .unwrap_or("en")
                .trim()
                .to_lowercase()
        })
        .unwrap_or_else(|| "en".to_string());

    req.extensions_mut().insert(accept_language);
    Ok(next.run(req).await)
}
