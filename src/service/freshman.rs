//! This module includes interfaces about freshman queries.
use serde::Deserialize;

use crate::error::ApiError;
use crate::model::freshman::FreshmanManager;
use crate::response::ApiResponse;
use poem::web::{Data, Json, Path, Query};
use poem::{handler, Result};
use sqlx::PgPool;

#[derive(Debug, Deserialize)]
pub struct FreshmanReqSecret {
    pub secret: String,
}

#[handler]
pub async fn get_basic_info(
    pool: Data<&PgPool>,
    Path(account): Path<String>,
    // token: Option<JwtToken>,
    Query(parameters): Query<FreshmanReqSecret>,
) -> Result<Json<serde_json::Value>> {
    // let token = token.unwrap();
    let secret = parameters.secret;

    if account.is_empty() {
        return Err(ApiError::custom(1, "请求的参数错误").into());
    }
    let manager = FreshmanManager::new(&pool);
    let freshman = manager.query(&account, secret.as_str()).await?;
    // if freshman.uid.is_none() && !manager.is_bound(token.uid).await? {
    //     manager.bind(&freshman.student_id, Some(token.uid)).await?;
    // }
    Ok(Json(ApiResponse::normal(freshman).into()))
}

#[derive(Deserialize)]
pub struct UpdateInfo {
    pub contact: Option<String>,
    pub visible: Option<bool>,
    pub secret: String,
}

#[handler]
pub async fn update_account(
    pool: Data<&PgPool>,
    Path(account): Path<String>,
    Json(form): Json<UpdateInfo>,
) -> Result<Json<serde_json::Value>> {
    let secret = form.secret;

    let freshman_manager = FreshmanManager::new(&pool);
    let student = freshman_manager.query(&account, &secret).await?;

    // Set visibility.
    if let Some(visible) = form.visible {
        if visible != student.visible {
            student.set_visibility(&pool, visible).await?;
        }
    }

    // Set contact information.
    if let Some(contact) = form.contact {
        match serde_json::from_str(contact.as_str()) {
            Ok(contact_json) => student.set_contact(&pool, contact_json).await?,
            Err(err) => return Ok(Json(ApiResponse::<()>::fail(1, "Json格式有误".to_string()).into())),
        }
    }
    Ok(Json(ApiResponse::<()>::empty().into()))
}

#[handler]
pub async fn get_roommate(
    pool: Data<&PgPool>,
    Path(account): Path<String>,
    Json(parameters): Json<FreshmanReqSecret>,
) -> Result<Json<serde_json::Value>> {
    let secret = parameters.secret;

    let freshman_manager = FreshmanManager::new(&pool);
    let roommates = freshman_manager
        .query(&account, &secret)
        .await?
        .get_roommates(&pool)
        .await?;

    let response = serde_json::json!({
        "roommates": roommates,
    });
    Ok(Json(ApiResponse::normal(response).into()))
}

#[handler]
pub async fn get_people_familiar(
    pool: Data<&PgPool>,
    Path(account): Path<String>,
    Json(parameters): Json<FreshmanReqSecret>,
) -> Result<Json<serde_json::Value>> {
    let secret = parameters.secret;

    let freshman_manager = FreshmanManager::new(&pool);
    let people_familiar = freshman_manager
        .query(&account, &secret)
        .await?
        .get_people_familiar(&pool)
        .await?;
    let response = serde_json::json!({
        "people_familiar": people_familiar, // TODO: use peopleFamiliar
    });
    Ok(Json(ApiResponse::normal(response).into()))
}

#[handler]
pub async fn get_classmate(
    pool: Data<&PgPool>,
    Path(account): Path<String>,
    Json(parameters): Json<FreshmanReqSecret>,
) -> Result<Json<serde_json::Value>> {
    let secret = parameters.secret;

    let freshman_manager = FreshmanManager::new(&pool);
    let classmates = freshman_manager
        .query(&account, &secret)
        .await?
        .get_classmates(&pool)
        .await?;
    let response = serde_json::json!({
        "classmates": classmates,
    });
    Ok(Json(ApiResponse::normal(response).into()))
}

#[handler]
pub async fn get_analysis_data(
    pool: Data<&PgPool>,
    Path(account): Path<String>,
    Json(parameters): Json<FreshmanReqSecret>,
) -> Result<Json<serde_json::Value>> {
    let secret = parameters.secret;

    let freshman_manager = FreshmanManager::new(&pool);
    let freshman = freshman_manager
        .query(&account, &secret)
        .await?
        .get_analysis(&pool)
        .await?;
    let response = serde_json::json!({
        "freshman": freshman,
    });
    Ok(Json(ApiResponse::normal(response).into()))
}

#[handler]
pub async fn post_analysis_log(
    pool: Data<&PgPool>,
    Path(account): Path<String>,
    Json(parameters): Json<FreshmanReqSecret>,
) -> Result<Json<serde_json::Value>> {
    let secret = parameters.secret;

    let freshman_manager = FreshmanManager::new(&pool);
    let freshman = freshman_manager
        .query(&account, &secret)
        .await?
        .post_analysis_log_model(&pool)
        .await?;

    Ok(Json(ApiResponse::<()>::empty().into()))
}
