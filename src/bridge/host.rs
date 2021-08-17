use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_bincode::{AsyncBincodeStream, AsyncDestination};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_tower::multiplex;
use tokio_tower::multiplex::MultiplexTransport;
use tower::{buffer::Buffer, Service, ServiceExt};

use crate::bridge::protocol::Tagged;
use crate::bridge::HostError;

use super::protocol::{RequestFrame, ResponseResult, Tagger};

fn on_service_error(e: anyhow::Error) {
    eprintln!("error handling: {:?}", e);
}

pub async fn ready<S: Service<RequestFrame>, RequestFrame>(svc: &mut S) -> Result<(), S::Error> {
    use futures_util::future::poll_fn;

    poll_fn(|cx| svc.poll_ready(cx)).await
}

type BincodeStream =
    AsyncBincodeStream<TcpStream, Tagged<ResponseResult>, Tagged<RequestFrame>, AsyncDestination>;

type Transport = MultiplexTransport<BincodeStream, Tagger>;
type MultiplexClient = multiplex::Client<
    Transport,
    //tokio_tower::Error<Transport, Tagged<RequestFrame>>,
    anyhow::Error,
    Tagged<RequestFrame>,
>;

#[derive(Clone)]
struct Client {
    address: String,
    client: Buffer<MultiplexClient, Tagged<RequestFrame>>,
}

impl Client {
    pub fn new(address: String, stream: TcpStream) -> Self {
        let stream: BincodeStream = AsyncBincodeStream::from(stream).for_async();
        let transport: Transport = multiplex::MultiplexTransport::new(stream, Tagger::default());
        let client = multiplex::Client::with_error_handler(transport, on_service_error);

        let buffered_client = Buffer::new(client, 1024);
        Self {
            address,
            client: buffered_client,
        }
    }

    pub async fn request(&mut self, request: RequestFrame) -> Result<ResponseResult> {
        let mut client = self.client.clone();
        let ready_client = client.ready().await.unwrap();

        let response = ready_client
            .call(Tagged::<RequestFrame>::from(request))
            .await
            .unwrap();

        Ok(response.v)
    }
}

#[derive(Clone)]
pub struct AgentManager {
    bind_addr: String,
    clients: Arc<RwLock<HashMap<String, Client>>>,
}

impl AgentManager {
    pub fn new(bind_addr: &str) -> Self {
        Self {
            bind_addr: bind_addr.to_string(),
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn add_client(&self, client: Client) {
        let mut clients = self.clients.write().await;
        clients.insert(client.address.clone(), client);
    }

    async fn remove_client(&self, address: String) {
        let mut clients = self.clients.write().await;
        clients.remove(&address);
    }

    async fn get_client(&self) -> Option<(String, Client)> {
        use rand::{seq::IteratorRandom, thread_rng};

        let mut rng = thread_rng();
        let clients = self.clients.read().await;
        if let Some((addr, client)) = clients.iter().choose(&mut rng) {
            return Some((addr.clone(), client.clone()));
        }
        None
    }

    pub async fn listen(&self) {
        // Bind a server socket
        let listener = TcpListener::bind(&self.bind_addr).await.unwrap();

        while let Ok((s, source_addr)) = listener.accept().await {
            let client = Client::new(source_addr.to_string(), s);
            self.add_client(client).await;
        }
    }

    pub async fn request(&self, request_frame: RequestFrame) -> Result<ResponseResult> {
        let client = self.get_client().await;

        // TODO: when the client is not available, remove it from clients.
        if let Some((_, mut client)) = client {
            client.request(request_frame).await
        } else {
            Err(HostError::NoAgentAvailable.into())
        }
    }
}
