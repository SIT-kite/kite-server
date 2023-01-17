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

use sqlx::PgPool;
use tonic::{Request, Response, Status};

use crate::error::ToStatus;
use crate::model::classroom_browser as model;
use crate::service::classroom_browser::gen::Classroom;
pub use crate::service::gen::classroom_browser as gen;

impl Into<model::ClassroomQuery> for gen::ClassroomQuery {
    fn into(self) -> model::ClassroomQuery {
        model::ClassroomQuery {
            building: self.building,
            region: self.region,
            campus: self.campus,
            week: self.week,
            day: self.day,
            want_time: self.time_flag,
        }
    }
}

impl Into<gen::Classroom> for model::Classroom {
    fn into(self) -> gen::Classroom {
        gen::Classroom {
            title: self.title,
            busy_flag: self.busy_flag,
            capacity: self.capacity,
        }
    }
}

pub async fn query_avail_classroom(
    db: &PgPool,
    query: &model::ClassroomQuery,
) -> anyhow::Result<Vec<model::Classroom>> {
    sqlx::query_as(
        "SELECT room, busy_time::int, capacity::int \
            FROM edu.query_available_classrooms($1, $2, $3, $4, $5, $6);",
    )
    .bind(&query.campus)
    .bind(&query.building)
    .bind(&query.region)
    .bind(query.week)
    .bind(query.day)
    .bind(query.want_time.unwrap_or(!0))
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

#[tonic::async_trait]
impl gen::classroom_browser_service_server::ClassroomBrowserService for super::KiteGrpcServer {
    async fn get_available_classroom(
        &self,
        request: Request<gen::ClassroomQuery>,
    ) -> Result<Response<gen::ClassroomListResponse>, Status> {
        let query = request.into_inner().into();

        // TODO: Add cache
        query_avail_classroom(&self.db, &query)
            .await
            .map_err(ToStatus::to_status)
            .map(|classroom_list| {
                let results = classroom_list.into_iter().map(Into::into).collect();

                Response::new(gen::ClassroomListResponse {
                    classroom_list: results,
                })
            })
    }
}
