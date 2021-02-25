use crate::error::{ApiError, Result};
use crate::models::search::query_notice;
use crate::models::{CommonError, PageView};
use crate::services::{response::ApiResponse, AppState, JwtToken};
use actix_web::{get, web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct SearchReq {
    pub query: String,
}

#[get("/search/{sort}/")]
pub async fn search(
    app: web::Data<AppState>,
    token: Option<JwtToken>,
    path: web::Path<String>,
    query: web::Query<SearchReq>,
    page: web::Query<PageView>,
) -> Result<HttpResponse> {
    let sort = path.into_inner();
    let query = query.into_inner().query;

    return match sort.as_str() {
        "notice" => {
            if token.is_none() {
                return Err(ApiError::new(CommonError::LoginNeeded));
            }
            let result = query_notice(&app.pool, &query, &page).await?;
            Ok(HttpResponse::Ok().json(ApiResponse::normal(result)))
        }
        _ => Ok(HttpResponse::Ok().json(ApiResponse::empty())),
    };
}
