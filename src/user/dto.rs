use chrono::{NaiveDate, NaiveDateTime};
use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use sqlx::FromRow;

#[derive(sqlx::Type, Debug, Serialize, Deserialize, ToSchema)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    User,
}

#[derive(sqlx::Type, Debug, Serialize, Deserialize, ToSchema)]
#[sqlx(type_name = "user_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Inactive,
    InModeration,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct CreateUserDTO {
    pub(crate) email: String,
    pub(crate) password: String,
    pub(crate) phone: Option<String>,
    pub(crate) first_name: Option<String>,
    pub(crate) second_name: Option<String>,
    pub(crate) last_name: Option<String>,
    pub(crate) birth_date: Option<NaiveDate>,
    pub(crate) role: Option<UserRole>,
    pub(crate) status: Option<UserStatus>,
    pub(crate) avatar: Option<String>,
    pub(crate) street: Option<String>,
    pub(crate) city: Option<String>,
    pub(crate) gender: Option<String>,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct UpdateUserDTO {
  pub(crate) email: Option<String>,
  pub(crate) phone: Option<String>,
  pub(crate) first_name: Option<String>,
  pub(crate) second_name: Option<String>,
  pub(crate) last_name: Option<String>,
  pub(crate) birth_date: Option<NaiveDate>,
  pub(crate) role: Option<UserRole>,
  pub(crate) status: Option<UserStatus>,
  pub(crate) avatar: Option<String>,
  pub(crate) street: Option<String>,
  pub(crate) city: Option<String>,
  pub(crate) gender: Option<String>,
}

#[derive(Serialize, Debug, ToSchema, FromRow)]
pub struct UserResponseDTO {
  pub(crate) uuid: Uuid,
  pub(crate) first_name: Option<String>,
  pub(crate) second_name: Option<String>,
  pub(crate) last_name: Option<String>,
  pub(crate) phone: Option<String>,
  pub(crate) email: String,
  pub(crate) birth_date: Option<NaiveDate>,
  pub(crate) avatar: Option<String>,
  pub(crate) street: Option<String>,
  pub(crate) gender: Option<String>,
  pub(crate) city: Option<String>,
  pub(crate) role: UserRole,
  pub(crate) status: UserStatus,
  pub(crate) created_at: NaiveDateTime,
  pub(crate) updated_at: NaiveDateTime,
}

#[derive(Serialize, Debug, sqlx::FromRow, ToSchema)]
pub struct User {
  pub(crate) uuid: Uuid,
  pub(crate) first_name: Option<String>,
  pub(crate) second_name: Option<String>,
  pub(crate) last_name: Option<String>,
  pub(crate) phone: Option<String>,
  pub(crate) email: String,
  pub(crate) password_hash: String,
  pub(crate) birth_date: Option<NaiveDate>,
  pub(crate) avatar: Option<String>,
  pub(crate) street: Option<String>,
  pub(crate) gender: Option<String>,
  pub(crate) city: Option<String>,
  pub(crate) role: UserRole,
  pub(crate) status: UserStatus,
  pub(crate) created_at: NaiveDateTime,
  pub(crate) updated_at: NaiveDateTime,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct UserSearchQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub search: Option<String>,
}
