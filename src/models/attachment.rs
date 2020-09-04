//! This module manages attachments, storage, upload and provides some interfaces for administrators.
//! At current time, file will be stored in local storage.

use chrono::{NaiveDateTime, Utc};
use serde::Serialize;
use sqlx::{postgres::PgQueryAs, PgPool};
use uuid::Uuid;

use super::PageView;
use crate::error::{ApiError, Result};

/// Url prefix for attachment.
static URL_PREFIX: &'static str = "https://kite.sunnysab.cn/static/upload/";

/// Allowed file extension
static ALLOWED_EXT: &[&str] = &[
    "jpg", "png", "svg", "gif", "jpeg", // Images
    "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", // Documents
];

/// Get url prefix for attachment.
pub fn get_attachment_url_prefix() -> &'static str {
    URL_PREFIX
}

#[derive(Debug, thiserror::Error, Serialize, ToPrimitive)]
pub enum AttachmentError {
    #[error("文件名不正确")]
    FilenameRefused = 170,
    #[error("文件不存在")]
    NotFound = 171,
    #[error("文件写入失败")]
    FailedToCreate = 172,
    #[error("无法获取上传的文件")]
    NoData = 173,
    #[error("文件上传中断")]
    Interrupted,
}

/// Attachment struct for the public.
#[derive(Serialize, sqlx::FromRow)]
pub struct AttachmentBasic {
    /// Name of the file
    pub name: String,
    /// Upload size.
    pub size: i32,
    /// Link for downloading.
    pub url: Option<String>,
}

/// Attachment struct for the administrator.
#[derive(Serialize, sqlx::FromRow)]
pub struct Attachment {
    /// Attachment id.
    #[serde(skip_serializing)]
    pub id: Uuid,
    /// Name of file uploaded.
    pub name: String,
    /// UID of uploader.
    pub uploader: i32,
    /// Upload time.
    #[serde(rename(serialize = "uploadTime"))]
    pub upload_time: NaiveDateTime,
    /// Storage path
    #[serde(skip_serializing)]
    pub path: Option<String>,
    /// Upload size.
    pub size: i32,
    /// Deleted
    pub is_deleted: bool,
    /// Link for downloading.
    pub url: Option<String>,
}

impl Attachment {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_id(id: Uuid) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn set_uploader(mut self, uid: i32) -> Self {
        self.uploader = uid;
        self
    }

    pub fn set_file(mut self, path: String, size: i32) -> Self {
        let ext = get_file_extension(&path);
        self.url = Some(format!("{}{}.{}", URL_PREFIX, self.id.to_string(), ext));

        self.path = Some(path);
        self.size = size;

        self
    }
}

pub struct AttachmentManager<'a> {
    pool: &'a PgPool,
}

/// Get file extension from file name.
pub fn get_file_extension(filename: &str) -> String {
    // Consider filename with no extension
    // "text.", "text.txt", "text".
    let last_terminator = filename.rfind(".").unwrap_or_default();
    if last_terminator == 0usize || last_terminator == filename.len() - 1 {
        return "".to_string();
    }
    filename[(last_terminator + 1)..].to_string()
}

/// Add file extension check to avoid attacking.
pub fn check_file_extension(filename: &str) -> bool {
    let extension = get_file_extension(filename);

    ALLOWED_EXT.contains(&extension.as_str())
}

impl<'a> AttachmentManager<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Get attachment list for administrators.
    pub async fn list(&self, page: PageView) -> Result<Vec<Attachment>> {
        let attachments: Vec<Attachment> = sqlx::query_as(
            "SELECT id, name, path, uploader, is_deleted, size, upload_time, url
                FROM public.attachments OFFSET $1 LIMIT $2",
        )
        .bind(page.offset(20) as i64)
        .bind(page.count(20) as i64)
        .fetch_all(self.pool)
        .await?;
        Ok(attachments)
    }

    /// Insert attachment record to database.
    pub async fn create(&self, attachment: &Attachment) -> Result<()> {
        let _ = sqlx::query(
            "INSERT INTO public.attachments (id, name, path, uploader, upload_time, size, url)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id",
        )
        .bind(attachment.id)
        .bind(&attachment.name)
        .bind(&attachment.path)
        .bind(&attachment.uploader)
        .bind(&attachment.upload_time)
        .bind(&attachment.size)
        .bind(&attachment.url)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, attachment_id: Uuid) -> Result<()> {
        let _ = sqlx::query("UPDATE public.attachment SET is_deleted = true WHERE attachment_id = $1")
            .bind(attachment_id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn query(&self, id: Uuid) -> Result<Attachment> {
        let basic_info: Option<Attachment> = sqlx::query_as(
            "SELECT id, name, path, uploader, upload_time, is_deleted, size, url 
                FROM public.attachments WHERE id = $1 LIMIT 1",
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        basic_info.ok_or(ApiError::new(AttachmentError::NotFound))
    }
}

// --------------- Some normal impls. ---------------

impl Default for Attachment {
    fn default() -> Self {
        Attachment {
            id: Uuid::new_v4(),
            name: "".to_string(),
            uploader: 0,
            upload_time: Utc::now().naive_local(),
            path: None,
            size: 0,
            is_deleted: false,
            url: None,
        }
    }
}

impl From<Attachment> for AttachmentBasic {
    fn from(attachment: Attachment) -> Self {
        Self {
            name: attachment.name,
            size: attachment.size,
            url: attachment.url,
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    pub fn get_file_extension() {
        assert_eq!(super::get_file_extension("a.jpg"), "jpg");
        assert_eq!(super::get_file_extension("a."), "");
        assert_eq!(super::get_file_extension("a"), "");
    }
}
