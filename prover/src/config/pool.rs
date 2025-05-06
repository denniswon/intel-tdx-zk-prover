// Adapted from performance-service by tsunyoku
// https://github.com/osuAkatsuki/performance-service/blob/9d40594d7645d38d1bde167fdadedb89cb4b4772/src/models/pool.rs
// Used under MIT license

use deadpool::managed::{Manager, Metrics, RecycleResult};
use sqlx::postgres::PgConnectOptions;
use sqlx::{ConnectOptions, Connection, Error as SqlxError, PgConnection};

#[derive(Clone, Debug)]
pub struct DbPool {
    options: PgConnectOptions,
}

impl DbPool {
    pub fn new(options: PgConnectOptions, max_size: usize) -> anyhow::Result<Pool> {
        Ok(Pool::builder(Self { options }).max_size(max_size).build()?)
    }
}

impl Manager for DbPool {
    type Type = PgConnection;
    type Error = SqlxError;

    async fn create(&self) -> Result<PgConnection, SqlxError> {
        self.options.connect().await
    }

    async fn recycle(&self, obj: &mut Self::Type, _: &Metrics) -> RecycleResult<SqlxError> {
        Ok(obj.ping().await?)
    }
}

pub type Pool = deadpool::managed::Pool<DbPool>;

#[macro_export]
macro_rules! get_conn {
    ($input:expr) => {{
        use std::ops::DerefMut;
        $input.get().await.unwrap().deref_mut()
    }};
}
#[allow(unused_imports)]
pub(crate) use get_conn;
