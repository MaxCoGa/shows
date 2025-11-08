use sqlx::sqlite::{SqlitePool, SqliteConnectOptions, SqlitePoolOptions};
use std::fs;
use std::path::Path;
use std::str::FromStr;

pub type DbPool = SqlitePool;

const DB_DIR: &str = "data";
const DB_FILE: &str = "pcloud.db";

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    // Create the database directory if it doesn't exist
    if !Path::new(DB_DIR).exists() {
        fs::create_dir_all(DB_DIR).expect("Failed to create database directory");
    }

    let db_path = Path::new(DB_DIR).join(DB_FILE);
    let db_path_str = db_path.to_str().expect("Failed to get db path string");

    println!("Attempting to connect to database at: {}", db_path_str);

    // Explicitly configure the connection to create the database file if it's missing.
    let connect_options = SqliteConnectOptions::from_str(db_path_str)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    
    println!("Database connection successful and migrations run.");

    Ok(pool)
}
