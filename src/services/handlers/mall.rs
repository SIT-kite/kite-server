use crate::error::{ApiError, Result};
use crate::models::mall::{self, MallError};
use crate::models::{CommonError, PageView};
use crate::services::response::ApiResponse;
use crate::services::{AppState, JwtToken};
use actix_web::{get, post, web, HttpResponse};

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
pub async fn query_textbook(app: web::Data<AppState>, isbn: web::Path<String>) -> Result<HttpResponse> {
    let isbn = isbn.into_inner();
    if !is_valid_isbn(&isbn) {
        return Err(ApiError::new(MallError::InvalidISBN));
    }

    let textbook = mall::query_textbook_by_isbn(&app.pool, &isbn).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(textbook)))
}

#[get("/mall/sorts")]
pub async fn get_goods_sorts(app: web::Data<AppState>) -> Result<HttpResponse> {
    let sort_list = mall::get_goods_sorts(&app.pool).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(sort_list)))
}

#[derive(serde::Deserialize)]
pub struct QueryParams {
    sort: Option<i32>,
    q: Option<String>,
}

#[get("/mall/goods")]
pub async fn get_goods_list(
    app: web::Data<AppState>,
    params: web::Query<QueryParams>,
    page: web::Query<PageView>,
) -> Result<HttpResponse> {
    let page = page.into_inner();
    let goods_list =
        mall::get_full_goods_list(&app.pool, params.sort.unwrap_or(0), &params.q, page).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(goods_list)))
}

#[post("/mall/goods")]
pub async fn publish_goods(
    app: web::Data<AppState>,
    token: JwtToken,
    form: web::Json<mall::NewGoods>,
) -> Result<HttpResponse> {
    let form = form.into_inner();

    if form.description.map(|x| x.len()).ge(&Some(200)) || form.title.len() > 30 {
        return Err(ApiError::new(CommonError::Parameter));
    }
    let goods_id = mall::publish_goods(&app.pool, token.uid, &form).await?;
    struct PublishResult {
        id: i32,
    }
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(&PublishResult { id: goods_id })))
}
