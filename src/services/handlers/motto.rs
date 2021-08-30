use actix_web::{get, web, HttpResponse};
use serde::Deserialize;

use crate::error::Result;
use crate::models::motto::Motto;
use crate::models::motto::{MOTTO_MAX_SIZE, MOTTO_MIN_SIZE};
use crate::services::response::ApiResponse;
use crate::services::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MottoRequest {
    pub min_length: Option<u8>,
    pub max_length: Option<u8>,
}

#[get("/motto")]
pub async fn get_one_motto(
    app: web::Data<AppState>,
    form: web::Query<MottoRequest>,
) -> Result<HttpResponse> {
    let parameter = form.into_inner();
    let motto = Motto::random_choice(
        &app.pool,
        parameter.min_length.unwrap_or(MOTTO_MIN_SIZE),
        parameter.max_length.unwrap_or(MOTTO_MAX_SIZE),
    )
    .await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(motto)))
}
