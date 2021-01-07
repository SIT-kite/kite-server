use crate::error::{ApiError, Result};
use serde::Serialize;
use sqlx::PgPool;
use std::borrow::Borrow;

/// Error handled in motto module.
#[derive(thiserror::Error, Debug, ToPrimitive)]
pub enum MallError {
    #[error("教材信息库中无对应教材")]
    NoSuchTextBook = 1,
    #[error("ISBN 格式错误")]
    InvalidISBN = 2,
}

/* Model */
/// Each predefined textbook
#[derive(Default, Serialize, sqlx::FromRow)]
pub struct TextBook {
    /// ISBN of the textbook
    isbn: Option<String>,
    /// Title
    title: String,
    /// Sub-title
    #[serde(rename = "subTitle")]
    sub_title: Option<String>,
    /// Publisher's full name
    press: String,
    /// Author
    author: Option<String>,
    /// Translator (if it is a translated book)
    translator: Option<String>,
    /// Official price
    price: Option<f32>,
    /// Edition
    edition: Option<String>,
    /// Publication year and month
    #[serde(rename = "editionDate")]
    edition_date: Option<String>,
    /// Page count
    page: Option<i32>,
    /// The major of the book itself
    tag: Option<String>,
    // A flag which indicates whether the book written by our school teacher
    // #[serde(rename = "selfEdited")]
    // self_edited: bool,
}

impl TextBook {
    /// Query book by isbn
    pub async fn query_by_isbn(client: &PgPool, isbn: &String) -> Result<Self> {
        let textbook = sqlx::query_as(
            "SELECT
                    isbn, title, sub_title, press, author, translator, price, edition, edition_date, page, tag
                FROM 
                    mall.textbooks
                WHERE 
                    isbn = $1
                LIMIT 1")
            .bind(isbn)
            .fetch_optional(client)
            .await?;
        textbook.ok_or(ApiError::new(MallError::NoSuchTextBook))
    }
}
