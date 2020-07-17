use crate::error::Result;
use crate::models::motto::Motto;
use crate::models::motto::{MOTTO_MAX_SIZE, MOTTO_MIN_SIZE};
use crate::services::NormalResponse;
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct MottoRequest {
    #[serde(rename = "minLength")]
    pub min_length: Option<u8>,
    #[serde(rename = "maxLength")]
    pub max_length: Option<u8>,
}

#[get("/motto")]
pub async fn get_one_motto(
    pool: web::Data<PgPool>,
    form: web::Query<MottoRequest>,
) -> Result<HttpResponse> {
    let parameter = form.into_inner();
    let motto = Motto::random_choice(
        &pool,
        parameter.min_length.unwrap_or(MOTTO_MIN_SIZE),
        parameter.max_length.unwrap_or(MOTTO_MAX_SIZE),
    )
    .await?;

    Ok(HttpResponse::Ok().json(&NormalResponse::new(motto)))
}
