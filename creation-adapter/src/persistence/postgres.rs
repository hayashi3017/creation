use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::config::Config;

// FIXME: pub(crate) not pub
#[derive(Clone)]
pub struct Db(pub Pool<Postgres>);

impl Db {
    pub async fn new() -> Self {
        let config = Config::init();
        let pool = match PgPoolOptions::new()
            .max_connections(10)
            .connect(&config.database_url)
            .await
        {
            Ok(pool) => {
                println!("✅Connection to the database is successful!");
                pool
            }
            Err(err) => {
                println!("🔥 Failed to connect to the database: {:?}", err);
                std::process::exit(1);
            }
        };

        Db(pool)
    }
    pub async fn new_test(pool: Pool<Postgres>) -> Self {
        Db(pool)
    }
}
