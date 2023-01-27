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

use std::collections::HashMap;
use std::io::Read;

use anyhow::Result;
use base64::Engine;
use hyper::{client::HttpConnector, Method, StatusCode};
use hyper_rustls::HttpsConnector;
use scraper::{Html, Selector};

/// 登录页. 第一次请求使用 GET 方法, 发送表单使用 POST 方法.
const LOGIN_URL: &str = "https://authserver.sit.edu.cn/authserver/login";
/// 访问登录后的信息页
const AUTH_SERVER_HOME_URL: &str = "https://authserver.sit.edu.cn/authserver/index.do";
/// 检查该用户登录是否需要验证码
const NEED_CAPTCHA_URL: &str = "https://authserver.sit.edu.cn/authserver/needCaptcha.html";
/// 登录时使用的 User-Agent
const DESKTOP_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.99 Safari/537.36 Edg/97.0.1072.69";
/// 验证码
const CAPTCHA_URL: &str = "https://authserver.sit.edu.cn/authserver/captcha.html";
/// 验证码识别服务
const CAPTCHA_REORGANIZATION_URL: &str = "https://kite.sunnysab.cn/api/ocr/captcha";

type HyperClientType = hyper::Client<HttpsConnector<HttpConnector>, hyper::Body>;

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
}

/// 会话. 用于在 Http 连接上虚拟若干不同用户的会话.
pub struct Session {
    /// 会话用的连接
    pub client: HyperClientType,
    /// Cookie 存储
    cookie_jar: CookieJar,
}

impl Session {
    pub fn new(client: HyperClientType) -> Session {
        Session {
            client,
            cookie_jar: CookieJar::default(),
        }
    }

    async fn request(
        &mut self,
        method: hyper::Method,
        url: &str,
        form: Option<&Vec<(&str, &str)>>,
    ) -> Result<hyper::Response<hyper::Body>> {
        self.client.ge

        let mut builder = self.client.request(method, url);

        builder = builder.header("User-Agent", DESKTOP_USER_AGENT);
        if let Some(cookie) = self.cookie_jar.to_string() {
            builder = builder.header("Cookie", cookie);
        }
        if let Some(form) = form {
            builder = builder.form(form);
        }
        let response = builder.send().await?;
        if let Some(cookies) = response.headers().get("Set-Cookie") {
            self.cookie_jar.append(cookies.to_str().unwrap());
        }
        return Ok(response);
    }
    pub async fn get(&mut self, url: &str) -> Result<hyper::Response<hyper::Body>> {
        self.request(Method::GET, url, None).await
    }

    pub async fn post(&mut self, url: &str, form: Option<&Vec<(&str, &str)>>) -> Result<hyper::Response<hyper::Body>> {
        self.request(Method::POST, url, form).await
    }
}

/// 统一认证模块
pub struct Portal {
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

impl Portal {
    /// Check whether captcha is need or not.
    async fn check_need_captcha(&mut self, account: &str) -> Result<bool> {
        let url = format!("{}?username={}&pwdEncrypt2=pwdEncryptSalt", NEED_CAPTCHA_URL, account);
        let response = self.session.get(&url).await?;
        Ok(response.text().await? == "true")
    }

    /// Fetch captcha image.
    async fn fetch_captcha(&mut self) -> Result<Vec<u8>> {
        let response = self.session.get(CAPTCHA_URL).await?;
        return Ok(response.bytes().await?.to_vec());
    }

    /// Identify captcha images
    async fn recognize_captcha(&mut self, image_content: Vec<u8>) -> Result<String> {
        let captcha_base64 = base64::engine::general_purpose::STANDARD.encode(image_content);

        #[derive(serde::Deserialize)]
        struct RecognizeResult {
            // code: i32,
            // msg: Option<String>,
            data: Option<String>,
        }
        let response = self
            .session
            .client
            .post(CAPTCHA_REORGANIZATION_URL)
            .body(captcha_base64)
            .send()
            .await?;
        let text = response.json::<RecognizeResult>().await?;

        return Ok(text.data.unwrap());
    }

    pub async fn get_person_name(&mut self) -> Result<String> {
        let mut target = AUTH_SERVER_HOME_URL.to_string();
        let mut response: hyper::Response;
        loop {
            response = self.session.get(&target).await?;
            if response.status() == StatusCode::FOUND {
                target = response
                    .headers()
                    .get("Location")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
            } else {
                break;
            }
        }
        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let name: String = document
            .select(&Selector::parse("#auth_siderbar > div.auth_username > span > span").unwrap())
            .next()
            .map(|e| e.text().collect())
            .unwrap_or_default();
        return Ok(name.trim().to_string());
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

    fn get_aes_key(text: &str) -> String {
        regex_find!(&text, r#"var pwdDefaultEncryptSalt = "(.*?)";"#).unwrap()
    }

    fn get_lt_field(text: &str) -> String {
        regex_find!(&text, r#"<input type="hidden" name="lt" value="(.*?)"/>"#).unwrap()
    }

    fn get_err_message(text: &str) -> String {
        let document = Html::parse_document(text);

        let err_message: String = document
            .select(&Selector::parse("#msg").unwrap())
            .next()
            .map(|e| e.text().collect())
            .unwrap_or_default();
    }

    /// Login on campus official auth-server with student id and password.
    /// Return session if done successfully.
    pub async fn login(raw_client: HyperClientType, credential: &Credential) -> Result<Self> {
        let session = Session::new(raw_client);
        let mut portal = Portal { session };

        // Request login page to get encrypt key and so on.
        let index_html = portal.session.get(LOGIN_URL).await?.text().await?;
        let aes_key = Self::get_aes_key(&index_html);
        let lt = Self::get_lt_field(&index_html);
        let encrypted_password = Self::generate_password_string(&credential.password, &aes_key);

        let need_captcha = portal.check_need_captcha(&credential.account).await?;
        let mut captcha = String::default();
        if need_captcha {
            let image = portal.fetch_captcha().await?;
            captcha = portal.recognize_captcha(image).await?;
        }
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
        let response = portal.session.post(LOGIN_URL, Some(&form)).await?;
        if response.status() == StatusCode::FOUND {
            return Ok(portal);
        }
        Err(anyhow::anyhow!(
            "Err message: {}",
            Self::get_err_message(response.body().to_string())
        ))
    }
}