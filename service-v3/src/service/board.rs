/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2021-2023 上海应用技术大学 上应小风筝团队
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use sqlx::{PgPool, Postgres};
use tonic::{Request, Response, Status};

use crate::error::ToStatus;
use crate::model::board as model;
use crate::service::board::gen::{Picture, PictureListResponse, UploadRequest};
pub use crate::service::gen::board as gen;
use crate::service::gen::template::{EmptyRequest, PageOption};
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

    async fn get_my_upload(&self, request: Request<EmptyRequest>) -> Result<Response<PictureListResponse>, Status> {
        Err(tonic::Status::unimplemented("todo"))
    }

    async fn upload(&self, request: Request<UploadRequest>) -> Result<Response<Picture>, Status> {
        Err(tonic::Status::unimplemented("todo"))
    }
}
