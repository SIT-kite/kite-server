use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct AgentInfoRequest;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ActivityListRequest {
    /// Count of activities per page.
    pub count: u16,
    /// Page index.
    pub index: u16,
}

#[derive(Debug, Serialize)]
pub struct ActivityDetailRequest {
    /// Activity id in sc.sit.edu.cn
    pub id: String,
}

/// Activity link, used for list recent activities.
#[derive(Debug, Serialize, Deserialize)]
pub struct Activity {
    pub title: String,
    pub id: String,
    pub link: String,
}

/// Activity link, used for list recent activities.
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityDetail {
    /// Activity id
    pub id: String,
    /// Activity title
    pub title: String,
    /// Activity start date time
    pub start_time: Option<NaiveDateTime>,
    /// Sign date time
    pub sign_time: Option<NaiveDateTime>,
    /// Activity end date time
    pub end_time: Option<NaiveDateTime>,
    /// Place
    pub place: Option<String>,
    /// Duration
    pub duration: Option<String>,
    /// Activity manager
    pub manager: Option<String>,
    /// Manager contact (phone)
    pub contact: Option<String>,
    /// Activity organizer
    pub organizer: Option<String>,
    /// Acitvity undertaker
    pub undertaker: Option<String>,
    /// Description in text[]
    pub description: Vec<String>,
}
