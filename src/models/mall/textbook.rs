use super::TextBook;
use crate::error::{ApiError, Result};
use crate::models::mall::MallError;
use sqlx::PgPool;

/// Query book by isbn
pub async fn query_textbook_by_isbn(db: &PgPool, isbn: &String) -> Result<TextBook> {
    let textbook = sqlx::query_as!(
        TextBook,
        "SELECT isbn, title, sub_title, press, author, translator, price, edition, 
                edition_date, page, tag
        FROM mall.textbooks
        WHERE isbn = $1 LIMIT 1",
        isbn
    )
    .fetch_optional(db)
    .await?;

    textbook.ok_or(ApiError::new(MallError::NoSuchTextBook))
}
