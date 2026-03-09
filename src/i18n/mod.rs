pub mod handlers;
pub mod middlewares;
mod dto;

use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct I18nService {
    pool: Pool<Postgres>,
    cache: Arc<RwLock<HashMap<(String, String), String>>>,
}

impl I18nService {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            pool,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn t(&self, key: &str, locale: &str) -> String {
        let cache_key = (key.to_string(), locale.to_string());

        if let Some(value) = self.cache.read().unwrap().get(&cache_key) {
            return value.clone();
        }

        let row: Option<(String,)> = sqlx::query_as(
            "SELECT value FROM i18n_translations WHERE key = $1 AND locale = $2"
        )
        .bind(key)
        .bind(locale)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

        let value = match row {
            Some((val,)) => val,
            None => {
                if locale != "en" {
                    if let Some((val,)) = sqlx::query_as::<_, (String,)>(
                        "SELECT value FROM i18n_translations WHERE key = $1 AND locale = 'en'"
                    )
                    .bind(key)
                    .fetch_optional(&self.pool)
                    .await
                    .ok()
                    .flatten() {
                        val
                    } else {
                        format!("missing:{}.{}", locale, key)
                    }
                } else {
                    format!("missing:{}.{}", locale, key)
                }
            }
        };

        self.cache.write().unwrap().insert(cache_key, value.clone());
        value
    }

    pub fn clear_cache(&self) {
        self.cache.write().unwrap().clear();
    }
}
