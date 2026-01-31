use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub id: String,
    pub text: String,
    pub remind_at: Option<String>,
    pub synced: bool,
    pub done: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNote {
    pub text: String,
    pub remind_at: Option<String>,
    #[serde(default)]
    pub done: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNote {
    pub text: Option<String>,
    pub remind_at: Option<String>,
    pub done: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct NotesQuery {
    pub synced: Option<bool>,
}
