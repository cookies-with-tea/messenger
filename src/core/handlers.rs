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
    "SELECT m.url, m.alt, m.title, m.media_type::text as media_type
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
      media_type: row.try_get("media_type").ok(),
    });

  Ok(row)
}


pub async fn get_media_by_uuid(
  state: &Arc<AppState>,
  media_uuid: Option<Uuid>
) -> Result<Option<MediaDTO>, sqlx::Error> {
  if let Some(uuid) = media_uuid {
    let query = "SELECT url, alt, title, media_type::text as media_type FROM media WHERE uuid = $1 LIMIT 1";

    let row = sqlx::query(query)
      .bind(uuid)
      .fetch_optional(&state.pool)
      .await?
      .map(|row| MediaDTO {
        url: row.get("url"),
        alt: row.get("alt"),
        title: row.get("title"),
        media_type: row.try_get("media_type").ok(),
      });

    Ok(row)
  } else {
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
        "SELECT uuid, url, alt, title, media_type::text as media_type FROM media WHERE uuid = ANY($1)"
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
            media_type: row.try_get("media_type").ok(),
        });
    }

    Ok(map)
}
