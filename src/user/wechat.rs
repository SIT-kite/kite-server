use std::collections::HashMap;
use std::sync::Arc;

use actix_http::HttpMessage;
use actix_web::client::{Client, Connector};
use actix_web::web::Bytes;
use serde::Deserialize;
use serde_json;

use crate::config::CONFIG;
use crate::user::error::UserError;
use crate::user::error::WechatErrorType;

#[macro_export]
macro_rules! make_parameter {
    ($($para: expr => $val: expr), *) => {{
            let mut url = String::new();
            $( url = url + $para + "=" + $val + "&"; )*

            url.clone()
    }}
}

#[macro_export]
macro_rules! wx_function {

    ($fn_name: ident, $structure: ident, $addr: expr) => {
        async fn $fn_name(param: &str) -> Result<$structure, UserError> {
            let mut client = Client::default();
            let url = format!("{}?{}", $addr, param);
            let response = client.get(url).send().await;

            match response {
                // Note: Sending successfully, not receiving.
                Ok(mut r) => {
                    // Wechat server always return HTTP 200, with errcode field when parameter error.
                    if r.status().is_success() {
                        // Resolve json string or give an empty json.
                        let body_string = r.body().await.unwrap_or(Bytes::from("{}"));
                        let body_json: $structure = serde_json::from_slice(body_string.as_ref()).unwrap();
                        return Ok(body_json);
                    }
                    // TODO:
                    Err(UserError::NetworkError)
                }
                Err(_) => {
                    Err(UserError::NetworkError)
                }
            }
        } // End of function.
    }; // End of pattern.
} // End of macro_rules.


#[derive(Debug, Deserialize)]
pub struct SessionResponse {
    // When error occurred
    errcode: Option<i32>,
    errmsg: Option<String>,
    // Successful.
    session_key: Option<String>,
    openid: Option<String>,
    // Unsupported:
    // unionid: Option<String>,
}


#[derive(Debug, Deserialize)]
pub struct AccessTokenResponse {
    access_token: Option<String>,
    expires_in: Option<i32>,
    errcode: Option<i32>,
    errmsg: Option<String>,
}


wx_function!(_get_session_key, SessionResponse, "https://api.weixin.qq.com/sns/jscode2session");
wx_function!(_get_access_token, AccessTokenResponse, "https://api.weixin.qq.com/cgi-bin/token");


pub async fn get_session_by_code(wechat_code: &str) -> Result<SessionResponse, UserError> {
    _get_session_key(make_parameter!(
        "appid" => &CONFIG.wechat_appid,
        "secret" => &CONFIG.wechat_secret,
        "js_code" => wechat_code,
        "grant_type" => "authorization_code"
    ).as_str()).await
}


pub async fn get_access_token() -> Result<AccessTokenResponse, UserError> {
    _get_access_token(make_parameter!(
        "appid" => &CONFIG.wechat_appid,
        "secret" => &CONFIG.wechat_secret,
        "grant_type" => "client_credential"
    ).as_str()).await
}
