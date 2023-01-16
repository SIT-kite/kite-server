use chrono::{DateTime, Local};
use sqlx::PgPool;
use tonic::{Request, Response, Status};

pub use crate::service::gen::badge::*;
use crate::service::gen::template::{Empty, EmptyRequestWithToken};

/// 识别结果
#[derive(num_derive::ToPrimitive, num_derive::FromPrimitive)]
enum ScanResult {
    /// 没有识别到校徽
    NoBadge = 1,
    /// 当日领福卡次数已达到限制
    ReachLimit = 2,
    /// 没有抽中
    NoCard = 3,
    /// 抽中了
    WinCard = 4,
}

/// 识别记录
#[derive(serde::Serialize)]
pub struct ScanRecord {
    /// 操作用户 ID
    pub uid: i32,
    /// 操作结果类型, 见 `ScanResult`
    pub result: i32,
    /// 卡片类型 （五种福卡之一）
    pub card: Option<i32>,
    /// 操作时间
    pub ts: DateTime<Local>,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct Card {
    /// 卡片类型 （五种福卡之一）
    pub card: Option<i32>,
    /// 操作时间
    pub ts: DateTime<Local>,
}

pub async fn get_cards_list(pool: &PgPool, uid: i32) -> anyhow::Result<Vec<Card>> {
    let cards = sqlx::query_as("SELECT card, ts FROM fu.scan WHERE uid = $1 AND result = 3 AND card != 0;")
        .bind(uid)
        .fetch_all(pool)
        .await?;
    Ok(cards)
}

pub async fn append_share_log(pool: &PgPool, uid: i32) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO fu.share_log (uid) VALUES ($1);")
        .bind(uid)
        .execute(pool)
        .await?;
    Ok(())
}

#[tonic::async_trait]
impl badge_service_server::BadgeService for super::KiteGrpcServer {
    async fn get_user_card_storage(
        &self,
        request: Request<EmptyRequestWithToken>,
    ) -> Result<Response<CardListResponse>, Status> {
        println!("{:?}", request);

        Ok(Response::new(CardListResponse { card_list: vec![] }))
    }

    async fn append_share_log(&self, request: Request<EmptyRequestWithToken>) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty::default()))
    }
}
