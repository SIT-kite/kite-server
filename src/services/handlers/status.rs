use crate::error::Result;
use crate::services::response::ApiResponse;
use crate::services::AppState;
use actix_web::{get, web, HttpResponse};
use chrono::Utc;
use serde::Serialize;

#[get("/status/timestamp")]
pub async fn get_timestamp() -> Result<HttpResponse> {
    let ts = Utc::now().timestamp_millis();

    #[derive(Serialize)]
    struct Response {
        pub ts: i64,
    }
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(Response { ts })))
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

#[get("/status/agent")]
pub async fn get_agent_list(app: web::Data<AppState>) -> Result<HttpResponse> {
    let host = &app.host;
    let agents = host.get_agent_list().await;

    Ok(HttpResponse::Ok().json(agents))
}
