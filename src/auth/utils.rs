use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use axum::http::StatusCode;

// Структура, которая соответствует payload вашего JWT
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // Обычно здесь user_id (uuid как строка)
    exp: usize,
}

pub fn get_user_from_token(token: &str) -> Result<Uuid, StatusCode> {
    // Ключ для проверки подписи (должен быть тем же, что и при генерации токена)
    // Лучше хранить его в AppState и доставать оттуда, но для примера хардкод или env
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true; // Проверять срок действия

    match decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation) {
        Ok(token_data) => {
            // Парсим UUID из строки sub
            Uuid::parse_str(&token_data.claims.sub)
                .map_err(|_| StatusCode::UNAUTHORIZED) // Если не валидный UUID
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED), // Токен невалиден или истек
    }
}
