use poem::web::{Data, Json, Path, Query};
use poem::{handler, Result};
use serde::Deserialize;
use sqlx::PgPool;

use crate::model::{game, user, PageView};
use crate::response::ApiResponse;
use crate::service::jwt::JwtToken;

#[derive(Debug, Deserialize)]
pub struct GameRankingParam {
    pub after: Option<String>,
}

#[handler]
pub async fn get_ranking(
    pool: Data<&PgPool>,
    Path(game): Path<i32>,
    Query(parameters): Query<GameRankingParam>,
    Query(page): Query<PageView>,
    token: Option<JwtToken>,
) -> Result<Json<serde_json::Value>> {
    let after = parameters.after;
    let after_time = match after {
        Some(x) => x,
        None => "2010-01-10".to_string(),
    };

    let user = if let Some(token) = token {
        user::get(&pool, token.uid).await?
    } else {
        None
    };
    let data = game::get_ranking(&pool, user.map(|x| x.account), game, after_time, page).await?;
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
