use std::io::Write;

use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use chrono::{NaiveDateTime, Utc};
use futures::{StreamExt, TryStreamExt};
use num_derive::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::attachment::{self, AttachmentError, SingleAttachment};
use crate::config::CONFIG;
use crate::error::{Result, ServerError};
use crate::server::{JwtToken, NormalResponse};
use sqlx::PgPool;


/// Upload attachment handler.
/// Attachments may be stored on ECS or local storage in the server, and now is local storage.
/// Adapted from https://github.com/actix/examples/.
/// Note: client can upload multiple files, so we use payload.try_next() to iterate all the files.
/// There is also a while loop and iteration for streams in each file.
#[post("/attachment")]
async fn upload_file(
    pool: web::Data<PgPool>,
    token: Option<JwtToken>,
    mut payload: Multipart,
) -> Result<HttpResponse> {

    if token.is_none() {
        return Err(ServerError::new(AttachmentError::Forbidden));
    }
    /// uploaded file.
    let mut successd_uploaded: Vec<SingleAttachment> = Vec::new();
    let token = token.unwrap();
    let uid = token.uid;

    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let mut new_attachment = SingleAttachment::default();
        // Get filename and sanitize it.
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();
        let sanitized_filename = sanitize_filename::sanitize(filename.to_string());

        let mut success_flag: bool = true;
        let mut file_size = 0;
        let filepath = format!("{}{}", CONFIG.attachment_dir, sanitized_filename); // Path, file actually stored.
        let filepath2 = filepath.clone(); // TODO: Improve.
        // File::create is blocking operation, use thread pool
        let mut f = web::block(move || std::fs::File::create(filepath.clone()))
            .await.map_err(|_| ServerError::new(AttachmentError::FileCreationFailed))?;

        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            file_size += data.len();
            match web::block(move || f.write_all(&data).map(|_| f)).await {
                Ok(handle) => f = handle,
                Err(_) => {
                    success_flag = false;
                    break;
                }
            }
        }

        attachment::create_attachment(&pool, &new_attachment.name, &filepath2,
                                      uid, new_attachment.size).await?;
        successd_uploaded.push(new_attachment);
    }

    use std::collections::HashMap;
    let mut resp = HashMap::new();
    resp.insert("uploaded", successd_uploaded);
    Ok(HttpResponse::Ok().json(NormalResponse::new(resp)))
}
