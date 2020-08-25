pub mod host;
mod model;
mod protocol;

use model::AgentInfo;
use protocol::{Request, Response};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};

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
type RequestQueue = HashMap<usize, oneshot::Sender<Response>>;

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
    channel: Option<mpsc::UnboundedSender<Request>>,
    // last_update
}

/// Local, host.
#[derive(Clone)]
pub struct Host {
    pub agents: Arc<Mutex<AgentMap>>,
}
