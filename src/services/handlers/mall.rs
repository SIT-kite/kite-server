use crate::error::{ApiError, Result};
use crate::models::mall::MallError;
use crate::models::mall::TextBook;
use crate::services::response::ApiResponse;
use crate::services::AppState;
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;

pub fn is_numeric(s: &String) -> bool {
    for ch in s.chars() {
        if !ch.is_numeric() {
            return false;
        }
    }
    return true;
}

/// It's not a strict function for validating isbn numbers.
pub fn is_valid_isbn(isbn: &String) -> bool {
    if isbn.len() != 13 && isbn.len() != 10 {
        return false;
    }
    if !is_numeric(&isbn) {
        return false;
    }
    true
}

#[get("/mall/textbook/{isbn}")]
pub async fn query_textbook_by_isbn(
    app: web::Data<AppState>,
    isbn: web::Path<String>,
) -> Result<HttpResponse> {
    let isbn = isbn.into_inner();
    if !is_valid_isbn(&isbn) {
        return Err(ApiError::new(MallError::InvalidISBN));
    }

    let textbook = TextBook::query_by_isbn(&app.pool, &isbn).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(textbook)))
}
