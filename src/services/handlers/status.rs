use actix_web::{get, web, HttpResponse};
use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::bridge;
use crate::bridge::{
    Activity, ActivityDetail, ActivityDetailRequest, ActivityListRequest, AgentInfo, AgentInfoRequest,
    Major, MajorRequest, RequestPayload, SchoolYear, Semester,
};
use crate::error::{ApiError, Result};
use crate::models::CommonError;
use crate::services::response::ApiResponse;
use crate::services::{AppState, JwtToken};

#[get("/status/timestamp")]
pub async fn get_timestamp() -> Result<HttpResponse> {
    let ts = Local::now().timestamp_millis();

    let response = serde_json::json!({
        "ts": ts,
    });
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(response)))
}

#[get("/status/agent/")]
pub async fn get_agent_list(app: web::Data<AppState>, token: Option<JwtToken>) -> Result<HttpResponse> {
    let token = token.ok_or_else(|| CommonError::LoginNeeded)?;
    if !token.is_admin {
        return Err(CommonError::Forbidden.into());
    }

    let agents = &app.agents;
    let response = serde_json::json!({
        "agents": agents.get_client_list().await,
    });
    Ok(HttpResponse::Ok().json(ApiResponse::normal(response)))
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
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse::normal(response))),
        Err(e) => Err(e.into()),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MRequest {
    pub entrance_year: Option<i32>,
    pub account: String,
    pub passwd: String,
}

#[get("/status/agent/major")]
pub async fn major_agent(
    params: web::Query<MRequest>,
    app: web::Data<AppState>,
) -> Result<HttpResponse> {
    let params = params.into_inner();
    let year = match params.entrance_year {
        Some(y) => SchoolYear::SomeYear(y),
        None => SchoolYear::AllYear,
    };

    let data = MajorRequest {
        entrance_year: year,
        account: params.account,
        passwd: params.passwd,
    };

    let agents = &app.agents;
    let payload = RequestPayload::MajorList(data);
    let request = bridge::RequestFrame::new(payload);

    let response = agents.request(request).await?;
    match response {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse::normal(response))),
        Err(e) => Err(e.into()),
    }
}
