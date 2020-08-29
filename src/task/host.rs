use super::model::{AgentInfo, AgentInfoRequest};
use super::protocol::{Request, RequestPayload, Response, ResponsePayload};
use super::{Agent, AgentManager, AgentStatus, HostError, RequestQueue};
use bytes::BytesMut;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tokio::time::Duration;

use super::Result;
use log::{info, warn};

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

    /// Send request to the agent
    async fn send(&mut self, request: Request) -> Result<()> {
        let mut channel = self.channel.clone().ok_or(HostError::AgentUnavailable)?;

        // Send request packet to the sender loop to post, if the process failed, set channel to None
        // so that the next try could return immediately.
        channel.send(request).await.or_else(move |_| {
            self.channel = None;
            Err(HostError::AgentUnavailable)
        });
        Ok(())
    }

    /// Request to agent, and add an oneshot sender and it can be used when the response received.
    /// Return HostError timeout if the agent doesn't respond in a reasonal time.
    pub async fn request(&mut self, request: RequestPayload) -> Result<Response> {
        let request = Request::new(request);
        let seq = request.seq;

        // Send the request to the sender loop
        self.send(request).await?;

        // Result channel, return rx to the caller and save the tx to the queue.
        let (tx, rx) = oneshot::channel();
        // Use a pair of big parentheses, to drop queue automatically.
        {
            let mut queue = self.queue.lock().await;
            queue.insert(seq, tx);
        }
        match tokio::time::timeout(Duration::from_millis(5000), rx).await {
            Ok(result) => Ok(result?),
            Err(_) => {
                let mut queue = self.queue.lock().await;
                queue.remove(&seq);

                Err(HostError::Timeout.into())
            }
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
    async fn sender_loop(
        mut socket_tx: OwnedWriteHalf,
        mut request_rx: mpsc::Receiver<Request>,
    ) -> Result<()> {
        info!("Sender loop started");
        while let Some(request) = request_rx.recv().await {
            socket_tx.write_u64(request.seq).await?;
            socket_tx.write_u32(request.size).await?;
            socket_tx.write_all(&request.payload).await?;
        }
        info!("Sender loop exited.");
        Ok(())
    }

    /// Receiver loop: receive responses from the agent and transfer it to the requester.
    async fn receiver_loop(
        mut socket_rx: OwnedReadHalf,
        queue: Arc<Mutex<RequestQueue>>,
        last_update: Arc<RwLock<Instant>>,
    ) -> Result<()> {
        info!("Receiver loop started");
        let mut buffer = BytesMut::with_capacity(1024 * 1024); // Default 1MB buffer

        loop {
            let response = Response::from_stream(&mut socket_rx, &mut buffer).await?;
            Self::dispatch_response(queue.clone(), response).await;

            let mut last_update = last_update.write().await;
            *last_update = Instant::now();
        }
        info!("Receiver loop exited.");
        Ok(())
    }

    async fn heartbeat_loop(last_update: Arc<RwLock<Instant>>, tx: mpsc::Sender<Request>) {
        let timeout = Duration::from_secs(5);

        loop {
            let last_update = last_update.read().await;
            if last_update.elapsed().as_secs() > 10 {}
        }
        info!("Heartbeat loop exited.");
    }

    /// Start listening response from the agent and request from web.
    pub async fn start(&mut self, stream: TcpStream) {
        let (recv_half, send_half) = stream.into_split();
        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(Self::receiver_loop(
            recv_half,
            self.queue.clone(),
            self.last_update.clone(),
        ));
        tokio::spawn(Self::sender_loop(send_half, rx));
        // tokio::spawn(Self::heartbeat_loop(self.last_update.clone(), tx.clone()));
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
    async fn request(&self, request: RequestPayload) -> Result<Response> {
        use rand::prelude::IteratorRandom;

        let mut rng = rand::thread_rng();
        let mut agents = self.agents.lock().await;

        let agent = agents.iter_mut().choose(&mut rng);
        // Send to an agent and record this request.
        if let Some((_, agent)) = agent {
            agent.request(request).await
        } else {
            Err(HostError::NoAgentAvailable.into())
        }
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

    async fn handle_connection(self, stream: TcpStream, peer: SocketAddr) -> Result<()> {
        info!("New agent connection established: {}", peer.to_string());

        let mut agent = Agent::new(AgentInfo { name: "".to_string() }, peer);
        let mut agents = self.agents.lock().await;

        agent.start(stream).await;
        agents.insert(peer, agent);

        Ok(())
    }

    pub async fn agent_main(self) -> Result<()> {
        let mut listener = TcpListener::bind("0.0.0.0:8444").await?;

        while let Ok((stream, _)) = listener.accept().await {
            match stream.peer_addr() {
                Ok(peer) => {
                    info!("New agent connection established, with {}", peer);

                    let new_handler = self.clone();
                    tokio::spawn(async move {
                        let exit_result = new_handler.handle_connection(stream, peer).await;
                        info!("One agent task initialized with {:?}", exit_result);
                    });
                }
                Err(e) => warn!(
                    "New connection established, but the peer address is not clear: {}",
                    e
                ),
            }
        }
        Ok(())
    }
}
