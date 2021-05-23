use serde::Serialize;
use sqlx::PgPool;

use crate::error::{ApiError, Result};

/* Constants. */
// Actually, the two constants are suggested min and max length, because of mottos in our DB.
// If a max size which less than MIN_SIZE is set, that's ok. But the function may not return record any more.
pub const MOTTO_MIN_SIZE: u8 = 5;
pub const MOTTO_MAX_SIZE: u8 = 255;

/// Error handled in motto module.
#[derive(thiserror::Error, Debug, ToPrimitive)]
pub enum MottoError {
    #[error("无数据")]
    NoMoreItem = 100,
}

/* Model */
/// Motto structure, as a motto item.
#[derive(Default, Serialize, sqlx::FromRow)]
pub struct Motto {
    /// Motto id, as a serial column in table.
    pub id: i32,
    /// Author or its book, like "孔子", "《论语》".
    pub source: Option<String>,
    /// Content.
    pub content: String,
    /// Impression count, self increment once when select.
    pub impressions: i32,
}

impl Motto {
    /// Choice one motto randomly from database.
    pub async fn random_choice(client: &PgPool, min_length: u8, max_length: u8) -> Result<Self> {
        let motto: Option<Motto> = sqlx::query_as(
            "WITH 
                whole_fitted AS 
                    (SELECT * FROM motto WHERE length BETWEEN $1 AND $2),
                selected AS 
                    (SELECT * FROM whole_fitted OFFSET floor(random() * (SELECT count(*) FROM whole_fitted)) LIMIT 1)
                UPDATE motto
                    SET impressions = selected.impressions + 1
                    FROM selected
                    WHERE selected.id = motto.id
                RETURNING motto.id, motto.source, motto.content, motto.impressions"
        )
            .bind(min_length as i32)
            .bind(max_length as i32)
            .fetch_optional(client)
            .await?;
        if let Some(motto) = motto {
            return Ok(motto);
        }
        Err(ApiError::new(MottoError::NoMoreItem))
    }
}
