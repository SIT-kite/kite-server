use super::model::*;
use super::Result;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

lazy_static! {
    /// Last seq of request packet
    static ref LAST_SEQ: AtomicU32 = AtomicU32::new(1 as u32);
}

/// Host request
// Implement Debug for error handling.
#[derive(Serialize)]
pub struct Request {
    /// Request sequence
    pub seq: usize,
    /// Payload
    pub payload: Vec<u8>,
}

/// Agent response
#[derive(Deserialize)]
pub struct Response {
    /// Response sequence
    pub ack: usize,
    /// Status code
    pub code: u16,
    /// Payload
    pub payload: Vec<u8>,
}

/// Response payload
#[derive(Serialize)]
pub enum RequestPayload {
    AgentInfo(AgentInfoRequest),
    ElectricityBill(ElectricityBillRequest),
    ActivityList(ActivityListRequest),
    ScoreList(CourseScoreRequest),
}

/// Response payload
#[derive(Deserialize)]
pub enum ResponsePayload {
    AgentInfo(AgentInfo),
    ElectricityBill(ElectricityBill),
    ActivityList(Vec<Activity>),
    ScoreList(Vec<CourseScore>),
}

impl Request {
    pub fn new(payload: RequestPayload) -> Self {
        // TODO
        let seq = LAST_SEQ.fetch_add(1, Ordering::Relaxed) as usize;

        Self {
            seq,
            payload: bincode::serialize(&payload).unwrap(),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

impl Response {
    pub fn from(content: Vec<u8>) -> Result<Self> {
        Ok(bincode::deserialize(&content)?)
    }
}
