pub use server::ServerError;
pub use sign::EventError;
pub use user::UserError;

mod server;
mod sign;
mod user;

pub type Result<T> = std::result::Result<T, ServerError>;
pub type Error = ServerError;
