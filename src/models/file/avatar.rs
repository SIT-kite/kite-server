use super::AttachmentManager;
use super::{AvatarImage, AvatarManager};
use crate::config::CONFIG;
use crate::error::{ApiError, Result};
use crate::models::file::AttachmentError;
use actix_web::client as awc;
use sqlx::postgres::PgQueryAs;
use sqlx::PgPool;
use tokio::io::AsyncWriteExt;

/// Url prefix for avatars.
static URL_PREFIX: &'static str = "https://kite.sunnysab.cn/static/avatar/";

impl<'a> AvatarManager<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        AvatarManager { pool }
    }

    pub async fn query(&self, original_url: &str) -> Result<AvatarImage> {
        let avatar = sqlx::query_as(
            "SELECT id, name, path, uploader, is_deleted, size, upload_time, url
                FROM public.attachments WHERE name = $1 LIMIT 1",
        )
        .bind(original_url)
        .fetch_optional(self.pool)
        .await?;

        avatar.ok_or(ApiError::new(AttachmentError::NotFound))
    }

    pub async fn save(&self, uid: i32, original_url: &str) -> Result<AvatarImage> {
        let avatar = Self::fetch_download(uid, original_url).await?;
        let attachment_manager = AttachmentManager::new(self.pool);

        attachment_manager.create(&avatar).await?;
        Ok(avatar)
    }

    async fn download(avatar_url: &str, path: &str) -> Result<usize> {
        let client = awc::Client::new();
        if let Ok(mut image) = client.get(avatar_url).send().await {
            if let Ok(content) = image.body().await {
                let size = content.len();
                let mut file = tokio::fs::File::create(path).await?;
                file.write_all(&content).await?;

                return Ok(size);
            }
        }
        Err(ApiError::new(AttachmentError::Interrupted))
    }

    async fn fetch_download(uid: i32, avatar_url: &str) -> Result<AvatarImage> {
        let uuid = uuid::Uuid::new_v4();
        let avatar = AvatarImage::with_id(uuid).set_name(avatar_url).set_uploader(uid);

        let path = format!("{}/avatar/{}.jpg", CONFIG.server.attachment, uuid);
        let size = Self::download(avatar_url, &path).await?;

        Ok(avatar.set_file(URL_PREFIX, path, size as i32))
    }
}
