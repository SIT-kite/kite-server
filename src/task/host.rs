use super::model::{AgentInfo, AgentInfoRequest};
use super::protocol::{Request, RequestPayload, Response, ResponsePayload};
use super::{Agent, Host, HostError, RequestQueue, Result};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot, Mutex};
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
        }
    }

    /// Send a request to the agent.
    pub async fn send(&mut self, request: Request) -> Result<()> {
        if let Some(channel) = &self.channel {
            channel.send(request);
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
        }
    }

    /// Sender loop: send requests to agent over ws.
    async fn sender_loop<T, Item>(mut socket_tx: T, mut message_rx: mpsc::UnboundedReceiver<Request>)
    where
        T: SinkExt<Item> + std::marker::Unpin,
        Item: From<Message>,
    {
        while let Some(request) = message_rx.recv().await {
            if let Ok(request) = bincode::serialize(&request) {
                if let Err(_) = socket_tx.send(Message::Binary(request).into()).await {
                    ()
                }
            }
        }
    }

    /// Receiver loop: receive responses from the agent and transfer it to the requester.
    async fn receiver_loop<T>(mut socket_rx: T, queue: Arc<Mutex<RequestQueue>>)
    where
        T: StreamExt + std::marker::Unpin,
        T::Item: Into<std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>,
    {
        while let Some(msg_result) = socket_rx.next().await {
            match msg_result.into() {
                Ok(Message::Binary(content)) => {
                    if let Ok(resp) = bincode::deserialize::<Response>(&content) {
                        Self::dispatch_response(queue.clone(), resp).await;
                    }
                }
                Ok(Message::Pong(_)) => (),
                _ => {
                    // socket_rx.close(None).await;
                }
            }
        }
    }

    /// Start listening response from the agent and request from web.
    pub async fn start(&mut self, ws_stream: WebSocketStream<TcpStream>) {
        let (send_half, recv_half) = ws_stream.split();
        let (tx, rx) = mpsc::unbounded_channel();

        // TODO: Close all channels and destroy Agent instance.
        self.channel = Some(tx);
        tokio::spawn(Self::receiver_loop(recv_half, self.queue.clone()));
        tokio::spawn(Self::sender_loop(send_half, rx));
    }
}

impl Host {
    /// Select an agent randomly and send request packet.
    async fn send(&self, request: Request, callback: oneshot::Sender<Response>) -> Result<()> {
        use rand::prelude::IteratorRandom;

        let mut rng = rand::thread_rng();
        let mut agents = self.agents.lock().await;

        let agent = agents.iter_mut().choose(&mut rng);
        // Send to an agent and record this request.
        if let Some((_, agent)) = agent {
            let seq = request.seq;
            agent.send(request).await?;

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
            let peer = stream.peer_addr()?;

            let new_handler = self.clone();
            tokio::spawn(async move {
                let r = new_handler.handle_connection(peer, stream).await;
                println!("{:?}", r);
            });
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
