use crate::error::{ApiError, Result};
use sqlx::PgPool;

#[derive(serde::Serialize, sqlx::FromRow, Debug)]
pub struct Telephone {
    /// Department
    pub department: Option<String>,
    /// Name of the number master
    pub name: Option<String>,
    /// Phone number
    pub phone: String,
    /// The action you can ask
    pub action: Option<String>,
}

pub async fn query_all_phone_number(db: &PgPool) -> Result<Vec<Telephone>> {
    let telephone =
        sqlx::query_as("SELECT department, name, phone, action FROM address_book.telephone;")
            .fetch_all(db)
            .await?;

    Ok(telephone)
}
