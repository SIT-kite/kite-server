use poem::web::{Data, Json};
use poem::{handler, Result};
use sqlx::PgPool;

use crate::error::ApiError;
use crate::model::user::{self, UserError};
use crate::portal;
use crate::response::ApiResponse;
use crate::service::jwt;

/// 登录凭据
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credential {
    /// 学号
    pub account: String,
    /// OA密码
    pub password: Option<String>,
    /// 登录 Cookie, 来源于统一认证平台, 用于免密码快速验证.
    pub cookie: Option<String>,
}

#[derive(serde::Serialize)]
pub struct SessionResponse {
    /// Jwt token
    token: String,
    /// Profile
    profile: user::User,
}

#[handler]
pub async fn login(
    pool: Data<&PgPool>,
    client: Data<&reqwest::Client>,
    Json(payload): Json<Credential>,
) -> Result<Json<serde_json::Value>> {
    if !user::Validator::validate_username(&payload.account) {
        return Err(ApiError::new(UserError::InvalidAccountFormat).into());
    }

    if let Some(cookie) = payload.cookie {
        // 使用 Cookie 验证
        portal::Portal::valid_cookie(&client, &payload.account, &cookie).await?;
    } else if let Some(password) = payload.password {
        // 使用密码尝试验证
        let credential = portal::Credential::new(payload.account.clone(), password);
        let _ = portal::Portal::login(&client, &credential).await?;
    } else {
        return Err(ApiError::new(UserError::CredentialMissing).into());
    }
    // 此处验证通过, 从数据库中查找相应记录.
    let query_result = user::query(&pool, &payload.account).await?;
    let user: user::User;
    if let Some(queried_user) = query_result {
        user = queried_user;
    } else {
        user = user::create(&pool, &payload.account).await?;
    }
    let token = jwt::JwtToken::new(user.uid, user.role).encode();
    let response: serde_json::Value = ApiResponse::normal(SessionResponse { token, profile: user }).into();
    Ok(Json(response))
}
