use crate::config::CONFIG;
use crate::error::{ApiError, Result};
use crate::models::attachment::get_file_extension;
use crate::models::attachment::{Attachment, AttachmentBasic, AttachmentError, AttachmentManager};
use crate::models::{CommonError, PageView};
use crate::services::{response::ApiResponse, AppState, JwtToken};
use actix_web::{get, post, web, HttpResponse};
use futures::TryStreamExt;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;

const MAX_ATTACHMENT_SIZE: usize = 10 * 1024 * 1024;

/// Upload attachment handler.
/// Attachments may be stored on ECS or local storage in the services, and now is local storage.
/// Adapted from https://github.com/actix/examples/.
/// Note: client can upload multiple files, so we use payload.try_next() to iterate all the files.
/// There is also a while loop and iteration for streams in each file.
#[post("/attachment")]
pub(crate) async fn upload_file(
    app: web::Data<AppState>,
    token: Option<JwtToken>,
    mut payload: actix_multipart::Multipart,
) -> Result<HttpResponse> {
    let uid = token.ok_or(ApiError::new(CommonError::Forbidden))?.uid;
    // Vector of uploaded file.
    let mut uploaded: Vec<Attachment> = vec![];

    // Iterate files over multipart stream
    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(|_| ApiError::new(AttachmentError::NoData))?
    {
        // Get filename and sanitize it.
        // See also:
        // https://docs.rs/actix-http/1.0.1/actix_http/http/header/struct.ContentDisposition.html
        let content_type = field.content_disposition().unwrap();
        let file_ext = get_file_extension(content_type.get_filename().unwrap_or_default());

        // New random uuid for this new file.
        let uuid = uuid::Uuid::new_v4();
        let path = format!("{}/{}.{}", &CONFIG.server.attachment, uuid, file_ext);
        let file = tokio::fs::File::create(&path)
            .await
            .map_err(|_| ApiError::new(AttachmentError::FailedToCreate))?;
        let mut writer = tokio::io::BufWriter::new(file);

        let mut success_flag: bool = true;
        let mut file_size = 0;
        while let Some(chunk) = field
            .try_next()
            .await
            .map_err(|_| ApiError::new(AttachmentError::Interrupted))?
        {
            file_size += chunk.len();
            if file_size > MAX_ATTACHMENT_SIZE {
                success_flag = false;
                break;
            }
            if let Err(_) = writer.write_all(&chunk).await {
                success_flag = false;
                break;
            }
        }

        if success_flag {
            let attachment = Attachment::with_id(uuid)
                .set_uploader(uid)
                .set_file(path, file_size as i32);
            let manager = AttachmentManager::new(&app.pool);
            manager.create(&attachment).await?;

            uploaded.push(attachment);
        }
    }

    let mut resp = HashMap::new();
    resp.insert("uploaded", uploaded);
    Ok(HttpResponse::Ok().json(ApiResponse::normal(resp)))
}

#[get("/attachment")]
pub async fn list_attachments(
    app: web::Data<AppState>,
    token: Option<JwtToken>,
    page: web::Query<PageView>,
) -> Result<HttpResponse> {
    // let token = token.ok_or(ApiError::new(CommonError::Forbidden))?;
    // if !token.is_admin {
    //     return Err(ApiError::new(CommonError::Forbidden));
    // }
    let attachments = AttachmentManager::new(&app.pool).list(page.into_inner()).await?;
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(attachments)))
}

#[get("/attachment/{attachment_id}")]
pub async fn query_attachment(
    app: web::Data<AppState>,
    token: Option<JwtToken>,
    id: web::Path<(uuid::Uuid,)>,
) -> Result<HttpResponse> {
    let attachment = AttachmentManager::new(&app.pool).query(id.into_inner().0).await?;
    if let Some(token) = token {
        if token.is_admin {
            return Ok(HttpResponse::Ok().json(&ApiResponse::normal(attachment)));
        }
    }
    let result: AttachmentBasic = attachment.into();
    Ok(HttpResponse::Ok().json(&ApiResponse::normal(result)))
}
