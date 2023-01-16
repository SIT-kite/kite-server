use chrono::{DateTime, Local};

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct Picture {
    /// Picture uuid
    pub id: super::Uuid,
    /// Updater
    pub uid: i32,
    /// Web path to origin image
    pub url: String,
    /// Web path to thumbnail image
    pub thumbnail: String,
    /// Upload time
    pub ts: DateTime<Local>,
    /// Extension
    pub ext: String,
}
