use std::collections::HashMap;
use std::sync::Arc;

use actix_http::HttpMessage;
use actix_web::client::{Client, Connector};
use actix_web::web::Bytes;
use serde::Deserialize;
use serde_json;

use crate::config::CONFIG;
use crate::user::error::UserError;

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


pub async fn get_session_by_code(wechat_code: &str) -> Result<SessionResponse, UserError> {
    //! Document:
    //! https://developers.weixin.qq.com/miniprogram/dev/api-backend/open-api/login/auth.code2Session.html
    //! Example:
    //! let session = get_session_by_code("021mYry413y4EP1etIx41ITpy41mYryd").await;
    //!     if let Ok(resp) = session {
    //!         //! No error showed.
    //!         if resp.errcode == None {
    //!         println!("SessionKey: {:?}, Openid: {:?}", resp.session_key, resp.openid);
    //!     }
    //!     else {
    //!         println ! ("Errcode: {:?}, Errmsg: {:?}", resp.errcode, resp.errmsg);
    //!     }
    //! }
    let mut client = Client::default();
    let url = format!("https://api.weixin.qq.com/sns/jscode2session?\
                                appid={}&secret={}&js_code={}&grant_type=authorization_code",
                      &CONFIG.wechat_appid, &CONFIG.wechat_secret, wechat_code);
    let response = client.get(url).send().await;
    match response {
        // Note: Sending successfully, not receiving.
        Ok(mut r) => {
            // Wechat server always return HTTP 200, with errcode field when parameter error.
            if r.status().is_success() {
                // Content-type is always "text/plain", so we cannot use following line to parse.
                // let response_json = r.json::<SessionResponse>().await;
                let body_string = r.body().await.unwrap_or(Bytes::from(""));
                // todo: 完善错误处理，多 impl
                let body_json: SessionResponse = serde_json::from_slice(body_string.as_ref()).unwrap();

                return Ok(body_json);
            }
            Err(UserError::ParsingError)
        }
        _ => {
            Err(UserError::NetworkError)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AccessTokenResponse {
    access_token: Option<String>,
    expires_in: Option<i32>,
    errcode: Option<i32>,
    errmsg: Option<String>,
}

pub async fn get_access_token() -> Result<AccessTokenResponse, UserError> {
    //! Document
    //!  https://developers.weixin.qq.com/miniprogram/dev/api-backend/open-api/access-token/auth.getAccessToken.html
    //!
    let mut client = Client::default();
    let url = format!("https://api.weixin.qq.com/cgi-bin/token?\
                                appid={}&secret={}&grant_type=client_credential",
                      &CONFIG.wechat_appid, &CONFIG.wechat_secret);
    let response = client.get(url).send().await;
    match response {
        Ok(mut r) => {
            if r.status().is_success() {
                let body_string = r.body().await.unwrap_or(Bytes::from("{}"));
                let body_json: AccessTokenResponse = serde_json::from_slice(body_string.as_ref()).unwrap();
                return Ok(body_json);
            }
            Err(UserError::ParsingError)
        }
        _ => {
            Err(UserError::NetworkError)
        }
    }
}

