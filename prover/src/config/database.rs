use async_trait::async_trait;
use tracing::info;

use crate::config::parameter;
use crate::config::pool::{DbPool, Pool};

#[derive(Clone)]
pub struct Database {
    pool: Pool,
}

#[async_trait]
pub trait DatabaseTrait {
    async fn init() -> Result<Self, anyhow::Error>
        where
            Self: Sized;
    fn get_pool(&self) -> &Pool;
}

const CONNECTION_COUNT: usize = 14;

#[async_trait]
impl DatabaseTrait for Database {
    async fn init() -> Result<Self, anyhow::Error> {
        parameter::init();
        let opts = parameter::get("DATABASE_URL", None).parse().unwrap();
        let pool = DbPool::new(opts, CONNECTION_COUNT).unwrap();
        info!("Connected to the database!");
        Ok(Self { pool })
    }

    fn get_pool(&self) -> &Pool {
        &self.pool
    }
}
