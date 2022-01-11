use poem::web::{Data, Json};
use poem::{handler, Result};
use sqlx::PgPool;

use crate::model::contact;
use crate::response::ApiResponse;

/**********************************************************************
 Interfaces in this module:
 query_all_contacts()         <-- GET /contact
*********************************************************************/

#[handler]
pub async fn query_all_contacts(pool: Data<&PgPool>) -> Result<Json<serde_json::Value>> {
    let result = contact::get_all_contacts(&pool).await?;
    let response: serde_json::Value = ApiResponse::normal(result).into();

    Ok(Json(response))
}
