use poem::web::{Data, Json, Path};
use poem::{handler, Result};
use sqlx::PgPool;

use crate::model::{game, user};
use crate::response::ApiResponse;
use crate::service::jwt::JwtToken;

#[handler]
pub async fn get_ranking(
    pool: Data<&PgPool>,
    Path(game): Path<i32>,
    token: Option<JwtToken>,
) -> Result<Json<serde_json::Value>> {
    let user = if let Some(token) = token {
        user::get(&pool, token.uid).await?
    } else {
        None
    };

    let data = game::get_ranking(&pool, user.map(|x| x.account), game).await?;
    let response: serde_json::Value = ApiResponse::normal(data).into();

    Ok(Json(response))
}

#[handler]
pub async fn post_record(
    pool: Data<&PgPool>,
    token: JwtToken,
    Json(record): Json<game::GameRecord>,
) -> Result<Json<serde_json::Value>> {
    game::post_record(&pool, token.uid, record).await?;

    Ok(Json(ApiResponse::<()>::empty().into()))
}
