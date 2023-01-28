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

use anyhow::{Context, Result};
use base64::Engine;
use hyper::StatusCode;
use scraper::{Html, Selector};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::TlsStream;

pub use constants::*;

use crate::service::user::authserver::client::Session;

mod constants {
    pub const DOMAIN: &str = "authserver.sit.edu.cn";
    /// 登录页. 第一次请求使用 GET 方法, 发送表单使用 POST 方法.
    pub const LOGIN_URI: &str = "/authserver/login";
    /// 访问登录后的信息页
    pub const AUTH_SERVER_HOME_URI: &str = "/authserver/index.do";
    /// 检查该用户登录是否需要验证码
    pub const NEED_CAPTCHA_URI: &str = "/authserver/needCaptcha.html";
    /// 登录时使用的 User-Agent
    pub const DESKTOP_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.99 Safari/537.36 Edg/97.0.1072.69";
    /// 验证码
    pub const CAPTCHA_URI: &str = "/authserver/captcha.html";
    /// 验证码识别服务
    pub const CAPTCHA_REORGANIZATION_URL: &str = "https://kite.sunnysab.cn/api/ocr/captcha";
}

mod tls_config {
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
}

mod client {
    use std::collections::HashMap;

    use anyhow::Result;
    use bytes::{BufMut, Bytes, BytesMut};
    use http::Response;
    use hyper::body::HttpBody;
    use hyper::client::conn;
    use hyper::Body;
    use tokio::io::{AsyncRead, AsyncWrite};
    use tokio_rustls::TlsStream;

    use super::constants;

    #[derive(Default)]
    struct CookieJar {
        pub inner: HashMap<String, String>,
    }

    impl CookieJar {
        fn parse_cookie(cookie: &str) -> Option<(&str, &str)> {
            // JSESSIONID=xSiUKpqm0lmjhDXB41_hhyxiNUa69u4xMnHkFOFS61E6VZ6Osp7S!-1266297679; path=/; HttpOnly
            cookie.split_once(';').and_then(|s| s.0.split_once('='))
        }

        pub fn append(&mut self, cookie: &str) {
            if let Some((k, v)) = Self::parse_cookie(cookie) {
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
    pub struct Session {
        /// 会话用的连接
        sender: conn::SendRequest<hyper::Body>,
        /// Cookie 存储
        cookie_jar: CookieJar,
    }

    impl Session {
        pub async fn create<T>(stream: TlsStream<T>) -> Result<Session>
        where
            T: AsyncRead + AsyncWrite + Send + Unpin + 'static,
        {
            let (sender, connection) = conn::handshake(stream).await?;

            // spawn a task to poll the connection and drive the HTTP state
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("Error in connection: {}", e);
                }
            });
            let result = Session {
                sender,
                cookie_jar: CookieJar::default(),
            };
            Ok(result)
        }

        async fn request(
            &mut self,
            method: hyper::Method,
            uri: &str,
            text_payload: Option<String>,
        ) -> Result<hyper::Response<Bytes>> {
            let mut builder = http::Request::builder()
                .method(method)
                .uri(uri)
                .header("Host", constants::DOMAIN)
                .header("User-Agent", constants::DESKTOP_USER_AGENT);

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
            let response = hyper::Response::from_parts(header, content);
            Ok(response)
        }

        pub async fn get(&mut self, url: &str) -> Result<hyper::Response<Bytes>> {
            self.request(hyper::Method::GET, url, None).await
        }

        pub async fn get_with_redirection(&mut self, url: &str, max_direction: u8) -> Result<hyper::Response<Bytes>> {
            let mut count = 0u8;
            let mut target = String::from(url);
            let mut response: Response<Bytes> = Default::default();

            assert!(max_direction > count);
            while count < max_direction {
                response = self.get(&target).await?;
                let status = response.status();

                if status == hyper::StatusCode::FOUND || status == hyper::StatusCode::MOVED_PERMANENTLY {
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

        pub async fn post(&mut self, url: &str, form: Option<&Vec<(&str, &str)>>) -> Result<hyper::Response<Bytes>> {
            let content = form.map(|items| {
                items
                    .into_iter()
                    .fold(String::new(), |c, (k, v)| c + &format!("{}={}&", k, v))
            });
            self.request(hyper::Method::POST, url, content).await
        }

        pub fn clear_cookie(&mut self) {
            self.cookie_jar.clear();
        }
    }
}

#[derive(Clone)]
pub struct Credential {
    /// 学号
    pub account: String,
    /// OA密码
    pub password: String,
}

impl Credential {
    pub fn new(account: String, password: String) -> Credential {
        Credential { account, password }
    }
}

pub struct PortalConnector {
    credential: Option<Credential>,
}

impl PortalConnector {
    pub fn new() -> Self {
        Self { credential: None }
    }

    pub fn user(mut self, credential: Credential) -> Self {
        Self {
            credential: Some(credential),
            ..self
        }
    }

    pub async fn bind<T>(self, stream: TlsStream<T>) -> Result<Portal>
    where
        T: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        let credential = self.credential.expect("Credential is required.");
        let session = Session::create(stream).await?;

        Ok(Portal { credential, session })
    }
}

/// 统一认证模块
pub struct Portal {
    credential: Credential,
    /// 登录会话
    session: Session,
}

/// Search in text by regex, and return the first group.
#[macro_export]
macro_rules! regex_find {
    ($text: expr, $pattern: expr) => {{
        let re = regex::Regex::new($pattern).unwrap();
        re.captures($text).map(|r| r[1].to_string())
    }};
}

struct IndexParameter {
    aes_key: String,
    lt: String,
}

impl Portal {
    /// Check whether captcha is need or not.
    async fn check_need_captcha(&mut self, account: &str) -> Result<bool> {
        let url = format!("{}?username={}&pwdEncrypt2=pwdEncryptSalt", NEED_CAPTCHA_URI, account);
        let response = self.session.get(&url).await?;

        let content = response.body();
        Ok(content.eq_ignore_ascii_case(b"true"))
    }

    /// Fetch captcha image.
    async fn fetch_captcha(&mut self) -> Result<Vec<u8>> {
        let response = self.session.get(CAPTCHA_URI).await?;
        let content = response.body();
        return Ok(content.to_vec());
    }

    /// Identify captcha images
    async fn recognize_captcha(&mut self, image_content: Vec<u8>) -> Result<String> {
        let standard_base64 = base64::engine::general_purpose::STANDARD;
        let captcha_base64 = standard_base64.encode(image_content);

        #[derive(serde::Deserialize)]
        struct RecognizeResult {
            data: Option<String>,
        }
        let response = reqwest::Client::new()
            .post(CAPTCHA_REORGANIZATION_URL)
            .body(captcha_base64)
            .send()
            .await
            .with_context(|| format!("Send captcha to ocr server"))?;
        let text = response.json::<RecognizeResult>().await?;
        return Ok(text.data.unwrap());
    }

    pub async fn get_person_name(&mut self) -> Result<String> {
        let response = self.session.get_with_redirection(AUTH_SERVER_HOME_URI, 5).await?;
        let text = String::from_utf8(response.body().to_vec())?;
        let document = Html::parse_document(&text);

        let name: String = document
            .select(&Selector::parse("#auth_siderbar > div.auth_username > span > span").unwrap())
            .next()
            .map(|e| e.text().collect())
            .unwrap_or_default();
        return Ok(name.trim().to_string());
    }

    async fn get_initial_parameters(&mut self) -> Result<IndexParameter> {
        self.session.clear_cookie();

        let response = self.session.get(LOGIN_URI).await?;
        let text = response.body().to_vec();
        let text = String::from_utf8(text)?;

        fn get_aes_key(text: &str) -> String {
            regex_find!(&text, r#"var pwdDefaultEncryptSalt = "(.*?)";"#).unwrap()
        }

        fn get_lt_field(text: &str) -> String {
            regex_find!(&text, r#"<input type="hidden" name="lt" value="(.*?)"/>"#).unwrap()
        }

        let aes_key = get_aes_key(&text);
        let lt = get_lt_field(&text);
        Ok(IndexParameter { aes_key, lt })
    }

    /// When submit password to `authserver.sit.edu.cn`, it's required to do AES and base64 algorithm with
    /// origin password. We use a key from HTML (generated and changed by `JSESSIONID`) to help with.
    fn generate_password_string(clear_password: &str, key: &str) -> String {
        use block_modes::block_padding::Pkcs7;
        use block_modes::{BlockMode, Cbc};
        type Aes128Cbc = Cbc<aes::Aes128, Pkcs7>;

        // Create an AES object.
        let cipher = Aes128Cbc::new_var(key.as_bytes(), &[0u8; 16]).unwrap();
        // Concat plaintext: 64 bytes random bytes and original password.
        let mut content = Vec::new();
        content.extend_from_slice(&[0u8; 64]);
        content.extend_from_slice(clear_password.as_bytes());

        // Encrypt with AES and use do base64 encoding.
        let encrypted_password = cipher.encrypt_vec(&content);
        base64::engine::general_purpose::STANDARD.encode(encrypted_password)
    }

    fn parse_err_message(text: &str) -> String {
        let document = Html::parse_document(text);
        let selector = Selector::parse("#msg").unwrap();

        document
            .select(&selector)
            .next()
            .map(|e| e.text().collect())
            .unwrap_or_default()
    }

    /// Login on campus official auth-server with student id and password.
    /// Return session if done successfully.
    pub async fn login<T>(mut self) -> Result<Self>
    where
        T: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        let credential = self.credential.clone();
        let IndexParameter { aes_key, lt } = self.get_initial_parameters().await?;
        let encrypted_password = Self::generate_password_string(&credential.password, &aes_key);

        /* Check if captcha is needed. */
        let captcha = if self.check_need_captcha(&credential.account).await? {
            let image = self.fetch_captcha().await?;
            self.recognize_captcha(image).await?
        } else {
            String::new()
        };

        /* Send login request */
        let form = vec![
            ("username", credential.account.as_str()),
            ("password", &encrypted_password),
            ("dllt", "userNamePasswordLogin"),
            ("execution", "e1s1"),
            ("_eventId", "submit"),
            ("rmShown", "1"),
            ("captchaResponse", &captcha),
            ("lt", &lt),
        ];
        let response = self.session.post(LOGIN_URI, Some(&form)).await?;
        if response.status() == StatusCode::FOUND {
            Ok(self)
        } else {
            let body = response.body().to_vec();
            let text = String::from_utf8(body)?;
            Err(anyhow::anyhow!("Err message: {}", Self::parse_err_message(&text)))
        }
    }
}
