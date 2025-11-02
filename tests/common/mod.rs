use app::database::DbPool;
use sqlx::sqlite::SqlitePoolOptions;

pub async fn setup_test_db() -> DbPool {
    // Use in-memory SQLite database for tests
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database pool.");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations.");

    pool
}
