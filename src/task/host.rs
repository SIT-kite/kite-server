use super::model::{AgentInfo, AgentInfoRequest};
use super::protocol::{Request, RequestPayload, Response, ResponsePayload};
use super::{Agent, AgentManager, HostError, RequestQueue, Result};
use crate::task::AgentStatus;
use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tokio::time::Duration;
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

impl Agent {
    /// An agent instance.
    pub fn new(basic: AgentInfo, addr: SocketAddr) -> Self {
        Self {
            basic,
            addr,
            queue: Default::default(),
            channel: None,
            last_update: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Send a request to the agent.
    pub async fn send(&mut self, message: Message) -> Result<()> {
        if let Some(channel) = &self.channel {
            channel.send(message);
            Ok(())
        } else {
            Err(HostError::AgentUnavailable.into())
        }
    }

    /// Select requester and post the response.
    async fn dispatch_response(queue: Arc<Mutex<RequestQueue>>, response: Response) {
        let mut queue = queue.lock().await;

        if let Some(sender) = queue.remove(&response.ack) {
            sender.send(response);
        } else {
            warn!("Received a response message without corresponding request.");
        }
    }

    /// Sender loop: send requests to agent over ws.
    async fn sender_loop<T, Item>(mut socket_tx: T, mut message_rx: mpsc::UnboundedReceiver<Message>)
    where
        T: SinkExt<Item> + std::marker::Unpin,
        T::Error: std::error::Error,
        Item: From<Message>,
    {
        info!("Sender loop started");
        while let Some(message) = message_rx.recv().await {
            if let Err(e) = socket_tx.send(message.into()).await {
                warn!("Failed to send a request to agent: {:?}", e);
                break;
            } else {
                println!("发送成功");
            }
        }
        info!("Sender loop exited.");
    }

    /// Receiver loop: receive responses from the agent and transfer it to the requester.
    async fn receiver_loop<T>(
        mut socket_rx: T,
        queue: Arc<Mutex<RequestQueue>>,
        last_update: Arc<RwLock<Instant>>,
    ) where
        T: StreamExt + std::marker::Unpin,
        T::Item: Into<std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>,
    {
        info!("Receiver loop started");
        while let Some(msg_result) = socket_rx.next().await {
            match msg_result.into() {
                Ok(Message::Binary(content)) => {
                    if let Ok(resp) = bincode::deserialize::<Response>(&content) {
                        info!("New valid response received.");
                        Self::dispatch_response(queue.clone(), resp).await;
                    }
                }
                Ok(Message::Pong(_)) => {
                    let mut last_update = last_update.write().await;
                    *last_update = Instant::now();
                }
                Err(e) => {
                    error!("Error {:?}", e);
                    break;
                }
                _ => {
                    break;
                }
            }
        }
        info!("Receiver loop exited.");
    }

    async fn heartbeat_loop(last_update: Arc<RwLock<Instant>>, tx: UnboundedSender<Message>) {
        let timeout = Duration::from_secs(5);

        loop {
            {
                if last_update.read().await.elapsed() > timeout {
                    info!("Ping");
                    tx.send(Message::Ping(Vec::new()));
                }
            }
            tokio::time::delay_for(Duration::from_secs(2)).await;

            {
                if last_update.read().await.elapsed() > timeout {
                    break;
                }
            }
            tokio::time::delay_for(timeout).await;
        }
        info!("Heartbeat loop exited.");
    }

    /// Start listening response from the agent and request from web.
    pub async fn start(&mut self, ws_stream: WebSocketStream<TcpStream>) {
        let (send_half, recv_half) = ws_stream.split();
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(Self::receiver_loop(
            recv_half,
            self.queue.clone(),
            self.last_update.clone(),
        ));
        tokio::spawn(Self::sender_loop(send_half, rx));
        tokio::spawn(Self::heartbeat_loop(self.last_update.clone(), tx.clone()));
        self.channel = Some(tx);
    }
}

impl AgentManager {
    /// Create a new host instance.
    pub fn new() -> Self {
        info!("A Host instance created.");
        Self {
            agents: Arc::new(Default::default()),
        }
    }

    /// Select an agent randomly and send request packet.
    async fn send(&self, request: Request, callback: oneshot::Sender<Response>) -> Result<()> {
        use rand::prelude::IteratorRandom;

        let mut rng = rand::thread_rng();
        let mut agents = self.agents.lock().await;

        let agent = agents.iter_mut().choose(&mut rng);
        // Send to an agent and record this request.
        if let Some((_, agent)) = agent {
            let seq = request.seq;
            let request = bincode::serialize(&request)?;

            agent.send(Message::binary(request)).await?;

            let mut queue = agent.queue.lock().await;
            queue.insert(seq, callback);

            Ok(())
        } else {
            Err(HostError::NoAgentAvailable.into())
        }
    }

    /// Send request to a agent.
    pub async fn request(&self, request: RequestPayload) -> Result<Response> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::new(request), tx).await?;
        let response = tokio::time::timeout(Duration::from_millis(5000), rx)
            .await
            .map_err(|_| HostError::Timeout)?;

        Ok(response?)
    }

    /// Get agent list
    pub async fn get_agent_list(&self) -> Vec<AgentStatus> {
        let agents = self.agents.lock().await;

        agents
            .iter()
            .map(|(_, agent)| AgentStatus {
                name: agent.basic.name.clone(),
                intranet_addr: "".to_string(),
                external_addr: agent.addr.to_string(),
                queue: 0u16,
            })
            .collect()
    }

    async fn handle_connection(self, agent_addr: SocketAddr, stream: TcpStream) -> Result<()> {
        let mut ws_stream = accept_async(stream).await?;

        println!("New websocket established");
        match Self::request_authentication(&mut ws_stream).await {
            Ok(agent_info) => {
                let mut new_agent = Agent::new(agent_info, agent_addr);
                let mut agents = self.agents.lock().await;

                new_agent.start(ws_stream).await;
                agents.insert(agent_addr, new_agent);

                Ok(())
            }
            _ => Err(HostError::InvalidAgent.into()),
        }
    }

    pub async fn websocket_main(self) -> Result<()> {
        let mut listener = TcpListener::bind("0.0.0.0:8444").await?;

        while let Ok((stream, _)) = listener.accept().await {
            match stream.peer_addr() {
                Ok(peer) => {
                    info!("New WS connection established, with {}", peer);

                    let new_handler = self.clone();
                    tokio::spawn(async move {
                        let exit_result = new_handler.handle_connection(peer, stream).await;
                        info!("One websocket task initialized with {:?}", exit_result);
                    });
                }
                Err(e) => warn!(
                    "New WS connection established, but the peer address is not clear: {}",
                    e
                ),
            }
        }
        Ok(())
    }

    async fn request_authentication(stream: &mut WebSocketStream<TcpStream>) -> Result<AgentInfo> {
        let request = Request::new(RequestPayload::AgentInfo(AgentInfoRequest));
        // Request agent basic info.
        stream.send(Message::Binary(request.to_vec())).await?;

        // Get response
        if let Some(response) = tokio::time::timeout(Duration::from_secs(3), stream.next())
            .await
            .map_err(|_| HostError::Timeout)?
        {
            match response? {
                Message::Binary(binary) => {
                    let r = bincode::deserialize::<Response>(&binary)?;
                    if let ResponsePayload::AgentInfo(info) = bincode::deserialize(&r.payload)? {
                        return Ok(info);
                    }
                }
                _ => {
                    stream.close(None).await?;
                    return Err(HostError::Disconnected.into());
                }
            }
        }
        Err(HostError::Timeout.into())
    }
}
