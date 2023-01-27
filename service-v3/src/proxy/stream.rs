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

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use anyhow::Result;
use hyper::client::connect::{Connected, Connection};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, DuplexStream, ReadBuf};
use tokio_rustls::client::TlsStream;

/// The KiteTls, perform TLS stream over a custom channel, instead of socket
pub struct EncryptedStream<T>(TlsStream<T>)
where
    T: AsyncRead + AsyncWrite + Unpin;

impl<T> EncryptedStream<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(stream: TlsStream<T>) -> Self {
        Self(stream)
    }
}

impl<T: AsyncRead + AsyncWrite + Connection + Unpin> Connection for EncryptedStream<T> {
    fn connected(&self) -> Connected {
        let (tcp, tls) = self.0.get_ref();
        if tls.alpn_protocol() == Some(b"h2") {
            tcp.connected().negotiated_h2()
        } else {
            tcp.connected()
        }
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> AsyncRead for EncryptedStream<T> {
    #[inline]
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context, buf: &mut ReadBuf<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl<T: AsyncWrite + AsyncRead + Unpin> AsyncWrite for EncryptedStream<T> {
    #[inline]
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, io::Error>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    #[inline]
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}
