use chrono::{DateTime, Local};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Result;

#[derive(Debug, serde::Deserialize)]
pub struct UserEvent {
    /// 时间戳
    pub ts: DateTime<Local>,
    /// 事件类型
    #[serde(rename = "type")]
    pub _type: i32,
    /// 事件参数
    pub params: serde_json::Value,
}

pub async fn append_user_event(pool: &PgPool, user: &Uuid, event_list: &Vec<UserEvent>) -> Result<()> {
    for e in event_list {
        sqlx::query("INSERT INTO public.history (user, ts, type, params) VALUES ($1, $2, $3, $4);")
            .bind(user)
            .bind(e.ts)
            .bind(e._type)
            .bind(&e.params)
            .execute(pool)
            .await?;
    }
    Ok(())
}
