use serde::Serialize;

// Anti-spam module.
pub mod antispam;
// Structs reflect database table.
pub mod models;
// Interfaces.
pub mod actions;
// Wechat ability
pub mod wechat;


#[derive(Debug, Serialize)]
pub struct NormalResponse {
    code: u16,
    pub data: String,
}

impl NormalResponse {
    pub fn new(data: String) -> NormalResponse {
        NormalResponse {
            code: 0,
            data,
        }
    }
}

impl ToString for NormalResponse {
    fn to_string(&self) -> String {
        if let Ok(body_json) = serde_json::to_string(&self) {
            return body_json;
        }
        r"{code: 1}".to_string()
    }
}