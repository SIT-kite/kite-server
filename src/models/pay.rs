use crate::bridge::{AgentManager, HostError, RequestPayload, ResponsePayload};
use crate::error::{ApiError, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
/// Request for electricity bill
pub struct ElectricityBillRequest {
    pub room: String,
}

#[derive(Serialize, Deserialize)]
/// Electricity Bill for FengXian dormitory.
pub struct ElectricityBill {
    /// Room id in the format which described in the doc.
    pub room: String,
    /// Remaining paid amount
    pub balance: f32,
    /// Remaining subsidy amount
    pub subsidy: f32,
    /// Total available amount
    pub total: f32,
    /// Available power
    pub power: f32,
}

impl ElectricityBillRequest {
    /// Create a new electricity bill request.
    pub fn new(room: String) -> Self {
        Self { room }
    }

    /// Send the request and get the response.
    pub async fn query(self, host: &AgentManager) -> Result<ElectricityBill> {
        let q = RequestPayload::ElectricityBill(self);
        let r = host.request(q).await?;

        if let ResponsePayload::ElectricityBill(e) = r.payload()?? {
            Ok(e)
        } else {
            Err(ApiError::new(HostError::BadResponse))
        }
    }
}
