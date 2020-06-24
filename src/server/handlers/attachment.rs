use std::io::Write;

use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use chrono::{NaiveDateTime, Utc};
use diesel::{r2d2::ConnectionManager, PgConnection};
use futures::{StreamExt, TryStreamExt};

use crate::error::Result;

//
// Adapted from https://github.com/actix/examples/
// #[post("/attachment")]
// async fn save_file(mut payload: Multipart) -> Result<HttpResponse> {
//     // iterate over multipart stream
//     while let Ok(Some(mut field)) = payload.try_next().await {
//         let content_type = field.content_disposition().unwrap();
//         let filename = content_type.get_filename().unwrap();
//
//         let filepath = format!("d:\\test\\{}", sanitize_filename::sanitize(&filename));
//         // File::create is blocking operation, use threadpool
//         // TODO: 处理错误
//         let mut f = web::block(|| std::fs::File::create(filepath)).await.unwrap();
//         // Field in turn is stream of *Bytes* object
//         let mut success_flag: bool = true;
//         while let Some(chunk) = field.next().await {
//             let data = chunk.unwrap();
//             // filesystem operations are blocking, we have to use threadpool
//             match web::block(move || f.write_all(&data).map(|_| f)).await {
//                 Ok(handle) => f = handle,
//                 Err(_) => {
//                     return Err(ServerError::from())
//                 },
//             }
//         }
//     }
//     Ok(HttpResponse::Ok().json())
// }
