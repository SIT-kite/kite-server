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

use std::collections::HashMap;

use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use hyper::body::HttpBody;
use hyper::client::conn;
use hyper::{Body, Method, Response, StatusCode};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::oneshot;
use tokio_rustls::client::TlsStream;

use super::constants::*;

#[derive(Default)]
struct CookieJar {
    pub inner: HashMap<String, String>,
}

impl CookieJar {
    fn parse_line(cookie: &str) -> Option<(&str, &str)> {
        // JSESSIONID=xSiUKpqm0lmjhDXB41_hhyxiNUa69u4xMnHkFOFS61E6VZ6Osp7S!-1266297679; path=/; HttpOnly
        cookie.split_once(';').and_then(|s| s.0.split_once('='))
    }

    pub fn append(&mut self, cookie: &str) {
        if let Some((k, v)) = Self::parse_line(cookie) {
            // This method will override the old one if k already exists.
            self.inner.insert(k.to_string(), v.to_string());
        }
    }

    pub fn to_string(&self) -> Option<String> {
        if self.inner.is_empty() {
            return None;
        }
        let result = self
            .inner
            .iter()
            .fold(String::new(), |s, (k, v)| s + &*format!("{}={};", k, v));
        return Some(result);
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

/// 会话. 用于在 Http 连接上虚拟若干不同用户的会话.
pub struct Session<T>
where
    T: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    /// 会话用的连接
    sender: conn::SendRequest<Body>,
    /// Cookie 存储
    cookie_jar: CookieJar,
    /// receiver to released TLS stream
    released_rx: oneshot::Receiver<TlsStream<T>>,
}

impl<T> Session<T>
where
    T: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    /// Initialize a TLS connection and wait for next operations
    pub async fn create(stream: TlsStream<T>) -> Result<Session<T>> {
        // Client hello !!
        let (sender, connection) = conn::handshake(stream).await?;

        // A task to poll the connection and drive the HTTP state
        // If the connection closed, return tls stream.
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            // Connection: close is set on the last request, the /login post, to cause the server to
            // close TLS layer connection actively.
            // So will the without_shutdown method return.
            let result = connection.without_shutdown().await;
            if let Ok(part) = result {
                let _ = tx.send(part.io);
            }
        });
        let result = Session {
            sender,
            cookie_jar: CookieJar::default(),
            released_rx: rx,
        };
        Ok(result)
    }

    /// A simple wrapper, which can send a HTTP(S) request
    async fn request(
        &mut self,
        method: Method,
        uri: &str,
        text_payload: Option<String>,
        header: Vec<(String, String)>,
    ) -> Result<Response<Bytes>> {
        let mut builder = http::Request::builder()
            .method(method)
            .uri(uri)
            .header("Host", SERVER_NAME)
            .header("User-Agent", DESKTOP_USER_AGENT);
        for (k, v) in header {
            builder = builder.header(k, v);
        }

        if let Some(cookie) = self.cookie_jar.to_string() {
            builder = builder.header("Cookie", cookie);
        }
        let body = text_payload.map(Body::from).unwrap_or_else(|| Body::empty());
        let request = builder.body(body)?;

        /* Send request and receive header*/
        let response = self.sender.send_request(request).await?;
        let (header, mut body) = response.into_parts();
        // Store cookies
        if let Some(cookies) = header.headers.get("Set-Cookie") {
            self.cookie_jar.append(cookies.to_str().unwrap());
        }
        // Pull data chunks
        let mut content = BytesMut::new();
        while let Some(chunk) = body.data().await {
            let chunk = chunk?;
            content.put(chunk);
        }
        let content = Bytes::from(content);
        let response = Response::from_parts(header, content);
        Ok(response)
    }

    pub async fn get(&mut self, url: &str) -> Result<Response<Bytes>> {
        self.request(Method::GET, url, None, vec![]).await
    }

    pub async fn get_with_redirection(&mut self, url: &str, max_direction: u8) -> Result<Response<Bytes>> {
        let mut count = 0u8;
        let mut target = String::from(url);
        let mut response: Response<Bytes> = Default::default();

        assert!(max_direction > count);
        while count < max_direction {
            response = self.get(&target).await?;
            let status = response.status();

            if status == StatusCode::FOUND || status == StatusCode::MOVED_PERMANENTLY {
                let new_target = response.headers().get("Location").unwrap();
                target = new_target.to_str()?.to_string();

                count += 1;
            }
        }
        if count == max_direction {
            Err(anyhow::anyhow!("Max redirection count exceeds."))
        } else {
            Ok(response)
        }
    }

    pub async fn post(
        &mut self,
        url: &str,
        form: Vec<(&str, &str)>,
        header: Vec<(&str, &str)>,
    ) -> Result<Response<Bytes>> {
        let content = if !form.is_empty() {
            let s = form
                .into_iter()
                .fold(String::new(), |c, (k, v)| c + &format!("{}={}&", k, v));
            Some(s)
        } else {
            None
        };
        let header = header
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        self.request(Method::POST, url, content, header).await
    }

    pub fn clear_cookie(&mut self) {
        self.cookie_jar.clear();
    }

    pub async fn wait_for_shutdown(self) -> Result<TlsStream<T>> {
        self.released_rx.await.map_err(Into::into)
    }
}
