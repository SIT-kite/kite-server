mod error;
pub mod host;
mod model;
mod protocol;

use crate::task::model::AgentInfoRequest;
use crate::task::protocol::{Request, RequestPayload, Response, ResponsePayload};
use futures_util::{SinkExt, StreamExt};
use log::*;
use rand::prelude::SliceRandom;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::Duration;
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

pub use error::Result;

#[derive(Debug, thiserror::Error)]
pub enum HostError {
    #[error("无可用的校内节点")]
    NoAgentAvailable,
    #[error("请求超时")]
    Timeout,
}

pub struct Agent {
    name: String,
    addr: String,
    channel: mpsc::Sender<Request>,
    queue: Vec<(usize, oneshot::Sender<Response>)>,
}

#[derive(Clone)]
pub struct Host {
    pub agents: Arc<Mutex<Vec<Agent>>>,
}

impl Host {
    async fn send(&self, request: Request, callback: oneshot::Sender<Response>) -> Result<()> {
        let mut rng = rand::thread_rng();
        let mut agents = self.agents.lock().await;
        let agent = agents[..].choose_mut(&mut rng);

        if let Some(agent) = agent {
            agent.queue.push((request.seq, callback));
            agent.channel.send(request).await;
            Ok(())
        } else {
            Err(HostError::NoAgentAvailable.into())
        }
    }

    async fn request(&self, request: RequestPayload) -> Result<Response> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::new(request), tx).await?;
        let response = tokio::time::timeout(Duration::from_millis(5000), rx)
            .await
            .map_err(|_| HostError::Timeout)?;

        Ok(response?)
    }

    async fn request_authentication(&mut self, stream: &mut WebSocketStream<TcpStream>) -> Result<bool> {
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
                    println!("binary accepted: {:?}", binary);
                    let r = bincode::deserialize::<Response>(&binary)?;
                    if let ResponsePayload::AgentInfo(info) = bincode::deserialize(&r.payload)? {
                        println!("{}", info.name);
                        return Ok(true);
                    }
                }
                _ => {
                    stream.close(None).await?;
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
    async fn handle_connection(mut self, _peer: SocketAddr, stream: TcpStream) -> Result<()> {
        let mut ws_stream = accept_async(stream).await.expect("Failed to accept");

        println!("New websocket established");
        if !self.request_authentication(&mut ws_stream).await? {
            return Ok(());
        }
        //
        // while let Some(msg) = ws_stream.next().await {
        //     let msg = msg?;
        //     if msg.is_text() || msg.is_binary() {
        //         ws_stream.send(msg).await?;
        //     }
        // }

        Ok(())
    }

    pub async fn websocket_main(self) {
        let mut listener = TcpListener::bind("0.0.0.0:8444").await.expect("Can't listen");

        while let Ok((stream, _)) = listener.accept().await {
            let peer = stream
                .peer_addr()
                .expect("connected streams should have a peer address");
            info!("Peer address: {}", peer);

            let new_handler = self.clone();
            tokio::spawn(async move {
                let r = new_handler.handle_connection(peer, stream).await;
                println!("{:?}", r);
            });
        }
    }
}
