use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

pub async fn init_pool(db_path: &str) -> SqlitePool {
    let options = SqliteConnectOptions::from_str(db_path)
        .expect("invalid db path")
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .expect("failed to connect to database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            text TEXT NOT NULL,
            remind_at TEXT,
            synced BOOLEAN NOT NULL DEFAULT FALSE,
            done BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("failed to run migration");

    // Migration: add done column if missing
    let _ = sqlx::query("ALTER TABLE notes ADD COLUMN done BOOLEAN NOT NULL DEFAULT FALSE")
        .execute(&pool)
        .await;

    pool
}
