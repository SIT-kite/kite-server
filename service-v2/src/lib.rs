/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2020-2023 上海应用技术大学 上应小风筝团队
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

use poem::middleware::AddData;
use poem::{get, listener::TcpListener, EndpointExt, Route};

use kite::get_db;
use kite::service::KiteModule;

mod response;

mod electricity;
mod error;

pub struct ServerHttp;

#[async_trait::async_trait]
impl KiteModule for ServerHttp {
    async fn run() {
        http_service().await.expect("Failed to run http_service")
    }
}

async fn http_service() -> Result<(), std::io::Error> {
    let route = Route::new().nest(
        "/electricity",
        Route::new()
            .at("/room/:room", get(electricity::query_room_balance))
            .at("/room/:room/rank", get(electricity::query_room_consumption_rank))
            .at("/room/:room/bill/days", get(electricity::query_room_bills_by_day))
            .at("/room/:room/bill/hours", get(electricity::query_room_bills_by_hour)),
    );

    let app = route.with(AddData::new(get_db().clone()));
    poem::Server::new(TcpListener::bind("127.0.0.1:3000")).run(app).await
}
