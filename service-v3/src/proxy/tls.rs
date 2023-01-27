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

use std::convert::TryFrom;
use std::io;
use std::net::ToSocketAddrs;
use std::sync::Arc;

use anyhow::Result;
use tokio::io::{copy, AsyncReadExt, AsyncWriteExt};
use tokio_rustls::rustls::{self, ClientConfig, OwnedTrustAnchor};
use tokio_rustls::TlsConnector;

/// The KiteTls, perform TLS stream over a custom channel, instead of socket
struct KiteTls {
    /// Client config, used to configure and manage client information...
    /// It's not related to the underlying connection.
    config: Arc<rustls::ClientConfig>,
}

/// The builder
struct KiteTlsBuilder {
    config: rustls::ClientConfig,
}

impl KiteTlsBuilder {
    pub fn default() -> Self {
        Self {
            config: Self::default_client_config(),
        }
    }

    pub fn new_with_config(client_config: ClientConfig) -> Self {
        Self { config: client_config }
    }

    pub fn build(self) -> KiteTls {
        KiteTls {
            config: Arc::from(self.config),
        }
    }

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
        rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(Self::default_cert_store())
            .with_no_client_auth()
    }
}

impl KiteTls {
    fn builder() -> KiteTlsBuilder {
        KiteTlsBuilder::default()
    }

    /// Give a TLS channel (`tls_stream`), and perform encrypted tls connection on it.
    /// Parameter `domain` indicates the server you want to connect.
    async fn request_with_channel<T: AsyncReadExt + AsyncWriteExt + Unpin>(
        &self,
        domain: &str,
        plaintext_stream: T,
        tls_stream: T,
    ) -> Result<()> {
        let tls_server_name = rustls::ServerName::try_from(domain)?;

        let connector = TlsConnector::from(self.config.clone());
        let encrypted_stream = connector.connect(tls_server_name, tls_stream).await?;

        let (mut plaintext_reader, mut plaintext_writer) = tokio::io::split(plaintext_stream);
        let (mut reader, mut writer) = tokio::io::split(encrypted_stream);

        tokio::select! {
            ret = tokio::io::copy(&mut reader, &mut plaintext_writer) => {
                ret?;
            },
            ret = tokio::io::copy(&mut plaintext_reader, &mut writer) => {
                ret?;
            }
        }
        Ok(())
    }
}
