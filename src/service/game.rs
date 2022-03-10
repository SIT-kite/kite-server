use poem::web::{Data, Json, Path};
use poem::{handler, Result};
use sqlx::PgPool;

use crate::model::game;
use crate::response::ApiResponse;

#[handler]
pub async fn get_ranking(pool: Data<&PgPool>, Path(game): Path<i32>) -> Result<Json<serde_json::Value>> {
    let data = game::get_ranking(&pool, game, 20).await?;
    let response: serde_json::Value = ApiResponse::normal(data).into();

    Ok(Json(response))
}

#[handler]
pub async fn post_record(pool: Data<&PgPool>, Json(record): Json<game::GameRecord>) -> Result<Json<serde_json::Value>> {
    game::post_record(&pool, record).await?;

    Ok(Json(ApiResponse::<()>::empty().into()))
}
