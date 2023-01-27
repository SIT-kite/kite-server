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
use std::task::Poll;

use anyhow::Context;
use hyper::{service::Service, Uri};
use once_cell::sync::OnceCell;
use rustls::{ClientConfig, OwnedTrustAnchor};
use tokio::io::DuplexStream;
use tokio_rustls::TlsConnector;

use crate::proxy::stream::EncryptedStream;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

static TLS_CONFIG: OnceCell<Arc<ClientConfig>> = OnceCell::new();

fn generate_tls_config() -> ClientConfig {
    fn default_cert_store() -> rustls::RootCertStore {
        let mut store = rustls::RootCertStore::empty();

        store.add_server_trust_anchors(
            webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
                OwnedTrustAnchor::from_subject_spki_name_constraints(ta.subject, ta.spki, ta.name_constraints)
            }),
        );
        store
    }

    fn default_client_config() -> ClientConfig {
        ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(default_cert_store())
            .with_no_client_auth()
    }

    default_client_config()
}

fn get_tls_config() -> &'static Arc<ClientConfig> {
    TLS_CONFIG.get_or_init(|| Arc::new(generate_tls_config()))
}

pub struct HttpsConnector {
    bottom: Option<DuplexStream>,
}

impl HttpsConnector {
    pub fn new(bottom: DuplexStream) -> Self {
        Self { bottom: Some(bottom) }
    }
}

impl Service<Uri> for HttpsConnector {
    type Response = EncryptedStream<DuplexStream>;
    type Error = BoxError;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, BoxError>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, dst: Uri) -> Self::Future {
        let scheme = dst.scheme().expect("Scheme is expected on the heading of url.");
        assert_eq!(scheme, &http::uri::Scheme::HTTPS);

        let host = dst.host().unwrap();
        let tls_server_name = rustls::ServerName::try_from(host)
            .with_context(|| format!("Failed to parse TLS ServerName from: {}", host))
            .unwrap();

        let socket_stream = self.bottom.take().unwrap();
        let f = async move {
            let config = get_tls_config().clone();

            let connector = TlsConnector::from(config);
            let tls = connector.connect(tls_server_name, socket_stream).await?;
            Ok(EncryptedStream::new(tls))
        };
        Box::pin(f)
    }
}
