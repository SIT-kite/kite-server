pub mod host;
mod model;
mod protocol;

use model::AgentInfo;

use crate::task::protocol::{Request, Response, ResponsePayload};
use serde::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};

pub type Result<T> = anyhow::Result<T>;

#[derive(Debug, thiserror::Error)]
/// Business error of web socket host
pub enum HostError {
    #[error("无可用的校内节点")]
    NoAgentAvailable,
    #[error("请求超时")]
    Timeout,
    #[error("连接已关闭")]
    Disconnected,
    #[error("当前节点不可用")]
    AgentUnavailable,
    #[error("无效客户端")]
    InvalidAgent,
}

/// Request queue in agent cache. When response received, use this queue to found the requester.
type RequestQueue = HashMap<u64, oneshot::Sender<Response>>;

/// Agents
type AgentMap = HashMap<SocketAddr, Agent>;

/// Agent structure, for each client node.
pub struct Agent {
    /// Agent info reported by agent.
    basic: AgentInfo,
    /// Remote socket addr
    addr: SocketAddr,
    /// Request queue, used to callback when the response is received.
    queue: Arc<Mutex<RequestQueue>>,
    /// Request channel to sender loop.
    channel: Option<mpsc::Sender<Request>>,
    /// Agent last update time.
    last_update: Arc<RwLock<Instant>>,
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
