use actix_web::{get, post, web, HttpResponse};

use crate::error::{ApiError, Result};
use crate::models::mall::{self, MallError};
use crate::models::{CommonError, PageView};
use crate::services::response::ApiResponse;
use crate::services::{AppState, JwtToken};

pub fn is_numeric(s: &str) -> bool {
    for ch in s.chars() {
        if !ch.is_numeric() {
            return false;
        }
    }
    true
}

/// It's not a strict function for validating isbn numbers.
pub fn is_valid_isbn(isbn: &str) -> bool {
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
    let params = params.into_inner();

    let goods_list = if params.q.is_some() || params.sort.is_some() {
        let sort = params.sort.unwrap_or(0i32);
        let params = params.q.unwrap_or_default();
        mall::query_goods(&app.pool, &params, sort, page).await?
    } else {
        mall::get_full_goods_list(&app.pool, page).await?
    };
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(goods_list)))
}

#[get("/mall/goods/{id}")]
pub async fn get_goods_byid(
    app: web::Data<AppState>,
    id: web::Path<i32>
) -> Result<HttpResponse> {

    let id = id.into_inner();

    let goods_detail = mall::get_goods_detail(&app.pool,id).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(goods_detail)))
}


#[post("/mall/goods")]
pub async fn publish_goods(
    app: web::Data<AppState>,
    token: JwtToken,
    form: web::Json<mall::NewGoods>,
) -> Result<HttpResponse> {
    let form = form.into_inner();

    if let Some(description_text) = &form.description {
        if description_text.len() > 200 {
            return Err(ApiError::new(CommonError::Parameter));
        }
    }
    if form.title.len() > 30 {
        return Err(ApiError::new(CommonError::Parameter));
    }
    let goods_id = mall::publish_goods(&app.pool, token.uid, &form).await?;
    #[derive(serde::Serialize)]
    struct PublishResult {
        id: i32,
    }
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(&PublishResult { id: goods_id })))
}

#[post("/mall/update_goods")]
pub async fn update_goods(
    app: web::Data<AppState>,
    token: JwtToken,
    form: web::Json<mall::NewGoods>,
) -> Result<HttpResponse> {
    let form = form.into_inner();

    let goods_detail = mall::get_goods_detail(&app.pool,form.id).await?;

    if !(goods_detail.publisher == token.uid || token.is_admin){
        return Err(ApiError::new(CommonError::Forbidden));
    }
    let goods_id = mall::update_goods(&app.pool,&form).await?;

    #[derive(serde::Serialize)]
    struct PublishResult {
        id: i32,
    }
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(&PublishResult { id: goods_id })))
}

#[get("/mall/delete_goods/{id}")]
pub async fn delete_goods(
    app: web::Data<AppState>,
    id: web::Path<i32>
) -> Result<HttpResponse> {

    let id = id.into_inner();
    //
    // let goods_detail = mall::get_goods_detail(&app.pool,id).await?;
    //
    // if !(goods_detail.publisher == token.uid || token.is_admin){
    //     return Err(ApiError::new(CommonError::Forbidden));
    // }

    let goods_id = mall::delete_goods(&app.pool,id).await?;

    Ok(HttpResponse::Ok().json(goods_id))
}
