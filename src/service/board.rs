use poem::web::{Data, Json, Multipart, Query};
use sqlx::PgPool;

use crate::{model::{board::{self, Picture}, PageView}, response::ApiResponse};
use super::jwt::JwtToken;
use crate::error::Result;


#[poem::handler]
pub async fn upload(pool: Data<&PgPool>, mut multipart: Multipart, token: JwtToken) -> Result<Json<serde_json::Value>> {
    fn parse_ext(filename: &str) -> String {
        if let Some((_, ext)) = filename.rsplit_once('.') {
            return ext.to_owned();
        }
        "".to_string()
    }
    
    while let Ok(Some(field)) = multipart.next_field().await {
        
        if let Some(name) = field.name().map(ToString::to_string) {
        
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