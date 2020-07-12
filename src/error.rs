pub use server::ServerError;
pub use sign::EventError;

mod server;
mod sign;

pub type Result<T> = std::result::Result<T, ServerError>;
pub type Error = ServerError;
