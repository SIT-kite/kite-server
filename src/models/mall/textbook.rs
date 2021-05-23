use sqlx::PgPool;

use crate::error::{ApiError, Result};
use crate::models::mall::MallError;

use super::TextBook;

/// Query book by isbn
pub async fn query_textbook_by_isbn(db: &PgPool, isbn: &str) -> Result<TextBook> {
    sqlx::query_as(
        "SELECT isbn, title, sub_title, press, author, translator, price, edition, 
                edition_date, page, tag
        FROM mall.textbooks
        WHERE isbn = $1 LIMIT 1",
    )
    .bind(isbn)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| ApiError::new(MallError::NoSuchTextBook))
}
