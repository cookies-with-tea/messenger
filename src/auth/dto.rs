use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, Debug, ToSchema)]
pub struct RegisterRequestDTO {
    pub email: String,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct CheckEmailCodeDTO {
    pub key: String,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct AuthRequestDTO {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct AuthResponseDTO {
    pub access_token: String,
    pub access_expires_in: i64,
    pub refresh_token: String,
    pub refresh_expires_in: i64,
}
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct AuthRefreshTokenDTO {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
}
