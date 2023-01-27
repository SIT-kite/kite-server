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

// Copied from hyper-rustls

use std::convert::TryFrom;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::{fmt, io};

use hyper::client::HttpConnector;
use hyper::{client::connect::Connection, service::Service, Uri};
use rustls::{ClientConfig, OwnedTrustAnchor};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::TlsConnector;

use crate::stream::HttpsStream;

use super::HttpsConnector;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub struct ConnectorBuilder {
    tls_config: ClientConfig,
}

impl ConnectorBuilder {
    pub fn default_cert_store() -> rustls::RootCertStore {
        let mut store = rustls::RootCertStore::empty();

        store.add_server_trust_anchors(
            webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
                OwnedTrustAnchor::from_subject_spki_name_constraints(ta.subject, ta.spki, ta.name_constraints)
            }),
        );
        store
    }

    pub fn default_client_config() -> ClientConfig {
        ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(Self::default_cert_store())
            .with_no_client_auth()
    }

    pub fn new() -> Self {
        let tls_config = Self::default_client_config();

        Self { tls_config }
    }
    pub fn build(self) -> HttpsConnector<HttpConnector> {
        let mut http = HttpConnector::new();
        // HttpConnector won't enforce scheme, but HttpsConnector will
        http.enforce_http(false);

        HttpsConnector {
            http,
            tls_config: std::sync::Arc::new(self.tls_config),
        }
    }
}

#[derive(Clone)]
pub struct HttpsConnector<T> {
    http: T,
    tls_config: Arc<ClientConfig>,
}

impl<T> Service<Uri> for HttpsConnector<T>
where
    T: Service<Uri>,
    T::Response: Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    T::Future: Send + 'static,
    T::Error: Into<BoxError>,
{
    type Response = HttpsStream<T::Response>;
    type Error = BoxError;

    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<HttpsStream<T::Response>, BoxError>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.http.poll_ready(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
            Poll::Pending => Poll::Pending,
        }
    }

    fn call(&mut self, dst: Uri) -> Self::Future {
        // dst.scheme() would need to derive Eq to be matchable;
        // use an if cascade instead
        if let Some(sch) = dst.scheme() {
            if sch == &http::uri::Scheme::HTTPS {
                let cfg = self.tls_config.clone();
                let mut hostname = dst.host().unwrap_or_default();

                // Remove square brackets around IPv6 address.
                if let Some(trimmed) = hostname.strip_prefix('[').and_then(|h| h.strip_suffix(']')) {
                    hostname = trimmed;
                }

                let hostname = match rustls::ServerName::try_from(hostname) {
                    Ok(dnsname) => dnsname,
                    Err(_) => {
                        let err = io::Error::new(io::ErrorKind::Other, "invalid dnsname");
                        return Box::pin(async move { Err(Box::new(err).into()) });
                    }
                };
                let connecting_future = self.http.call(dst);

                let f = async move {
                    let tcp = connecting_future.await.map_err(Into::into)?;
                    let connector = TlsConnector::from(cfg);
                    let tls = connector
                        .connect(hostname, tcp)
                        .await
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                    Ok(HttpsStream { tls_stream: tls })
                };
                Box::pin(f)
            } else {
                let err = io::Error::new(io::ErrorKind::Other, format!("Unsupported scheme {}", sch));
                Box::pin(async move { Err(err.into()) })
            }
        } else {
            let err = io::Error::new(io::ErrorKind::Other, "Missing scheme");
            Box::pin(async move { Err(err.into()) })
        }
    }
}
