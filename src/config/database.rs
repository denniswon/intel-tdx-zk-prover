use async_trait::async_trait;
use sqlx::{Error, Postgres, Pool};

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing::info;

#[derive(Clone)]
pub struct Database {
    pool: Pool<Postgres>,
}

#[async_trait]
pub trait DatabaseTrait {
    async fn init() -> Result<Self, Error>
        where
            Self: Sized;
    fn get_pool(&self) -> &Pool<Postgres>;
}

#[async_trait]
impl DatabaseTrait for Database {
    async fn init() -> Result<Self, Error> {
        dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new().connect(&url).await?;
        info!("Connected to the database!");
        Ok(Self { pool })
    }

    fn get_pool(&self) -> &Pool<Postgres> {
        &self.pool
    }
}
