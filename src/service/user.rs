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
pub struct Credential {
    /// 学号
    pub account: String,
    /// OA密码
    pub password: String,
    /// 登录类型
    pub mode: Option<i32>,
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
    let name: String = if payload.mode.unwrap_or(0) == 0
    /* 常规登录 */
    {
        if !user::Validator::validate_username(&payload.account) {
            return Err(ApiError::new(UserError::InvalidAccountFormat).into());
        }
        let credential = portal::Credential::new(payload.account.clone(), payload.password.clone());
        // 若登录失败, 函数从此处结束.
        let mut portal = portal::Portal::login(&client, &credential).await?;

        portal.get_person_name().await?
    } else
    /* 管理员登录 */
    {
        if !user::hit_admin(&pool, &payload.account, &payload.password).await? {
            return Err(ApiError::new(UserError::NoSuchUser).into());
        }
        payload.account.clone()
    };

    // 此处验证通过, 从数据库中查找相应记录.
    let query_result = user::query(&pool, &payload.account).await?;
    let user: user::User = if let Some(queried_user) = query_result {
        queried_user
    } else {
        user::create(&pool, &payload.account, &name).await?
    };
    let token = jwt::JwtToken::new(user.uid, user.role).encode();
    let response: serde_json::Value = ApiResponse::normal(SessionResponse { token, profile: user }).into();
    Ok(Json(response))
}
