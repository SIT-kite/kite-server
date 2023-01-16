use anyhow::anyhow;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

use crate::config;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct JwtToken {
    /// 用户 ID
    pub uid: i32,
    /// 用户角色
    pub role: i32,
}

impl JwtToken {
    pub fn new(uid: i32, role: i32) -> Self {
        Self { uid, role }
    }
    pub fn encode(&self) -> String {
        let key = config::get().secret.as_str();
        let encoding_key = EncodingKey::from_secret(key.as_ref());

        encode(&Header::default(), &self, &encoding_key).unwrap()
    }

    pub fn decode(token: &str) -> Option<Self> {
        let key = config::get().secret.as_str();
        let decoding_key = DecodingKey::from_secret(key.as_ref());
        let option = Validation::default();
        let token_data = decode::<Self>(token, &decoding_key, &option);

        if let Ok(token_data) = token_data {
            Some(token_data.claims)
        } else {
            None
        }
    }
}

pub fn get_token_from_request<T>(req: tonic::Request<T>) -> anyhow::Result<JwtToken> {
    if let Some(token) = req.metadata().get("authorization") {
        let token = token
            .to_str()
            .map_err(|e| anyhow!("Failed to parse token to str: {:?}", e))?;
        if let Some(token) = JwtToken::decode(token) {
            Ok(token)
        } else {
            Err(anyhow!("Invalid token: May be expired?"))
        }
    } else {
        Err(anyhow!("No authorization can be found in your request."))
    }
}

pub fn require_login<T>(req: tonic::Request<T>) -> Result<(), tonic::Status> {
    if let Err(e) = get_token_from_request(req) {
        Err(tonic::Status::unauthenticated(e.to_string()))
    } else {
        Ok(())
    }
}

pub fn require_user<T>(req: tonic::Request<T>, uid: i32) -> Result<(), tonic::Status> {
    match get_token_from_request(req) {
        Ok(token) => {
            if token.uid == uid {
                Ok(())
            } else {
                Err(tonic::Status::permission_denied(
                    "You are manipulating other's resource.",
                ))
            }
        }
        Err(e) => Err(tonic::Status::unauthenticated(e.to_string())),
    }
}
