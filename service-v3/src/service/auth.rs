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

pub fn get_token_from_request<T>(req: tonic::Request<T>) -> Result<JwtToken, tonic::Status> {
    if let Some(token) = req.metadata().get("authorization") {
        let token = token
            .to_str()
            .map_err(|e| tonic::Status::unauthenticated(format!("Failed to parse token to str: {:?}", e)))?;
        if let Some(token) = JwtToken::decode(token) {
            Ok(token)
        } else {
            Err(tonic::Status::unauthenticated("Invalid token: May be expired?"))
        }
    } else {
        Err(tonic::Status::unauthenticated(
            "No authorization can be found in your request.",
        ))
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
