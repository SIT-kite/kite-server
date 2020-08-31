use super::model::*;
use super::Result;
use bytes::{Buf, BytesMut};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use tokio::io::{AsyncReadExt, BufReader};

lazy_static! {
    /// Last seq of request packet
    static ref LAST_SEQ: AtomicU64 = AtomicU64::new(1u64);
}

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
    async fn read_header(stream: &mut (impl AsyncReadExt + Unpin)) -> Result<Self> {
        // Default response header is 14 bytes.
        let mut buffer = BufReader::with_capacity(14, stream);
        let mut response = Response::default();

        // Read the control fields
        response.ack = buffer.read_u64().await?;
        response.size = buffer.read_u32().await?;
        response.code = buffer.read_u16().await?;

        Ok(response)
    }

    pub async fn from_stream(
        stream: &mut (impl AsyncReadExt + Unpin),
        buffer: &mut BytesMut,
    ) -> Result<Self> {
        let mut response = Self::read_header(stream).await?;

        if response.size == 0 {
            return Ok(response);
        }
        if buffer.capacity() < response.size as usize {
            buffer.resize(response.size as usize, 0u8);
        }
        // Read body
        let mut p = 0usize; // read len
        while p < response.size as usize {
            let mut read_currently = response.size as usize - p;
            if read_currently > 2048 {
                read_currently = 2048usize;
            }
            p += stream.read_exact(&mut buffer[p..(p + read_currently)]).await?;
        }
        response.payload = buffer.to_vec();
        Ok(response)
    }
}
