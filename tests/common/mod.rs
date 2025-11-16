use sqlx::{Sqlite, Pool};
use sqlx::sqlite::SqlitePoolOptions;

pub async fn setup_test_db() -> Pool<Sqlite> {
    // Create an in-memory SQLite database for testing
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database pool.");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations.");

    pool
}
