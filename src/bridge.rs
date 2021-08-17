pub use host::AgentManager;

mod host;
mod model;
mod protocol;

pub type Result<T> = anyhow::Result<T>;

#[derive(Debug, ToPrimitive, thiserror::Error)]
/// Business error of web socket host
pub enum HostError {
    #[error("无可用的代理节点，无法连接到校园网")]
    NoAgentAvailable = 120,
    #[error("请求超时")]
    Timeout = 121,
}

/// Agent state
#[derive(serde::Serialize)]
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
