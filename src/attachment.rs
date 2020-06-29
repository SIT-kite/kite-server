//! This module manages attachments, storage, upload and provides some interfaces for administrators.
//! At current time, file will be stored in local storage.

use chrono::{NaiveDateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, postgres::PgQueryAs};
use std::fs;
use uuid::Uuid;

use crate::error::Result;

/// Url prefix for attachment.
static URL_PREFIX: &'static str = "https://kite.sunnysab.cn/api/v1/attachment/";

/// Get url prefix for attachment.
pub fn get_attachment_url_prefix() -> &'static str {
    URL_PREFIX
}

#[derive(Debug, Fail, Serialize, ToPrimitive)]
pub enum AttachmentError {
    #[fail(display = "需要登录才能继续")]
    Forbidden = 24,
    #[fail(display = "文件名不正确")]
    FilenameRefused = 25,
    #[fail(display = "文件写入失败")]
    FileCreationFailed = 26,
}

/// Determine whether save directory is available or not.
pub fn check_directory(path: &String) -> bool {
    // Returns true if the path exists on disk and is pointing at a directory.
    std::path::Path::new(path).is_dir()
}

/// Create save directory, may fail if permission denied.
pub fn create_directory(path: &String) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

/// Attachment struct for the public.
#[derive(Serialize, sqlx::FromRow)]
pub struct SingleAttachment {
    /// Attachment id.
    pub id: Uuid,
    /// Name of file uploaded.
    pub name: String,
    /// UID of uploader.
    pub uploader: i32,
    /// Upload time.
    pub upload_time: NaiveDateTime,
    /// Upload size.
    pub size: i32,
    /// Storage path.
    #[serde(rename(serialize = "storagePath"))]
    pub storage_path: Option<String>,
    /// Link for downloading.
    pub url: Option<String>,
}


impl Default for SingleAttachment {
    fn default() -> Self {
        SingleAttachment {
            id: Uuid::new_v4(),
            name: "".to_string(),
            uploader: 0,
            upload_time: Utc::now().naive_local(),
            size: 0,
            storage_path: None,
            url: None
        }
    }
}

/// Get attachment list for administrators.
pub async fn get_all_attachment_list(client: &PgPool, page_index: i64, page_size: i64)
    -> Result<Vec<SingleAttachment>> {

    let page_index: i64 = if page_index < 0 { 0 } else { page_index };
    let page_size: i64 = if page_size < 1 { 10 } else { page_size };
    let attachments: Vec<SingleAttachment> = sqlx::query_as(
        "SELECT id, name, uploader, size, CONCAT($1, $2
         FROM public.attachments OFFSET $1 LIMIT $2",
    )
    .bind((page_index - 1) * page_size)
    .bind(page_size)
    .fetch_all(client)
    .await?;
    Ok(attachments)
}

pub fn get_file_extension(filename: &str) -> String {
    // Consider filename with no extension
    // "text.", "text.txt", "text".
    let last_terminator = filename.rfind(".");
    if last_terminator.is_none() || last_terminator.unwrap() == filename.len() - 1 {
        return "".to_string();
    }
    filename[last_terminator.unwrap()..].to_string()
}

pub fn check_file_extension(filename: &str) -> bool {
    false
}

/// Insert attachment record to database.
pub async fn create_attachment(
    client: &PgPool,
    filename: &String,
    storage_path: &String,
    uploader: i32,
    size: i32,
) -> Result<SingleAttachment> {
    let attachment_id = Uuid::new_v4();
    let new_attachment = SingleAttachment {
        id: attachment_id,
        name: filename.clone(),
        uploader,
        storage_path: Some(storage_path.clone()),
        upload_time: Utc::now().naive_local(),
        size,
        url: Some(format!(
            "{}/{}",
            get_attachment_url_prefix(),
            attachment_id.to_string()
        )),
    };
    let _ = sqlx::query(
        "INSRET INTO public.attachments (id, name, storage_path, uploader, upload_time, size, url)\
            RETURNING id",
    )
    .bind(attachment_id)
    .bind(&new_attachment.name)
    .bind(&storage_path)
    .bind(&new_attachment.uploader)
    .bind(&new_attachment.upload_time)
    .bind(&new_attachment.size)
    .bind(&new_attachment.url)
    .execute(client)
    .await?;
    Ok(new_attachment)
}

pub async fn get_attachment_by_uid(client: &PgPool, uid: u32) -> Result<Vec<SingleAttachment>> {
    Ok(Vec::new())
}

#[cfg(test)]
mod test {
    #[test]
    pub fn get_file_extension() {
        assert_eq!(super::get_file_extension("a.jpg"), ".jpg");
        assert_eq!(super::get_file_extension("a."), "");
        assert_eq!(super::get_file_extension("a"), "");
    }
}