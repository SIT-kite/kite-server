//! This module includes interfaces about freshman queries.
use poem_openapi::{
    param::{Path, Query},
    payload::Json,
    OpenApi,
};

use crate::error::ApiError;
use crate::model::freshman::FreshmanManager;
use crate::response::ApiResponse;
use poem::web::Data;
use poem::Result;
use sqlx::PgPool;

pub struct FreshmanApi;

#[derive(serde::Deserialize)]
pub struct UpdateInfo {
    pub contact: Option<String>,
    pub visible: Option<bool>,
    pub secret: String,
}

#[OpenApi]
impl FreshmanApi {
    #[oai(path = "/freshman/:account", method = "get")]
    pub async fn get_basic_info(
        &self,
        pool: Data<&PgPool>,
        account: Path<String>,
        secret: Query<String>,
    ) -> Result<Json<serde_json::Value>> {
        let account = urlencoding::decode(&account.0).expect("UTF-8");
        let secret = secret.0;

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
    #[oai(path = "/freshman/update", method = "put")]
    pub async fn update_account(
        &self,
        pool: Data<&PgPool>,
        account: Path<String>,
        form: Json<serde_json::Value>,
    ) -> Result<Json<serde_json::Value>> {
        let form = serde_json::from_value::<UpdateInfo>(form.0).unwrap();

        let account = urlencoding::decode(&account.0).expect("UTF-8");
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
                Err(_) => return Ok(Json(ApiResponse::<()>::fail(1, "Json格式有误".to_string()).into())),
            }
        }
        Ok(Json(ApiResponse::<()>::empty().into()))
    }

    #[oai(path = "/freshman/:account/roommate", method = "get")]
    pub async fn get_roommate(
        &self,
        pool: Data<&PgPool>,
        account: Path<String>,
        secret: Query<String>,
    ) -> Result<Json<serde_json::Value>> {
        let account = urlencoding::decode(&account.0).expect("UTF-8");
        let secret = secret.0;

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

    #[oai(path = "/freshman/:account/familiar", method = "get")]
    pub async fn get_people_familiar(
        &self,
        pool: Data<&PgPool>,
        account: Path<String>,
        secret: Query<String>,
    ) -> Result<Json<serde_json::Value>> {
        let account = urlencoding::decode(&account.0).expect("UTF-8");
        let secret = secret.0;

        let freshman_manager = FreshmanManager::new(&pool);
        let people_familiar = freshman_manager
            .query(&account, &secret)
            .await?
            .get_people_familiar(&pool)
            .await?;
        let response = serde_json::json!({
            "peopleFamiliar": people_familiar,
        });
        Ok(Json(ApiResponse::normal(response).into()))
    }

    #[oai(path = "/freshman/:account/classmate", method = "get")]
    pub async fn get_classmate(
        &self,
        pool: Data<&PgPool>,
        account: Path<String>,
        secret: Query<String>,
    ) -> Result<Json<serde_json::Value>> {
        let account = urlencoding::decode(&account.0).expect("UTF-8");
        let secret = secret.0;

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

    #[oai(path = "/freshman/:account/analysis", method = "get")]
    pub async fn get_analysis_data(
        &self,
        pool: Data<&PgPool>,
        account: Path<String>,
        secret: Query<String>,
    ) -> Result<Json<serde_json::Value>> {
        let account = urlencoding::decode(&account.0).expect("UTF-8");
        let secret = secret.0;

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

    #[oai(path = "/freshman/:account/analysis/log", method = "post")]
    pub async fn post_analysis_log(
        &self,
        pool: Data<&PgPool>,
        account: Path<String>,
        secret: Query<String>,
    ) -> Result<Json<serde_json::Value>> {
        let account = urlencoding::decode(&account.0).expect("UTF-8");
        let secret = secret.0;

        let freshman_manager = FreshmanManager::new(&pool);
        freshman_manager
            .query(&account, &secret)
            .await?
            .post_analysis_log_model(&pool)
            .await?;

        Ok(Json(ApiResponse::<()>::empty().into()))
    }
}
