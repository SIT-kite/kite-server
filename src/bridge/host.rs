use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};
use std::sync::Arc;

use anyhow::Result;
use async_bincode::{AsyncBincodeStream, AsyncDestination};
use chrono::Local;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_tower::multiplex;
use tokio_tower::multiplex::MultiplexTransport;
use tower::{buffer::Buffer, Service, ServiceExt};

use super::protocol::Tagged;
use super::protocol::{RequestFrame, ResponseResult, Tagger};
use super::{AgentStatus, HostError};

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
    count: Arc<AtomicU32>,
    last_use: Arc<AtomicI64>,
    client: Buffer<MultiplexClient, Tagged<RequestFrame>>,
}

impl Client {
    pub fn new(address: String, stream: TcpStream) -> Self {
        let stream: BincodeStream = AsyncBincodeStream::from(stream).for_async();
        let transport: Transport = multiplex::MultiplexTransport::new(stream, Tagger::default());
        let client = multiplex::Client::with_error_handler(transport, on_service_error);

        let buffered_client = Buffer::new(client, 1024);

        let current_time = Local::now().timestamp_millis();
        Self {
            address,
            count: Arc::new(AtomicU32::default()),
            last_use: Arc::new(AtomicI64::new(current_time)),
            client: buffered_client,
        }
    }

    pub async fn request(&mut self, request: RequestFrame) -> Result<ResponseResult> {
        let mut client = self.client.clone();
        let ready_client = client.ready().await.map_err(|_| HostError::Disconnected)?;

        let response = ready_client
            .call(Tagged::<RequestFrame>::from(request))
            .await
            .map_err(|_| HostError::Timeout)?;

        self.count.fetch_add(1, Ordering::SeqCst);
        self.last_use
            .store(Local::now().timestamp_millis(), Ordering::Release);
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

    pub async fn get_client_list(&self) -> Vec<AgentStatus> {
        let clients = self.clients.read().await;
        clients
            .iter()
            .map(|(socket_address, client)| AgentStatus {
                name: "".to_string(),
                intranet_addr: "".to_string(),
                external_addr: socket_address.clone(),
                requests: client.count.load(Ordering::Acquire),
            })
            .collect()
    }

    pub async fn listen(&self) {
        // Bind a server socket
        let listener = TcpListener::bind(&self.bind_addr)
            .await
            .expect("Could not bind to server.");

        while let Ok((s, source_addr)) = listener.accept().await {
            let client = Client::new(source_addr.to_string(), s);
            self.add_client(client).await;
        }
    }

    pub async fn request(&self, request_frame: RequestFrame) -> Result<ResponseResult> {
        let client = self.get_client().await;

        if let Some((remote_addr, mut client)) = client {
            let result = client.request(request_frame).await;
            // Remove client if error occurred in transport layer, like agent disconnection.
            if result.is_err() {
                self.remove_client(remote_addr).await;
            }
            result
        } else {
            Err(HostError::NoAgentAvailable.into())
        }
    }
}
