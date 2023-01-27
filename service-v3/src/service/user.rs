/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2021-2023 上海应用技术大学 上应小风筝团队
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

use sqlx::PgPool;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::codegen::futures_core::Stream;
use tonic::{Request, Response, Status, Streaming};

pub use crate::service::gen::user as gen;

mod authserver;
mod tls;

type RpcClientPayload = gen::client_stream::Payload;
type RpcServerPayload = gen::server_stream::Payload;
type LoginResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<gen::ServerStream, Status>> + Send>>;

pub struct Frame(Vec<u8>);

async fn stream_translation_task(
    db: PgPool,
    mut in_stream: Streaming<gen::ClientStream>,
    out_sender: mpsc::Sender<Result<gen::ServerStream, Status>>,
) {
    // Send message from here to login_task through this channel.
    let (tx_sender, tx_receiver) = mpsc::channel::<RpcClientPayload>(16);
    // Receive message here from login_task through this channel.
    let (rx_sender, mut rx_receiver) = mpsc::channel::<RpcServerPayload>(16);

    // Launch redirection from login_task to the outer.
    tokio::spawn(async move {
        while let Some(payload_from_login_task) = rx_receiver.recv().await {
            let payload_to_outer = gen::ServerStream {
                payload: Some(payload_from_login_task),
            };
            if let Err(e) = out_sender.send(Ok(payload_to_outer)).await {
                tracing::error!(
                    "Could not send RpcServerPayload outside: {}, maybe out stream is closed?",
                    e
                );
                break;
            }
        }
    });
    // Launch login_task, go!!!
    tokio::spawn(login_task(db, rx_sender, tx_receiver));

    while let Some(result) = in_stream.next().await {
        match result {
            Ok(gen::ClientStream { payload }) => {
                if let Some(payload) = payload {
                    if let Err(e) = tx_sender.send(payload).await {
                        tracing::error!("Could not send RpcClientPayload: {}, maybe login_task is closed?", e);
                        break;
                    }
                } else {
                    tracing::error!("Unexpected: there is a None value in `oneof` field in proto, exit.");
                    break;
                }
            }
            Err(err) => {
                tracing::error!("Unexpected: failed to received ClientStream: {:?}", err);
                break;
            }
        }
    }
    tracing::trace!("stream ended");
}

async fn login_task(db: PgPool, tx: mpsc::Sender<RpcServerPayload>, rx: mpsc::Receiver<RpcClientPayload>) {
    // Prepare environment

    // Step 1

    // Step 2

    // ...

    // Query database
}

#[tonic::async_trait]
impl gen::user_service_server::UserService for super::KiteGrpcServer {
    type LoginStream = ResponseStream;

    async fn login(&self, request: Request<Streaming<gen::ClientStream>>) -> LoginResult<Self::LoginStream> {
        let mut in_stream = request.into_inner();
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
