use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::{CreateNote, Note, NotesQuery, UpdateNote};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub token: String,
}

fn check_auth(headers: &HeaderMap, token: &str) -> Result<(), StatusCode> {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let expected = format!("Bearer {token}");
    if auth != expected {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(())
}

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn list_notes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NotesQuery>,
) -> Result<Json<Vec<Note>>, StatusCode> {
    check_auth(&headers, &state.token)?;

    let notes = if let Some(synced) = query.synced {
        sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE synced = ? ORDER BY created_at DESC")
            .bind(synced)
            .fetch_all(&state.pool)
            .await
    } else {
        sqlx::query_as::<_, Note>("SELECT * FROM notes ORDER BY created_at DESC")
            .fetch_all(&state.pool)
            .await
    };

    notes.map(Json).map_err(|e| {
        tracing::error!("list_notes query failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

pub async fn create_note(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateNote>,
) -> Result<(StatusCode, Json<Note>), StatusCode> {
    check_auth(&headers, &state.token)?;

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("INSERT INTO notes (id, text, remind_at, synced, done, created_at, updated_at) VALUES (?, ?, ?, FALSE, FALSE, ?, ?)")
        .bind(&id)
        .bind(&input.text)
        .bind(&input.remind_at)
        .bind(&now)
        .bind(&now)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("create_note insert failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let note = sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("create_note fetch failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::CREATED, Json(note)))
}

pub async fn update_note(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<UpdateNote>,
) -> Result<Json<Note>, StatusCode> {
    check_auth(&headers, &state.token)?;

    let now = chrono::Utc::now().to_rfc3339();

    // Fetch existing
    let existing = sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let text = input.text.unwrap_or(existing.text);
    let remind_at = input.remind_at.or(existing.remind_at);

    sqlx::query("UPDATE notes SET text = ?, remind_at = ?, updated_at = ? WHERE id = ?")
        .bind(&text)
        .bind(&remind_at)
        .bind(&now)
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let note = sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(note))
}

pub async fn delete_note(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    check_auth(&headers, &state.token)?;

    let result = sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn mark_synced(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    check_auth(&headers, &state.token)?;

    let now = chrono::Utc::now().to_rfc3339();

    let result = sqlx::query("UPDATE notes SET synced = TRUE, updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::OK)
}

pub async fn mark_done(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    check_auth(&headers, &state.token)?;

    let now = chrono::Utc::now().to_rfc3339();

    let result = sqlx::query("UPDATE notes SET done = TRUE, synced = FALSE, updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::OK)
}
