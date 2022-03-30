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
    pub password: Option<String>,
    /// 登录类型
    pub mode: Option<i32>,
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
    // 常规登录
    if payload.mode.unwrap_or(0) == 0 {
        if !user::Validator::validate_username(&payload.account) {
            return Err(ApiError::new(UserError::InvalidAccountFormat).into());
        }

        // 使用 Cookie 验证
        if let Some(cookie) = payload.cookie {
            portal::Portal::valid_cookie(&client, &payload.account, &cookie).await?;
        } else if let Some(password) = payload.password {
            // 使用身份证号(倒2-倒7)验证
            if (password.len() == 6 && !user::hit_card_number(&pool, &payload.account, &password).await?)
                || password.len() != 6
            {
                // 使用密码尝试验证
                if !user::hit_cache(&pool, &payload.account, &password).await? {
                    let credential = portal::Credential::new(payload.account.clone(), password);
                    // 若登录失败, 函数从此处结束.
                    let mut portal = portal::Portal::login(&client, &credential).await?;
                    let name = portal.get_person_name().await?;
                }
            }
        } else {
            return Err(ApiError::new(UserError::CredentialMissing).into());
        }
    } else
    /* 管理员登录 */
    {
        if let Some(password) = payload.password {
            if !user::hit_admin(&pool, &payload.account, &password).await? {
                return Err(ApiError::new(UserError::NoSuchUser).into());
            }
        } else {
            return Err(ApiError::new(UserError::CredentialMissing).into());
        }
    }

    // 此处验证通过, 从数据库中查找相应记录.
    let query_result = user::query(&pool, &payload.account).await?;
    let user: user::User = if let Some(queried_user) = query_result {
        queried_user
    } else {
        user::create(&pool, &payload.account).await?
    };
    let token = jwt::JwtToken::new(user.uid, user.role).encode();
    let response: serde_json::Value = ApiResponse::normal(SessionResponse { token, profile: user }).into();
    Ok(Json(response))
}
