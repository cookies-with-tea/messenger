use serde_derive::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, Debug, ToSchema)]
pub struct PaginationQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct ApiResponse<T: serde::Serialize> {
    pub(crate) data: Option<T>,
    pub(crate) errors: Option<HashMap<String, Vec<String>>>,
    pub(crate) messages: Option<Vec<String>>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct ApiPaginationDTO<T: serde::Serialize> {
  pub items: Vec<T>,
  pub pagination: PaginationDTO,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct ApiResponseWithPagination<T: serde::Serialize> {
  pub(crate) data: Option<ApiPaginationDTO<T>>,
  pub(crate) errors: Option<HashMap<String, Vec<String>>>,
  pub(crate) messages: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, FromRow, ToSchema)]
pub struct PaginationDTO {
    pub page: i32,                // default - 1
    pub total: Option<i32>,       // default - 0
    pub total_pages: Option<i32>, // default - 0
    pub limit: Option<i32>,       // default - 10
}

#[derive(Serialize, Deserialize, Debug, FromRow, ToSchema)]
pub struct IconTextDTO {
    pub title: Option<String>,
    pub icon: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, FromRow, ToSchema, Clone)]
pub struct MediaDTO {
    pub(crate) url: String,
    pub(crate) alt: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) media_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct AttractionRowDTO {
    pub icon: Option<String>,
    pub text: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, FromRow, ToSchema)]
pub struct AttractionDTO {
    pub(crate) uuid: Uuid,
    pub(crate) title: String,
    pub(crate) subtitle: Option<String>,
    pub(crate) items: Vec<IconTextDTO>,
}
