mod db;
mod models;
mod routes;

use axum::routing::{get, patch, put};
use axum::Router;
use tower_http::services::ServeDir;

use routes::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let _ = dotenvy::dotenv();

    let db_path = std::env::var("NAVINOTE_DB_PATH").unwrap_or_else(|_| "navinote.db".into());
    let token = std::env::var("NAVINOTE_TOKEN").expect("NAVINOTE_TOKEN must be set");
    let port: u16 = std::env::var("NAVINOTE_PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()
        .expect("NAVINOTE_PORT must be a number");
    let static_dir = std::env::var("NAVINOTE_STATIC_DIR").unwrap_or_else(|_| "dist".into());

    let pool = db::init_pool(&db_path).await;

    let state = AppState { pool, token };

    let api = Router::new()
        .route("/api/health", get(routes::health))
        .route("/api/notes", get(routes::list_notes).post(routes::create_note))
        .route("/api/notes/{id}", put(routes::update_note).delete(routes::delete_note))
        .route("/api/notes/{id}/synced", patch(routes::mark_synced))
        .with_state(state);

    let app = api.fallback_service(ServeDir::new(&static_dir));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("failed to bind");

    tracing::info!("listening on 0.0.0.0:{port}");
    axum::serve(listener, app).await.expect("server error");
}
