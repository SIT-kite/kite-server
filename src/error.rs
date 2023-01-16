use tonic::Status;

pub trait ToStatus {
    fn to_status(self) -> Status;
}

impl ToStatus for anyhow::Error {
    fn to_status(self) -> Status {
        Status::internal(self.to_string())
    }
}

impl ToStatus for sqlx::Error {
    fn to_status(self) -> Status {
        // TODO: Add log here.
        Status::internal("Error occurred in database")
    }
}
