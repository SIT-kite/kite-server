use serde::Serialize;

#[derive(serde::Serialize)]
pub struct ApiResponse<T: Serialize> {
    code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(code: u16, msg: Option<String>, data: Option<T>) -> Self {
        Self { code, msg, data }
    }

    pub fn normal(data: T) -> Self {
        Self::new(0, None, Some(data))
    }

    pub fn empty() -> Self {
        Self::new(0, None, None)
    }

    pub fn fail(code: u16, msg: String) -> Self {
        Self::new(code, Some(msg), None)
    }
}

impl<T: Serialize> Into<serde_json::Value> for ApiResponse<T> {
    fn into(self) -> serde_json::Value {
        serde_json::to_value(&self).unwrap()
    }
}
