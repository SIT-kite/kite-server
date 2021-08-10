use sqlx::PgPool;

use crate::error::{ApiError, Result};

#[derive(serde::Serialize, sqlx::FromRow, Debug)]
pub struct Contact {
    /// Department
    pub department: Option<String>,
    /// Name of the number master
    pub name: Option<String>,
    /// Phone number
    pub phone: String,
    /// The action you can ask
    pub action: Option<String>,
}

pub async fn get_all_contacts(db: &PgPool) -> Result<Vec<Contact>> {
    let telephone =
        sqlx::query_as("SELECT department, name, phone, action FROM address_book.telephone;")
            .fetch_all(db)
            .await?;

    Ok(telephone)
}
