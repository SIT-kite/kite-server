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

use std::fmt;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

// Copied from hyper-rustls
use hyper::body::Bytes;
use hyper::client::connect::{Connected, Connection};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::sync::mpsc;
use tokio_rustls::client::TlsStream;

/// A stream that might be protected with TLS.
#[allow(clippy::large_enum_variant)]
pub struct HttpsStream<T> {
    pub tls_stream: TlsStream<T>,
}

impl<T: AsyncRead + AsyncWrite + Connection + Unpin> Connection for HttpsStream<T> {
    fn connected(&self) -> Connected {
        let (tcp, tls) = self.tls_stream.get_ref();
        if tls.alpn_protocol() == Some(b"h2") {
            tcp.connected().negotiated_h2()
        } else {
            tcp.connected()
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for HttpsStream<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("Https(..)")
    }
}

impl<T> From<TlsStream<T>> for HttpsStream<T> {
    fn from(inner: TlsStream<T>) -> Self {
        HttpsStream { tls_stream: inner }
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> AsyncRead for HttpsStream<T> {
    #[inline]
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context, buf: &mut ReadBuf<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.tls_stream).poll_read(cx, buf)
    }
}

impl<T: AsyncWrite + AsyncRead + Unpin> AsyncWrite for HttpsStream<T> {
    #[inline]
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, io::Error>> {
        Pin::new(&mut self.tls_stream).poll_write(cx, buf)
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.tls_stream).poll_flush(cx)
    }

    #[inline]
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.tls_stream).poll_shutdown(cx)
    }
}
