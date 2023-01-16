use anyhow::Error;
use tonic::Status;

pub trait ToStatus {
    fn to_status(self) -> Status;
}

impl ToStatus for Error {
    fn to_status(self) -> Status {
        Status::internal(self.to_string())
    }
}
