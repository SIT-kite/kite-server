use chrono::{DateTime, Local};
use sqlx::PgPool;
use tokio::io::AsyncWriteExt;

use crate::error::{Result, ApiError};
use super::PageView;


const THUMB_MAX_SIZE: u32 = 600;
const IMAGE_FOLDER: &str = "/var/kite/static/board";
const THUMB_FOLDER: &str = "/var/kite/static/board/thumb";

const IMAGE_URL_PREFIX: &str = "https://kite.sunnysab.cn/static/board";
const THUMB_URL_PREFIX: &str = "https://kite.sunnysab.cn/static/board/thumb";

#[derive(thiserror::Error, Debug, ToPrimitive)]
pub enum BoardError {
    #[error("无法解析图片")]
    BadImage = 200,
    #[error("保存图片失败")]
    FailToSave = 201,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct Picture {
    /// Picture uuid
    pub id: uuid::Uuid,
    /// Updater
    pub uid: i32,
    /// Web path to origin image
    pub path: String,
    /// Web path to thumbnail image
    pub thumbnail: String,
    /// Upload time
    pub ts: DateTime<Local>,
    /// Deletion flag
    pub deleted: bool,
    /// Extension
    pub ext: String,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct PictureSummary {
    /// Picture uuid
    pub id: uuid::Uuid,
    /// Publisher
    pub publisher: String,
    /// Thumbnail image url
    pub thumbnail: String,
    /// Publish time
    pub ts: DateTime<Local>,
}

impl Picture {
    pub fn new(uid: i32, ext: &str) -> Self {
        let id = uuid::Uuid::new_v4();

        Self { 
            id,
            uid, 
            path: format!("{}/{}.{}", IMAGE_URL_PREFIX, id, ext),
            thumbnail: format!("{}/{}.webp", THUMB_URL_PREFIX, id),
            ts: Local::now(),
            deleted: false,
            ext: ext.to_string(),
        }
    }
}

fn make_thumbnail(content: &[u8], longest_edge: u32) -> Result<Vec<u8>> {
    let img = image::load_from_memory(content).map_err(|_| ApiError::new(BoardError::BadImage))?;
    let img = img.thumbnail(longest_edge, longest_edge);

    let mut buf = std::io::Cursor::new(Vec::<u8>::new());
    img.write_to(&mut buf, image::ImageFormat::WebP);

    Ok(buf.get_ref().to_vec())
}

pub async fn save(pic: &Picture, content: &[u8]) -> Result<()> {
    let path = format!("{}/{}.{}", IMAGE_FOLDER, &pic.id, &pic.ext);
    let mut file = tokio::fs::File::create(&path).await.map_err(|_| ApiError::new(BoardError::FailToSave))?;
    file.write_all(content).await;

    let path = format!("{}/{}.webp", THUMB_URL_PREFIX, &pic.id);
    let thumb_image = make_thumbnail(content, THUMB_MAX_SIZE)?;
    let mut file = tokio::fs::File::create(&path).await.map_err(|_| ApiError::new(BoardError::FailToSave))?;
    file.write_all(&thumb_image).await;

    Ok(())
}

pub async fn insert_db(pool: &PgPool, pic: &Picture) -> Result<()> {
    sqlx::query("INSERT INTO board.picture (id, uid, path, thumbnail, ts, deleted, ext)
        VALUES ($1, $2, $3, $4, $5, $6, $7);")
        .bind(&pic.id)
        .bind(pic.uid)
        .bind(&pic.path)
        .bind(&pic.thumbnail)
        .bind(&pic.ts)
        .bind(pic.deleted)
        .bind(&pic.ext)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_picture_list(pool: &PgPool, page: &PageView) -> Result<Vec<PictureSummary>> {
    let result: Vec<PictureSummary> = sqlx::query_as(
        "SELECT id, '', thumbnail, ts
        ORDER BY ts DESC
        LIMIT $1 OFFSET $2;")
        .bind(page.count(20))
        .bind(page.offset(20))
        .fetch_all(pool)
        .await?;

    Ok(result)

}