use poem::web::{Data, Json, Multipart, Path, Query};
use serde::Deserialize;
use sqlx::PgPool;

use super::jwt::JwtToken;
use crate::error::Result;
use crate::{
    model::{
        board::{self, Picture},
        PageView,
    },
    response::ApiResponse,
};

#[derive(Debug, Deserialize)]
pub struct LikeType {
    pub like_type: Option<i32>,
}


#[poem::handler]
pub async fn upload(pool: Data<&PgPool>, mut multipart: Multipart, token: JwtToken) -> Result<Json<serde_json::Value>> {
    fn parse_ext(filename: &str) -> String {
        if let Some((_, ext)) = filename.rsplit_once('.') {
            return ext.to_owned();
        }
        "".to_string()
    }

    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(name) = field.file_name().map(ToString::to_string) {
            if let Ok(bytes) = field.bytes().await {
                let ext = parse_ext(&name);
                let picture = Picture::new(token.uid, &ext);
                board::save(&picture, &bytes).await?;
                board::insert_db(&pool, &picture).await?;
            }
        }
    }
    Ok(Json(ApiResponse::<()>::empty().into()))
}

#[poem::handler]
pub async fn get_picture_list(pool: Data<&PgPool>, Query(page): Query<PageView>) -> Result<Json<serde_json::Value>> {
    let result = board::get_picture_list(&pool, &page).await?;
    let response: serde_json::Value = ApiResponse::normal(result).into();

    Ok(Json(response))
}

#[poem::handler]
pub async fn post_like(pool: Data<&PgPool>, Path(id): Path<String>, Query(parameters): Query<LikeType>, token: JwtToken) -> Result<Json<serde_json::Value>> {
    let like_type = match parameters.like_type {
        Some(x) => x,
        None => -1
    };

    let result = board::post_like(&pool, id, token.uid, like_type).await?;
    let response: serde_json::Value = ApiResponse::normal(result).into();

    Ok(Json(response))
}

#[poem::handler]
pub async fn get_my_picture_list(pool: Data<&PgPool>, token: JwtToken, Query(page): Query<PageView>) -> Result<Json<serde_json::Value>> {
    let result = board::get_my_picture_list(&pool, token.uid, &page).await?;
    let response: serde_json::Value = ApiResponse::normal(result).into();
    Ok(Json(response))
}

#[poem::handler]
pub async fn post_delete(pool: Data<&PgPool>, Path(id): Path<String>) -> Result<Json<serde_json::Value>> {
    let result = board::post_delete(&pool, id).await?;
    let response: serde_json::Value = ApiResponse::normal(result).into();

    Ok(Json(response))
}