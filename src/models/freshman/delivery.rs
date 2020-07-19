use crate::error::Result;
use chrono::{NaiveDateTime, Utc};
use serde::Serialize;
use sqlx::postgres::{PgPool, PgQueryAs};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Package {
    /// Package id
    #[serde(rename = "packageId")]
    pub package_id: i32,
    /// Submit user
    pub uid: i32,
    /// Express tracking number
    #[serde(rename = "trackingNumber")]
    pub tracking_number: String,
    /// Express company name
    pub express: String,
    /// Package image
    pub image: Vec<Uuid>,
    /// Publish time
    #[serde(rename = "publishTime")]
    pub create_time: NaiveDateTime,
    /// Last tracking update time
    #[serde(rename = "lastUpdate")]
    pub last_update: NaiveDateTime,
    /// Finish status
    pub finish: bool,
    /// Details from express company
    pub details: serde_json::Value,
}

impl Default for Package {
    fn default() -> Self {
        Package {
            package_id: 0,
            uid: 0,
            tracking_number: "".to_string(),
            express: "".to_string(),
            image: vec![],
            create_time: Utc::now().naive_local(),
            last_update: Utc::now().naive_local(),
            finish: false,
            details: serde_json::Value::default(),
        }
    }
}

impl Package {
    /// Create new delivery structure.
    pub fn new() -> Self {
        Package::default()
    }

    /// Write to database.
    pub async fn create(&self, client: &PgPool) -> Result<i32> {
        let package_id: (i32,) = sqlx::query_as(
            "INSERT INTO freshman.delivery
            (uid, tracking_number, express, image, details) VALUES ($1, $2, $3, $4, $5)
            RETURNING package_id",
        )
        .bind(self.uid)
        .bind(&self.tracking_number)
        .bind(&self.express)
        .bind(&self.image)
        .bind(&self.details)
        .fetch_one(client)
        .await?;
        Ok(package_id.0)
    }

    /// List packages.
    pub async fn list_by_uid(client: &PgPool, uid: i32) -> Result<Vec<Self>> {
        let packages: Vec<Self> = sqlx::query_as(
            "SELECT uid, tracking_number, express, image, create_time, last_update, finish
            FROM freshman.delivery WHERE uid = $1",
        )
        .bind(uid)
        .fetch_all(client)
        .await?;
        Ok(packages)
    }

    /// List all packages
    pub async fn list_all(_client: &PgPool) -> Result<Vec<Self>> {
        Ok(vec![])
    }
}
