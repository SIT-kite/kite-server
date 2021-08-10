use actix_web::{get, web, HttpResponse};

use crate::error::Result;
use crate::models::contact;
use crate::services::response::ApiResponse;
use crate::services::AppState;

#[get("/contact")]
pub async fn query_all_telephone(app: web::Data<AppState>) -> Result<HttpResponse> {
    let result = contact::get_all_contacts(&app.pool).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(result)))
}
