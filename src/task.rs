pub mod host;
mod model;
mod protocol;

use model::AgentInfo;

use protocol::{Request, Response};
use serde::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};

pub type Result<T> = anyhow::Result<T>;

pub use protocol::{RequestPayload, ResponsePayload};

#[derive(Debug, ToPrimitive, thiserror::Error)]
/// Business error of web socket host
pub enum HostError {
    #[error("无可用的代理节点，无法连接到校园网")]
    NoAgentAvailable = 120,
    #[error("请求超时")]
    Timeout = 121,
    #[error("连接已关闭")]
    Disconnected = 122,
    #[error("当前代理节点不可用")]
    AgentUnavailable = 123,
    #[error("返回的响应与请求类型不一致")]
    BadResponse = 124,
    #[error("非法 Agent")]
    InvalidAgent = 125,
    #[error("Payload 过大")]
    TooLargePayload = 126,
}

/// Request queue in agent cache. When response received, use this queue to found the requester.
type RequestQueue = HashMap<u64, oneshot::Sender<Response>>;

/// Agents
type AgentMap = HashMap<SocketAddr, Agent>;

struct HaltChannel {
    sender: broadcast::Sender<()>,
    receiver: broadcast::Receiver<()>,
}

/// Agent structure, for each client node.
#[derive(Clone)]
pub struct Agent {
    /// Agent info reported by agent.
    basic: AgentInfo,
    /// Remote socket addr
    addr: SocketAddr,
    /// Request queue, used to callback when the response is received.
    queue: Arc<Mutex<RequestQueue>>,
    /// Request channel to sender loop.
    channel: Option<mpsc::Sender<Request>>,
    /// Halt channel
    halt: Option<HaltChannel>,
}

/// Agent state
#[derive(Serialize)]
pub struct AgentStatus {
    /// Agent name
    pub name: String,
    /// Intranet network address
    #[serde(rename = "intranetAddr")]
    pub intranet_addr: String,
    /// External network address
    #[serde(rename = "externalAddr")]
    pub external_addr: String,
    /// Current queue length
    pub queue: u16,
}

/// Local, host.
#[derive(Clone)]
pub struct AgentManager {
    agents: Arc<Mutex<AgentMap>>,
}
