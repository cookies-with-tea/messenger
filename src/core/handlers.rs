use sqlx::{PgPool, Row};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;
use crate::{AppState, core::dto::MediaDTO};

pub async fn get_media(
  state: &Arc<AppState>,
  table_name: &str,
  entity_uuid_column: &str,
  entity_uuid: &Uuid,
) -> Result<Option<MediaDTO>, sqlx::Error> {
  let query = format!(
    "SELECT m.url, m.alt, m.title
         FROM {} em
         JOIN media m ON em.media_uuid = m.uuid
         WHERE em.{} = $1
         LIMIT 1",
    table_name, entity_uuid_column
  );

  let row = sqlx::query(&query)
    .bind(entity_uuid)
    .fetch_optional(&state.pool)
    .await?
    .map(|row| MediaDTO {
      url: row.get("url"),
      alt: row.get("alt"),
      title: row.get("title"),
    });

  Ok(row)
}


pub async fn get_media_by_uuid(
  state: &Arc<AppState>,
  media_uuid: Option<Uuid>
) -> Result<Option<MediaDTO>, sqlx::Error> {
  // Если UUID не передан (None), сразу возвращаем None
  if let Some(uuid) = media_uuid {
    // Запрос для получения медиа-данных по UUID
    let query = "SELECT url, alt, title FROM media WHERE uuid = $1 LIMIT 1";

    // Выполнение запроса
    let row = sqlx::query(query)
      .bind(uuid)
      .fetch_optional(&state.pool)
      .await?
      .map(|row| MediaDTO {
        url: row.get("url"),
        alt: row.get("alt"),
        title: row.get("title"),
      });

    Ok(row)
  } else {
    // Если UUID отсутствует, возвращаем None
    Ok(None)
  }
}

pub async fn get_media_by_uuids(
    pool: &PgPool,
    uuids: Vec<Uuid>,
) -> Result<HashMap<Uuid, MediaDTO>, sqlx::Error> {
    if uuids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query(
        "SELECT uuid, url, alt, title FROM media WHERE uuid = ANY($1)"
    )
    .bind(&uuids)
    .fetch_all(pool)
    .await?;

    let mut map = HashMap::with_capacity(rows.len());
    for row in rows {
        let uuid: Uuid = row.get("uuid");
        map.insert(uuid, MediaDTO {
            url: row.get("url"),
            alt: row.get("alt"),
            title: row.get("title"),
        });
    }

    Ok(map)
}
