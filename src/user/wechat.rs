use std::collections::HashMap;
use std::sync::Arc;

use actix_http::HttpMessage;
use actix_web::client::{Client, Connector};
use actix_web::web::Bytes;
use failure::Fail;
use serde::Deserialize;
use serde_json;

use crate::config::CONFIG;
use crate::error::ServerError;

#[macro_export]
macro_rules! make_parameter {
    // Concatenate web form parameters to a string.
    // Example:
    // make_parameter!("a" => "1", "b" => 2);
    // will returned "a=1&b=2&"
    ($($para: expr => $val: expr), *) => {{
        let mut url = String::new();
        $( url = url + $para + "=" + $val + "&"; )*

        url.clone()
    }}
}

#[macro_export]
macro_rules! wx_function {

    ($fn_name: ident, $structure: ident, $addr: expr) => {
        async fn $fn_name(param: &str) -> Result<$structure, ServerError> {
            // create actix-web client for request.
            let mut client = Client::default();
            let url = format!("{}?{}", $addr, param);
            let response = client.get(url).send().await;

            match response {
                // Note: Sending successfully, not receiving.
                Ok(mut r) => {
                    // Wechat server always return HTTP 200, with errcode field when parameter error.
                    if r.status().is_success() {
                        // Resolve json string or give an empty json.
                        let body_string = r.body().await?;
                        let body_json: $structure = serde_json::from_slice(body_string.as_ref())?;
                        return Ok(body_json);
                    }
                    Err(ServerError::from(format!("Wechat interface returned:{}", r.status())))
                }
                Err(_) => {
                    Err(ServerError::from(String::from("Failed to connect wechat interface.")))
                }
            }
        } // End of function.
    }; // End of pattern.
} // End of macro_rules.


#[derive(Debug, Deserialize)]
struct SessionResponse {
    // When error occurred
    pub errcode: Option<u16>,
    pub errmsg: Option<String>,
    // Successful.
    pub session_key: Option<String>,
    pub openid: Option<String>,
    // TODO: support union id in wechat.
    // unionid: Option<String>,
}


#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: Option<String>,
    expires_in: Option<i32>,
    errcode: Option<u16>,
    errmsg: Option<String>,
}


wx_function!(_get_session_key, SessionResponse, "https://api.weixin.qq.com/sns/jscode2session");
wx_function!(_get_access_token, AccessTokenResponse, "https://api.weixin.qq.com/cgi-bin/token");


#[derive(Debug, Fail)]
#[fail(display = "Wechat interface error {}: {}.", errcode, errmsg)]
pub struct WxErr {
    pub errcode: u16,
    pub errmsg: String,
}

pub struct WxSession {
    pub session_key: String,
    pub openid: String,
}


pub async fn get_session_by_code(wechat_code: &str) -> Result<WxSession, ServerError> {
    let resp: SessionResponse = _get_session_key(make_parameter!(
        "appid" => &CONFIG.wechat_appid,
        "secret" => &CONFIG.wechat_secret,
        "js_code" => wechat_code,
        "grant_type" => "authorization_code"
    ).as_str()).await?;

    // TODO:
    // 每个函数中的这段 match 代码可以放到 wx_function 宏里面去提前处理错误
    // 但是考虑到需要处理所有 Response 的字段，以后可以精简下这块代码
    match resp {
        SessionResponse {
            session_key: Some(session_key),
            openid: Some(openid),
            ..
        } => return Ok(WxSession { session_key, openid }),
        SessionResponse {
            errcode: Some(errcode),
            errmsg: Some(errmsg),
            ..
        } => return Err(ServerError::from(WxErr { errcode, errmsg })),
        _ => return Err(ServerError::from(WxErr { errcode: 0, errmsg: String::from("Unknown.") })),
    };
}


pub struct WxAccessToken {
    pub access_token: String,
    pub expires_in: i32,
}


pub async fn get_access_token() -> Result<WxAccessToken, ServerError> {
    let resp: AccessTokenResponse = _get_access_token(make_parameter!(
        "appid" => &CONFIG.wechat_appid,
        "secret" => &CONFIG.wechat_secret,
        "grant_type" => "client_credential"
    ).as_str()).await?;

    match resp {
        AccessTokenResponse {
            access_token: Some(access_token),
            expires_in: Some(expires_in),
            ..
        } => Ok(WxAccessToken { access_token, expires_in }),
        AccessTokenResponse {
            errcode: Some(errcode),
            errmsg: Some(errmsg),
            ..
        } => Err(ServerError::from(WxErr { errcode, errmsg })),
        _ => Err(ServerError::from(WxErr { errcode: 0, errmsg: String::from("Unknown.") })),
    }
}
