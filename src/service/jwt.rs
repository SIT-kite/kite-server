use crate::config::CONFIG;

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
        let key = CONFIG.get().unwrap().secret.as_str();
        let encoding_key = jsonwebtoken::EncodingKey::from_secret(key.as_ref());

        jsonwebtoken::encode(&jsonwebtoken::Header::default(), &self, &encoding_key).unwrap()
    }

    pub fn decode(token: &str) -> Option<Self> {
        let key = CONFIG.get().unwrap().secret.as_str();
        let decoding_key = jsonwebtoken::DecodingKey::from_secret(key.as_ref());
        let option = jsonwebtoken::Validation::default();
        let token_data = jsonwebtoken::decode::<Self>(token, &decoding_key, &option);

        if let Ok(token_data) = token_data {
            Some(token_data.claims)
        } else {
            None
        }
    }

    pub fn validate(token: &str) -> bool {
        Self::decode(token).is_some()
    }
}
