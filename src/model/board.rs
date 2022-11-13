use chrono::{DateTime, Local};
use image::EncodableLayout;
use sqlx::PgPool;
use tokio::io::AsyncWriteExt;
use poem::web::{Path, Query};

use uuid::{Error, Uuid};
use webp::{Encoder, WebPMemory};

use super::PageView;
use crate::error::{ApiError, Result};

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
    /// Origin url
    pub origin: String,
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
    let is_horizontal: bool = img.width() > img.height();
    let mut img = img.thumbnail(longest_edge, longest_edge);
    if is_horizontal != (img.width() > img.height()) {
        img = img.rotate90();
    }

    let encoder: Encoder = Encoder::from_image(&img).unwrap();
    let encoded_webp: WebPMemory = encoder.encode(70f32);

    Ok(encoded_webp.as_bytes().to_vec())
}

pub async fn save(pic: &Picture, content: &[u8]) -> Result<()> {
    let path = format!("{}/{}.{}", IMAGE_FOLDER, &pic.id, &pic.ext);
    let mut file = tokio::fs::File::create(&path).await.map_err(|e| {
        println!("{}", e);
        ApiError::new(BoardError::FailToSave)
    })?;
    file.write_all(content).await;

    let path = format!("{}/{}.webp", THUMB_FOLDER, &pic.id);
    let thumb_image = make_thumbnail(content, THUMB_MAX_SIZE)?;
    let mut file = tokio::fs::File::create(&path).await.map_err(|e| {
        println!("{}", e);
        ApiError::new(BoardError::FailToSave)
    })?;
    file.write_all(&thumb_image).await;

    Ok(())
}

pub async fn insert_db(pool: &PgPool, pic: &Picture) -> Result<()> {
    sqlx::query(
        "INSERT INTO board.picture (id, uid, path, thumbnail, ts, deleted, ext)
        VALUES ($1, $2, $3, $4, $5, $6, $7);",
    )
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
        "SELECT id, substring(name from 1 for 1) || '同学' AS publisher, thumbnail, path AS origin, ts FROM board.picture_view
        WHERE deleted = FALSE
        ORDER BY ts DESC
        LIMIT $1 OFFSET $2;",
    )
        .bind(page.count(20))
        .bind(page.offset(20))
        .fetch_all(pool)
        .await?;

    Ok(result)
}

pub async fn post_like(pool: &PgPool, id: String, uid: i32, like_type: i32) -> Result<()> {
    let mid = Uuid::parse_str(&id);
    let mid2 = match mid {
        Ok(x) => x,
        Err(..) => uuid::uuid!("e65162d0-9127-4efe-8f84-aef706ffffff"),
    };


    if &like_type == &-1 || &mid2 == &uuid::uuid!("e65162d0-9127-4efe-8f84-aef706ffffff") {
        return Err(ApiError { code: 400, msg: Option::from("参数有错".to_string()) });
    }

    if &like_type == &0 {
        sqlx::query(
            "DELETE FROM board.like_record WHERE \"id\"=$1 AND uid=$2"
        )
            .bind(&mid2)
            .bind(&uid)
            .execute(pool)
            .await?;
    }

    if like_type > 0 && like_type < 6 {
        sqlx::query(
            "INSERT INTO board.like_record (\"id\",uid,like_type )VALUES ($1,$2,$3 ) \
        ON  CONFLICT(\"id\",uid) DO UPDATE SET like_type=$3;"
        )
            .bind(&mid2)
            .bind(&uid)
            .bind(&like_type)
            .execute(pool)
            .await?;
    }


    // println!("{}", &id);
    Ok(())
}


pub async fn get_my_picture_list(pool: &PgPool, uid: i32, page: &PageView) -> Result<Vec<PictureSummary>> {
    let result: Vec<PictureSummary> = sqlx::query_as(
        "SELECT id, substring(name from 1 for 1) || '同学' AS publisher, thumbnail, path AS origin, ts FROM board.picture_view
        WHERE deleted = FALSE AND uid=$3
        ORDER BY ts DESC
        LIMIT $1 OFFSET $2 ;",
    )
        .bind(page.count(20))
        .bind(page.offset(20))
        .bind(&uid)
        .fetch_all(pool)
        .await?;

    Ok(result)
}


pub async fn post_delete(pool: &PgPool, id: String) -> Result<()> {
    let mid = Uuid::parse_str(&id);

    sqlx::query(
        "UPDATE board.picture SET deleted='t' WHERE \"id\"=$1"
    )
        .bind(&mid)
        .execute(pool)
        .await?;
    Ok(())
}
