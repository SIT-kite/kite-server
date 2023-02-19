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

use std::ops::Sub;

use chrono::{DateTime, Duration, Local};
use tonic::{Request, Response, Status};

use kite::model::balance as model;

use crate::error::ToStatus;
use crate::model::ToTimestamp;
pub use crate::service::gen::balance as gen;

impl Into<gen::RoomBalance> for model::ElectricityBalance {
    fn into(self) -> gen::RoomBalance {
        gen::RoomBalance {
            room: self.room,
            balance: self.balance,
            ts: Some(ToTimestamp::datetime(self.ts)),
        }
    }
}

impl Into<gen::BillItem> for model::DailyElectricityBill {
    fn into(self) -> gen::BillItem {
        use gen::bill_item::Identifier;

        gen::BillItem {
            increment: self.charge,
            decrement: self.consumption,
            identifier: Some(Identifier::Date(self.date)),
        }
    }
}

impl Into<gen::BillItem> for model::HourlyElectricityBill {
    fn into(self) -> gen::BillItem {
        use gen::bill_item::Identifier;

        gen::BillItem {
            increment: self.charge,
            decrement: self.consumption,
            identifier: Some(Identifier::Time(self.time)),
        }
    }
}

impl Into<gen::ConsumptionRank> for model::RecentConsumptionRank {
    fn into(self) -> gen::ConsumptionRank {
        gen::ConsumptionRank {
            consumption: self.consumption,
            rank: self.rank,
            total_room: self.room_count,
        }
    }
}

#[tonic::async_trait]
impl gen::balance_service_server::BalanceService for super::KiteGrpcServer {
    async fn get_room_balance(
        &self,
        request: Request<gen::BalanceRequest>,
    ) -> Result<Response<gen::RoomBalance>, Status> {
        let room = request.into_inner().room_number;
        let response = model::get_latest_balance(&self.db, room)
            .await
            .map_err(ToStatus::to_status)?
            .ok_or_else(|| Status::not_found("No such room."))?
            .into();

        Ok(Response::new(response))
    }

    async fn get_consumption_rank(
        &self,
        request: Request<gen::BalanceRequest>,
    ) -> Result<Response<gen::ConsumptionRank>, Status> {
        let room = request.into_inner().room_number;
        let response = model::get_consumption_rank(&self.db, room)
            .await
            .map_err(ToStatus::to_status)?
            .ok_or_else(|| Status::not_found("No such room."))?
            .into();
        Ok(Response::new(response))
    }

    async fn get_bill(&self, request: Request<gen::BillRequest>) -> Result<Response<gen::BillResponse>, Status> {
        let request = request.into_inner();

        // TODO:
        // It's defined that the value of gen::BillType::Daily is 0 and that of gen::BillType::Hourly is 1.
        // However, according to the reason I don't know till now, tonic (with the compiler prost) sees the "request.type" as i32,
        // So I can't use enum match  :-(
        const IS_DAILY: i32 = 0;
        const IS_HOURLY: i32 = 1;

        let bill_list: Vec<gen::BillItem> = match request.r#type {
            IS_DAILY => {
                let to_str = |x: DateTime<Local>| x.format("%Y-%m-%d").to_string();

                let today = Local::now();
                let last_week = today.sub(Duration::days(7));

                model::get_bill_in_day(&self.db, request.room_number, to_str(last_week), to_str(today))
                    .await
                    .map_err(ToStatus::to_status)?
                    .into_iter()
                    .map(Into::into)
                    .collect()
            }
            IS_HOURLY => {
                let now = Local::now();
                let yesterday = now.sub(Duration::hours(24));

                model::get_bill_in_hour(&self.db, request.room_number, yesterday, now)
                    .await
                    .map_err(ToStatus::to_status)?
                    .into_iter()
                    .map(Into::into)
                    .collect()
            }
            _ => {
                return Err(Status::invalid_argument("Bill type is unexpected"));
            }
        };
        Ok(Response::new(gen::BillResponse { bill_list }))
    }
}
