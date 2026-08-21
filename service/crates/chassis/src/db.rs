use sqlx::{migrate::Migrator, Pool, Postgres};

pub type DbPool = Pool<Postgres>;

pub async fn connect(database_url: &str) -> Result<DbPool, sqlx::Error> {
    Pool::connect(database_url).await
}

pub async fn migrate(pool: &DbPool) -> Result<(), sqlx::migrate::MigrateError> {
    static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");
    MIGRATOR.run(pool).await
}
