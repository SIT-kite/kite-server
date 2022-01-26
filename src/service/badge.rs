use poem::web::{Data, Json};
use poem::{handler, Result};
use serde_json::json;
use sqlx::PgPool;

use crate::model::badge;
use crate::response::ApiResponse;
use crate::service::JwtToken;

/**********************************************************************
 Interfaces in this module:
 get_my_cards()         <-- GET /badge/card/
 get_event_result()     <-- GET /badge/result
*********************************************************************/

#[handler]
pub async fn get_my_cards(pool: Data<&PgPool>, token: JwtToken) -> Result<Json<serde_json::Value>> {
    let cards = badge::get_cards_list(&pool, token.uid).await?;
    let response: serde_json::Value = ApiResponse::normal(cards).into();

    Ok(Json(response))
}

#[handler]
pub async fn get_event_result() -> Result<Json<serde_json::Value>> {
    let result = json!({
        "result": 0,
        "url": Option::<()>::None,
    });
    let response: serde_json::Value = ApiResponse::normal(result).into();

    Ok(Json(response))
}
