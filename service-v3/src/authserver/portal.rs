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

use anyhow::{Context, Result};
use base64::Engine;
use http::StatusCode;
use scraper::{Html, Selector};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

use super::constants::*;
use super::Session;

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

    pub fn user(self, credential: Credential) -> Self {
        Self {
            credential: Some(credential),
            ..self
        }
    }

    pub async fn bind<T>(self, stream: T) -> Result<Portal<T>>
    where
        T: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        let credential = self.credential.expect("Credential is required.");

        // Prepare client configuration which used to handshake
        let config = crate::authserver::tls_get().clone();
        let connector = tokio_rustls::TlsConnector::from(config);
        let server_name = "authserver.sit.edu.cn".try_into().unwrap();

        // Bind IO with TLS config (do some TLS initializing operation)
        // Maybe the connect function should not be a async function?
        let stream = connector.connect(server_name, stream).await.unwrap();
        let session = Session::create(stream).await?;

        Ok(Portal { credential, session })
    }
}

/// 统一认证模块
pub struct Portal<T>
where
    T: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    credential: Credential,
    /// 登录会话
    session: Session<T>,
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

impl<T> Portal<T>
where
    T: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
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
        use base64::engine::general_purpose::STANDARD as base64_standard;
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
        base64_standard.encode(encrypted_password)
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
    pub async fn try_login(&mut self) -> Result<()> {
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
        // Login post is the last request.
        // Send `Connection: close` to make the server close the connection actively,
        // so will the without_shutdown method in the closure in Session::create return.
        // Then original stream will be returned.
        let header = vec![("Content-Type", "application/x-www-form-urlencoded")];
        let response = self.session.post(LOGIN_URI, form, header).await?;
        if response.status() == StatusCode::FOUND {
            Ok(())
        } else {
            let body = response.body().to_vec();
            let text = String::from_utf8(body)?;
            Err(anyhow::anyhow!("{} (from authserver)", Self::parse_err_message(&text)))
        }
    }

    pub async fn shutdown(mut self) -> Result<T> {
        self.session.request_close_connection().await?;

        match self.session.wait_for_shutdown().await {
            Ok(mut s) => {
                // Close TLS connection (send `TLS Encrypted Alert` message)
                s.shutdown().await?;

                // Return original stream back
                let (os, _) = s.into_inner();
                Ok(os)
            }
            Err(e) => Err(e),
        }
    }
}
