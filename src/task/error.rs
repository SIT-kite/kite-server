use super::HostError;
use crate::convert_inner_errors;
use crate::error::ApiError;

pub type Result<T> = std::result::Result<T, ApiError>;

use bincode::Error as BincodeError;
use tokio::sync::oneshot::error::RecvError;
use tokio_tungstenite::tungstenite::error::Error as WsError;

convert_inner_errors!(HostError);
convert_inner_errors!(RecvError);
convert_inner_errors!(WsError);
convert_inner_errors!(BincodeError);
// convert_inner_errors!(SendError<T>);
