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

use poem::{listener::TcpListener, Route};
use poem_openapi::payload::PlainText;
use poem_openapi::{OpenApi, OpenApiService};

use kite::service::KiteModule;

pub struct ServerHttp;

#[async_trait::async_trait]
impl KiteModule for ServerHttp {
    async fn run() {
        http_service().await.expect("Failed to run http_service")
    }
}

struct V2Interface;

#[OpenApi]
impl V2Interface {
    #[oai(path = "/v2", method = "get")]
    async fn index(&self) -> PlainText<String> {
        PlainText("hello world!".to_string())
    }
}

async fn http_service() -> Result<(), std::io::Error> {
    let api_service = OpenApiService::new(V2Interface, "Kite service", "1.0");
    let app = Route::new().nest("/api", api_service);

    poem::Server::new(TcpListener::bind("127.0.0.1:3000")).run(app).await
}
