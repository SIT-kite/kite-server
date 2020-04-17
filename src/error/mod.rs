pub use server::ServerError;
pub use user::UserError;

mod user;
mod server;

pub type Result<T> = std::result::Result<T, ServerError>;
pub type Error = ServerError;