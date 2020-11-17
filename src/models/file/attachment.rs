use super::{Attachment, AttachmentBasic, AttachmentError, AttachmentManager};
use crate::error::{ApiError, Result};
use crate::models::PageView;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

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

    pub fn set_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn set_uploader(mut self, uid: i32) -> Self {
        self.uploader = uid;
        self
    }

    pub fn set_file(mut self, prefix: &str, path: String, size: i32) -> Self {
        let ext = get_file_extension(&path);
        self.url = Some(format!("{}{}.{}", prefix, self.id.to_string(), ext));

        self.path = Some(path);
        self.size = size;

        self
    }
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
