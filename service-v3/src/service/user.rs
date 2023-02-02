/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2020-2023 上海应用技术大学 上应小风筝团队
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use std::pin::Pin;

use anyhow::{anyhow, Result};
use sqlx::PgPool;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::codegen::futures_core::Stream;
use tonic::{Request, Response, Status, Streaming};

pub use stream::VirtualStream;

use crate::authserver::{Credential, PortalConnector};
pub use crate::service::gen::user as gen;
use crate::service::gen::user::ClientStream;
use crate::service::user::gen::User;

type RpcClientPayload = gen::client_stream::Payload;
type RpcServerPayload = gen::server_stream::Payload;
type LoginResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<gen::ServerStream, Status>> + Send>>;

mod stream {
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
    use tokio::sync::mpsc;

    use super::{RpcClientPayload, RpcServerPayload};

    pub struct VirtualStream {
        rx_buffer: Vec<u8>,
        rx: mpsc::Receiver<RpcClientPayload>,
        tx: mpsc::Sender<RpcServerPayload>,
    }

    impl VirtualStream {
        pub fn new(rx: mpsc::Receiver<RpcClientPayload>, tx: mpsc::Sender<RpcServerPayload>) -> Self {
            Self {
                rx_buffer: Vec::with_capacity(1024),
                rx,
                tx,
            }
        }

        pub fn split(self) -> (mpsc::Receiver<RpcClientPayload>, mpsc::Sender<RpcServerPayload>) {
            (self.rx, self.tx)
        }
    }

    impl AsyncRead for VirtualStream {
        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            // Copy bytes from this_frame to target.
            // If target doesn't fit, copy remaining bytes to rx_buffer and wait next polling.
            fn copy_buffer(rx_buffer: &mut Vec<u8>, this_frame: Vec<u8>, target: &mut ReadBuf<'_>) {
                let copy_size = std::cmp::min(this_frame.len(), target.remaining());
                target.put_slice(&this_frame[..copy_size]);

                // If some bytes not copied yet, save them
                if copy_size < this_frame.len() {
                    rx_buffer.extend_from_slice(&this_frame[copy_size..]);
                }
            }

            // If last polling doesn't finish, continue and return the remaining bytes.
            if !self.rx_buffer.is_empty() {
                let this_frame = self.rx_buffer.clone();
                copy_buffer(&mut self.rx_buffer, this_frame, buf);
                return Poll::Ready(Ok(()));
            }
            // Try to poll from underlying channel :D
            match self.rx.poll_recv(cx) {
                Poll::Ready(payload) => {
                    if let Some(RpcClientPayload::TlsStream(content)) = payload {
                        if !content.is_empty() {
                            copy_buffer(&mut self.rx_buffer, content, buf);
                        }
                    }
                    Poll::Ready(Ok(()))
                }
                Poll::Pending => Poll::Pending,
            }
        }
    }

    impl AsyncWrite for VirtualStream {
        fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, std::io::Error>> {
            let len = buf.len();
            if len == 0 {
                return Poll::Ready(Ok(0));
            }

            let payload = RpcServerPayload::TlsStream(buf.to_vec());
            let fut = self.tx.send(payload);
            tokio::pin!(fut);

            use tokio::sync::mpsc::error::SendError;
            Future::poll(fut, cx)
                .map(|_state| Ok(len))
                .map_err(|e: SendError<RpcServerPayload>| std::io::Error::new(std::io::ErrorKind::BrokenPipe, e))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
            Poll::Ready(Ok(()))
        }
    }
}

async fn stream_translation_task(
    db: PgPool,
    stream_in: Streaming<gen::ClientStream>,
    channel_out: mpsc::Sender<Result<gen::ServerStream, Status>>,
) {
    async fn stream_translation_task_inner(
        db: PgPool,
        stream_in: Streaming<gen::ClientStream>,
        channel_out: mpsc::Sender<Result<gen::ServerStream, Status>>,
    ) -> Result<()> {
        // Send message from here to login_task through this channel.
        let (tx_sender, tx_receiver) = mpsc::channel::<RpcClientPayload>(16);
        // Receive message here from login_task through this channel.
        let (rx_sender, mut rx_receiver) = mpsc::channel::<RpcServerPayload>(16);

        fn mapping_inbound_stream(element: Result<ClientStream, Status>) -> Result<RpcClientPayload> {
            match element {
                Ok(stream) => stream
                    .payload
                    .ok_or_else(|| anyhow!("Expect client payload from in_stream but received None.")),
                Err(status) => Err(status.into()),
            }
        }

        // Launch login_task, go!!!
        tokio::spawn(login_task(db, rx_sender, tx_receiver));

        let mut in_stream = stream_in.map(mapping_inbound_stream);
        loop {
            tokio::select! {
                v = in_stream.next() => {
                    let v: Result<_> = v.unwrap();
                    // maybe it's unnecessary to handle error returned by channel
                    let _ = tx_sender
                    .send(v?)
                    .await?;
                    },
                v = rx_receiver.recv() => {
                    let payload_to_outer = gen::ServerStream {payload: v};
                    channel_out.send(Ok(payload_to_outer)).await?;
                },
            }
        }
    }

    let result = stream_translation_task_inner(db, stream_in, channel_out).await;
    if let Err(e) = result {
        tracing::trace!("stream_translation_task exits with error {}, ", e);
    }
}

async fn login_task(
    db: PgPool,
    tx: mpsc::Sender<RpcServerPayload>,
    mut rx: mpsc::Receiver<RpcClientPayload>,
) -> Result<()> {
    // Step 1: Get user credential from client
    let credential = if let Some(RpcClientPayload::Credential(oa)) = rx.recv().await {
        Credential::new(oa.account, oa.password)
    } else {
        // todo
        return Ok(());
    };

    println!("user = {}, password = {}", credential.account, credential.password);
    // Step 2: Create virtual stream, merge tx & rx -> stream
    let stream = VirtualStream::new(rx, tx);
    let mut portal = PortalConnector::new().user(credential).bind(stream).await?;

    // Step 3: Do login
    if let Err(e) = portal.try_login().await {
        println!("failed with {e}");
    } else {
        println!("login successfully");
    }

    // Step 4: Recycle virtual stream
    let stream = portal.shutdown().await?;
    let (mut rx, tx) = stream.split();

    tx.send(RpcServerPayload::User(User {
        uid: 10,
        account: "test user".to_string(),
        create_time: None,
    }))
    .await?;

    rx.close();
    // // Query database
    Ok(())
}

#[tonic::async_trait]
impl gen::user_service_server::UserService for super::KiteGrpcServer {
    type LoginStream = ResponseStream;

    async fn login(&self, request: Request<Streaming<gen::ClientStream>>) -> LoginResult<Self::LoginStream> {
        let in_stream = request.into_inner();
        // Send message to remote through this channel, tx is used for stream_redirection_task, which
        // can transfer message to here, and then the ServerStream can arrived rx, and be redirected to
        // out_stream
        let (to_remote_tx, to_remote_rx) = mpsc::channel(16);
        let out_stream = ReceiverStream::new(to_remote_rx);

        tokio::spawn(stream_translation_task(self.db.clone(), in_stream, to_remote_tx));
        // Function returns, but the stream continues...
        Ok(Response::new(Box::pin(out_stream) as Self::LoginStream))
    }
}
