use actix_web::{get, web, HttpResponse};
use chrono::Local;
use serde::Deserialize;

use crate::bridge;
use crate::bridge::RequestPayload;
use crate::error::Result;
use crate::services::response::ApiResponse;
use crate::services::AppState;

#[get("/status/timestamp")]
pub async fn get_timestamp() -> Result<HttpResponse> {
    let ts = Local::now().timestamp_millis();

    let response = serde_json::json!({
        "ts": ts,
    });
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(response)))
}

#[derive(Deserialize)]
pub struct PingRequest {
    msg: Option<String>,
}

#[get("/status/agent/ping")]
pub async fn ping_agent(
    params: web::Query<PingRequest>,
    app: web::Data<AppState>,
) -> Result<HttpResponse> {
    let params = params.into_inner();
    let message = params.msg.unwrap_or_else(|| "Hello world!".to_string());

    let agents = &app.agents;
    let payload = RequestPayload::Ping(message);
    let request = bridge::RequestFrame::new(payload);

    let response = agents.request(request).await?;
    match response {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Err(e.into()),
    }
}
