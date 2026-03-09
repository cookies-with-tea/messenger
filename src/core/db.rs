use crate::core::app::AppConfig;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub db_engine: String,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
    pub db_host: String,
}

impl DatabaseConfig {
    pub fn new() -> Self {
        let db_engine = env::var("APP_DB_ENGINE").expect("APP_DB_ENGINE must be set");
        let db_name = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");
        let db_user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
        let db_password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
        let db_host = env::var("POSTGRES_HOST").expect("POSTGRES_HOST must be set");

        DatabaseConfig {
            db_engine,
            db_name,
            db_user,
            db_password,
            db_host,
        }
    }

    pub fn connect_url(&self) -> String {
        format!(
            "{}://{}:{}@{}/{}",
            self.db_engine, self.db_user, self.db_password, self.db_host, self.db_name
        )
    }
}

pub async fn create_pool(config: &AppConfig) -> sqlx::Pool<sqlx::Postgres> {
    let connect_url = config.db_config.connect_url();

    println!("Connecting to database: {}", connect_url);

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&connect_url)
        .await
        .expect("Error")
}
