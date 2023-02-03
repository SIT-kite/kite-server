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

use std::sync::Arc;

use once_cell::sync::OnceCell;
use rustls::ClientConfig;

static TLS_CONFIG: OnceCell<Arc<ClientConfig>> = OnceCell::new();

fn generate_tls_config() -> ClientConfig {
    fn default_cert_store() -> rustls::RootCertStore {
        let mut store = rustls::RootCertStore::empty();

        store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
            rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(ta.subject, ta.spki, ta.name_constraints)
        }));
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

pub fn get() -> &'static Arc<ClientConfig> {
    TLS_CONFIG.get_or_init(|| Arc::new(generate_tls_config()))
}
