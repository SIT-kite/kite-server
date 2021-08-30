//! This module manages attachments, storage, upload and provides some interfaces for administrators.
//! At current time, file will be stored in local storage.

use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

pub use attachment::get_attachment_url_prefix;
pub use attachment::get_file_extension;

mod attachment;
mod avatar;

#[derive(Debug, thiserror::Error, Serialize, ToPrimitive)]
pub enum AttachmentError {
    #[error("文件名不正确")]
    FilenameRefused = 170,
    #[error("文件不存在")]
    NotFound = 171,
    #[error("文件写入失败")]
    FailedToWrite = 172,
    #[error("文件上传中断")]
    Interrupted = 173,
    #[error("没有发现要上传的文件")]
    NoPayload = 174,
    #[error("文件大小超过限制")]
    TooLarge = 175,
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
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    /// Attachment id.
    #[serde(skip_serializing)]
    pub id: Uuid,
    /// Name of file uploaded.
    pub name: String,
    /// UID of uploader.
    pub uploader: i32,
    /// Upload time.
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

pub struct AttachmentManager<'a> {
    pool: &'a PgPool,
}

pub type AvatarImage = Attachment;

pub struct AvatarManager<'a> {
    pool: &'a PgPool,
}
