use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpResponse};
use chrono::Utc;
use futures::{StreamExt, TryStreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;

use crate::config::CONFIG;
use crate::error::{Result, ServerError};
use crate::models::attachment::{self, AttachmentError, SingleAttachment};
use crate::services::{JwtToken, NormalResponse};
use sqlx::PgPool;
use uuid::Uuid;

const MAX_ATTACHMENT_SIZE: usize = 10 * 1024 * 1024;

/// Upload attachment handler.
/// Attachments may be stored on ECS or local storage in the services, and now is local storage.
/// Adapted from https://github.com/actix/examples/.
/// Note: client can upload multiple files, so we use payload.try_next() to iterate all the files.
/// There is also a while loop and iteration for streams in each file.
#[post("/attachment")]
pub(crate) async fn upload_file(
    pool: web::Data<PgPool>,
    token: Option<JwtToken>,
    mut payload: Multipart,
) -> Result<HttpResponse> {
    if token.is_none() {
        return Err(ServerError::new(AttachmentError::Forbidden));
    }
    // Vector of uploaded file.
    let mut successd_uploaded: Vec<SingleAttachment> = Vec::new();
    let token = token.unwrap();
    let uid = token.uid;

    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        // Get filename and sanitize it.
        // See also:
        // https://docs.rs/actix-http/1.0.1/actix_http/http/header/struct.ContentDisposition.html
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();
        let timestamp = format!("{}", Utc::now().timestamp());
        let sanitized_filename = format!(
            "{}-{}",
            timestamp,
            sanitize_filename::sanitize(filename.to_string())
        );

        let mut success_flag: bool = true;
        let mut file_size = 0;
        let filepath = format!("{}{}", CONFIG.attachment_dir, &sanitized_filename);

        let file = tokio::fs::File::create(&filepath)
            .await
            .map_err(|_| ServerError::new(AttachmentError::FileCreationFailed))?;
        let mut writer = tokio::io::BufWriter::new(file);

        // What about user canceling uploading?
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            file_size += data.len();

            if file_size > MAX_ATTACHMENT_SIZE {
                success_flag = false;
                break;
            }
            if let Err(_) = writer.write(&data).await {
                success_flag = false;
                break;
            }
        }
        if success_flag {
            let new_attachment =
                attachment::create_attachment(&pool, &sanitized_filename, &filepath, uid, file_size)
                    .await?;
            successd_uploaded.push(new_attachment);
        }
    }

    let mut resp = HashMap::new();
    resp.insert("uploaded", successd_uploaded);
    Ok(HttpResponse::Ok().json(NormalResponse::new(resp)))
}

#[derive(Deserialize)]
pub struct PageOption {
    pub index: Option<u32>,
    pub count: Option<u32>,
}

#[get("/attachment")]
pub async fn get_attachment_list(
    pool: web::Data<PgPool>,
    token: Option<JwtToken>,
    form: web::Form<PageOption>,
) -> Result<HttpResponse> {
    if token.is_none() {
        return Err(ServerError::new(AttachmentError::Forbidden));
    }
    if !token.unwrap().is_admin {
        return Err(ServerError::new(AttachmentError::Forbidden));
    }

    let attachs =
        attachment::get_attachment_by_page(&pool, form.index.unwrap_or(1), form.count.unwrap_or(20))
            .await?;

    let mut resp = HashMap::new();
    resp.insert("attachments", attachs);
    Ok(HttpResponse::Ok().json(&NormalResponse::new(resp)))
}

#[get("/attachment/{attachment_id}")]
pub async fn index(pool: web::Data<PgPool>, id: web::Path<(Uuid,)>) -> Result<HttpResponse> {
    let url = attachment::get_attachment_url_by_id(&pool, id.into_inner().0).await?;
    if let None = url {
        return Ok(HttpResponse::NotFound().finish());
    }
    Ok(HttpResponse::PermanentRedirect()
        .set_header("Location", url.unwrap())
        .finish())
}
