use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::fs;
use std::path::Path;

pub type DbPool = SqlitePool;

const DB_DIR: &str = "data";
const DB_FILE: &str = "pcloud.db";

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    // Create the database directory if it doesn't exist
    if !Path::new(DB_DIR).exists() {
        fs::create_dir_all(DB_DIR).expect("Failed to create database directory");
    }

    let db_path = Path::new(DB_DIR).join(DB_FILE);
    let db_url = db_path.to_str().unwrap();

    // Create the database file if it doesn't exist
    if !Path::new(db_url).exists() {
        fs::File::create(db_url).expect("Failed to create database file");
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
