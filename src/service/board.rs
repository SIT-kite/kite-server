use sqlx::{PgPool, Postgres};
use tonic::{Request, Response, Status};

use crate::error::ToStatus;
use crate::model::board as model;
use crate::service::board::gen::{Picture, PictureListResponse, UploadRequest};
pub use crate::service::gen::board as gen;
use crate::service::gen::template::{EmptyRequestWithToken, PageOption};
use crate::service::template::ToPageView;

impl Into<gen::Picture> for model::Picture {
    fn into(self) -> gen::Picture {
        let uuid = super::template::ToUuidMessage::uuid(self.id);
        let ts = crate::model::ToTimestamp::datetime(self.ts);

        gen::Picture {
            uuid: Some(uuid),
            uid: self.uid,
            publisher: "".to_string(), // TODO
            origin_url: self.url,
            thumbnail: self.thumbnail,
            ts: Some(ts),
        }
    }
}

async fn get_picture_list(pool: &PgPool, page: &crate::model::PageView) -> anyhow::Result<Vec<gen::Picture>> {
    sqlx::query_as::<Postgres, model::Picture>(
        "SELECT id, uid, path as url, thumbnail, ts, ext FROM board.picture_view
        WHERE deleted = FALSE
        ORDER BY ts DESC
        LIMIT $1 OFFSET $2;",
    )
    .bind(page.count(20))
    .bind(page.offset(20))
    .fetch_all(pool)
    .await
    .map(|pic_list| pic_list.into_iter().map(Into::into).collect())
    .map_err(Into::into)
}

#[tonic::async_trait]
impl gen::board_service_server::BoardService for super::KiteGrpcServer {
    async fn get_picture_list(&self, request: Request<PageOption>) -> Result<Response<PictureListResponse>, Status> {
        let request = request.into_inner();
        let page = ToPageView::page_option(request);

        get_picture_list(&self.db, &page)
            .await
            .map(|picture_list| Response::new(PictureListResponse { picture_list }))
            .map_err(ToStatus::to_status)
    }

    async fn get_my_upload(
        &self,
        request: Request<EmptyRequestWithToken>,
    ) -> Result<Response<PictureListResponse>, Status> {
        Err(tonic::Status::unimplemented("todo"))
    }

    async fn upload(&self, request: Request<UploadRequest>) -> Result<Response<Picture>, Status> {
        Err(tonic::Status::unimplemented("todo"))
    }
}
