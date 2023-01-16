use sqlx::PgPool;
use tonic::{Request, Response, Status};

use crate::error::ToStatus;
use crate::model::badge as model;
use crate::service::auth::get_token_from_request;
pub use crate::service::gen::badge as gen;
use crate::service::gen::template::{Empty, EmptyRequest};

impl Into<gen::Card> for model::Card {
    fn into(self) -> gen::Card {
        use crate::model::ToTimestamp;

        gen::Card {
            card_type: self.card,
            ts: Some(ToTimestamp::datetime(self.ts)),
        }
    }
}

async fn get_cards_list(pool: &PgPool, uid: i32) -> anyhow::Result<Vec<gen::Card>> {
    let cards = sqlx::query_as("SELECT card, ts FROM fu.scan WHERE uid = $1 AND result = 3 AND card != 0;")
        .bind(uid)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|e: model::Card| e.into())
        .collect::<Vec<gen::Card>>();
    Ok(cards)
}

async fn append_share_log(pool: &PgPool, uid: i32) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO fu.share_log (uid) VALUES ($1);")
        .bind(uid)
        .execute(pool)
        .await?;
    Ok(())
}

#[tonic::async_trait]
impl gen::badge_service_server::BadgeService for super::KiteGrpcServer {
    async fn get_user_card_storage(
        &self,
        request: Request<EmptyRequest>,
    ) -> Result<Response<gen::CardListResponse>, Status> {
        let token = get_token_from_request(request)?;
        let result = get_cards_list(&self.db, token.uid).await.map_err(ToStatus::to_status)?;

        Ok(Response::new(gen::CardListResponse { card_list: result }))
    }

    async fn append_share_log(&self, request: Request<EmptyRequest>) -> Result<Response<Empty>, Status> {
        let token = get_token_from_request(request)?;

        append_share_log(&self.db, token.uid)
            .await
            .map_err(ToStatus::to_status)?;
        Ok(Response::new(Empty::default()))
    }
}
