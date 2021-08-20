use actix_web::{get, web, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::bridge;
use crate::bridge::RequestPayload;
use crate::error::Result;
use crate::services::response::ApiResponse;
use crate::services::AppState;

#[get("/status/timestamp")]
pub async fn get_timestamp() -> Result<HttpResponse> {
    let ts = Utc::now().timestamp_millis();

    let response = serde_json::json!({
        "ts": ts,
    });
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(response)))
}

// TODO: Consider to separate the interface to /status/cpu, /status/memory
#[get("/status/system")]
pub async fn get_system_status() -> Result<HttpResponse> {
    #[derive(Serialize, Default)]
    struct MemoryStatus {
        pub total: u64,
        pub avail: u64,
        pub free: u64,
    }
    #[derive(Serialize)]
    struct SystemStatus {
        #[serde(rename = "loadAvg")]
        pub load_avg: String,
        pub memory: MemoryStatus,
    }

    let load_avg = sys_info::loadavg()
        .map(|x| (format!("{:.2} {:.2} {:.2}", x.one, x.five, x.fifteen)))
        .unwrap_or_default();
    let memory = sys_info::mem_info()
        .map(|x| MemoryStatus {
            total: x.total / 1024,
            avail: x.avail / 1024,
            free: x.free / 1024,
        })
        .unwrap_or_default();

    Ok(HttpResponse::Ok().json(&ApiResponse::normal(SystemStatus { load_avg, memory })))
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
