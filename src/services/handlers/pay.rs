//! This module includes interfaces for querying electricity bill and expenses record.
use crate::error::{ApiError, Result};
use crate::models::pay::BalanceManager;
use crate::models::CommonError;
use crate::services::response::ApiResponse;
use crate::services::AppState;
use actix_web::{get, web, HttpResponse};

/**********************************************************************
    Interfaces in this module:
    query_room_balance()         <-- GET  /pay/room/{room}
    query_consumption_bill()     <-- GET  /pay/consumption/{studentId}
*********************************************************************/

#[get("/pay/room/{room}")]
pub async fn query_room_balance(app: web::Data<AppState>, form: web::Path<i32>) -> Result<HttpResponse> {
    let room = form.into_inner();
    let manager = BalanceManager::new(&app.pool);
    let result = manager.query_last_balance(room).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(result)))
}
