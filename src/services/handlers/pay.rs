//! This module includes interfaces for querying electricity bill and expenses record.
use crate::error::Result;
use crate::models::pay::ElectricityBillRequest;
use crate::services::response::ApiResponse;
use crate::services::AppState;
use actix_web::{get, web, HttpResponse};

/**********************************************************************
    Interfaces in this module:
    query_room_balance()         <-- GET  /pay/room/{room}
    query_consumption_bill()     <-- GET  /pay/consumption/{studentId}
*********************************************************************/

#[get("/pay/room/{room}")]
pub async fn query_room_balance(
    app: web::Data<AppState>,
    form: web::Path<String>,
) -> Result<HttpResponse> {
    let req_balance = ElectricityBillRequest::new(form.into_inner());
    let balance = req_balance.query(&app.host).await?;

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(balance)))
}
