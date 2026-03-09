use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;


#[derive(Deserialize, ToSchema)]
pub struct CreateTranslationDTO {
    pub key: String,
    pub locale: String,
    pub value: String,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct TranslationDTO {
    id: uuid::Uuid,
    key: String,
    locale: String,
    value: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}
