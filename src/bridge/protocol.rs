use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;

use crate::bridge::HostError;
use crate::error::ApiError;

use super::model::*;
use super::Result;

lazy_static! {
    /// Last seq of request packet
    static ref LAST_SEQ: AtomicU64 = AtomicU64::new(1u64);
}

// Result has two sides, Ok(ResponsePayload) and Err(ResponseError)
pub type ResponseResult = std::result::Result<ResponsePayload, ErrorResponse>;

/// Host request
// Implement Debug for error handling.
#[derive(Default, Serialize)]
pub struct Request {
    /// Request sequence
    pub seq: u64,
    /// Packet size
    pub size: u32,
    /// Payload
    pub payload: Vec<u8>,
}

/// Agent response
#[derive(Debug, Default, Deserialize)]
pub struct Response {
    /// Response sequence
    pub ack: u64,
    /// Response size
    pub size: u32,
    /// Status code
    pub code: u16,
    /// Payload
    pub payload: Vec<u8>,
}

/// Error code and message from response
#[derive(Debug, thiserror::Error)]
#[error("{} ({})", msg, code)]
pub struct ErrorResponse {
    pub code: u16,
    pub msg: String,
}

/// Response payload
#[derive(Serialize)]
pub enum RequestPayload {
    AgentInfo(AgentInfoRequest),
    ActivityList(ActivityListRequest),
    ScoreList(CourseScoreRequest),
}

/// Response payload
#[derive(Deserialize)]
pub enum ResponsePayload {
    AgentInfo(AgentInfo),
    ActivityList(Vec<Activity>),
    ScoreList(Vec<CourseScore>),
}

impl Request {
    pub fn new(payload: RequestPayload) -> Self {
        let seq = LAST_SEQ.fetch_add(1, Ordering::Relaxed);
        let payload = bincode::serialize(&payload).unwrap();

        Self {
            seq,
            // We will not construct a message more than 2^32 bytes
            size: payload.len() as u32,
            payload,
        }
    }
}

impl Response {
    async fn read_header(buffer: &mut BufReader<OwnedReadHalf>) -> Result<Self> {
        // Note: Do not use Response {} initialization statement here because the read_xx() calling
        // order is not under control.

        // Default response header is 14 bytes.
        let mut response = Response::default();

        // Read the control fields
        response.ack = buffer.read_u64().await?;
        response.size = buffer.read_u32().await?;
        response.code = buffer.read_u16().await?;

        Ok(response)
    }

    pub async fn from_stream(buffer: &mut BufReader<OwnedReadHalf>) -> Result<Self> {
        let mut response = Self::read_header(buffer).await?;

        if response.size == 0 {
            return Ok(response);
        }
        if response.size > 10 * 1024 * 1024 {
            return Err(HostError::TooLargePayload.into());
        }
        response.payload = vec![0u8; response.size as usize];
        // Read body
        let mut p = 0usize; // read len
        while p < response.size as usize {
            let mut read_currently = response.size as usize - p;
            if read_currently > 2048 {
                read_currently = 2048usize;
            }
            p += buffer
                .read_exact(&mut response.payload[p..(p + read_currently)])
                .await?;
        }
        Ok(response)
    }

    pub async fn is_ok(&self) -> bool {
        self.code == 0
    }

    pub fn payload(self) -> Result<ResponseResult> {
        if self.code == 0 {
            Ok(Ok(bincode::deserialize(&self.payload)?))
        } else {
            let err_string = std::str::from_utf8(&self.payload)?;
            Ok(Err(ErrorResponse {
                code: self.code,
                msg: String::from(err_string),
            }))
        }
    }
}

impl From<ErrorResponse> for ApiError {
    fn from(resp: ErrorResponse) -> Self {
        ApiError {
            code: resp.code,
            inner_msg: None,
            error_msg: Some(resp.msg),
        }
    }
}
