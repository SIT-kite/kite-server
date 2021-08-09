use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, Result};
use crate::models::telephone;
use crate::services::response::ApiResponse;
use crate::services::AppState;

#[get("/address/telephone")]
pub async fn query_all_telephone(app: web::Data<AppState>) -> Result<HttpResponse> {
    let result = telephone::query_all_phone_number(&app.pool).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(result)))
}
