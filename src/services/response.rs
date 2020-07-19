use serde::Serialize;

/// Common response type for Kite http server.
/// Generating the response to the caller.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T = ()> {
    code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

impl ApiResponse<()> {
    /// Response a success code but with no payload.
    pub fn empty() -> Self {
        ApiResponse { code: 0, data: None }
    }
}

impl<T> ApiResponse<T> {
    /// Response with regular data.
    pub fn normal(data: T) -> ApiResponse<T>
    where
        T: Serialize,
    {
        ApiResponse {
            code: 0,
            data: Some(data),
        }
    }
}

impl<T> ToString for ApiResponse<T>
where
    T: Serialize,
{
    // Serialize
    fn to_string(&self) -> String {
        if let Ok(body_json) = serde_json::to_string(&self) {
            return body_json;
        }
        String::from("Critical: Could not serialize error message.")
    }
}
