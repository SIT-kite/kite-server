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

pub use convert::{ToDateTime, ToTimestamp};
pub use template::*;

pub mod badge;
pub mod balance;
pub mod board;
pub mod classroom_browser;
pub mod template;
pub mod user;

pub mod convert {
    use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Timelike};
    use prost_types::Timestamp;

    pub trait ToDateTime {
        fn timestamp(self) -> DateTime<Local>;
    }

    pub trait ToTimestamp {
        fn datetime(self) -> Timestamp;
    }

    impl ToDateTime for Timestamp {
        fn timestamp(self) -> DateTime<Local> {
            let (secs, nsecs) = (self.seconds, self.nanos);
            let dt = NaiveDateTime::from_timestamp_opt(secs, nsecs as u32).unwrap();

            Local::from_local_datetime(&Local, &dt).unwrap()
        }
    }

    impl ToTimestamp for DateTime<Local> {
        fn datetime(self) -> Timestamp {
            let (secs, nsecs) = (self.timestamp(), self.nanosecond());
            Timestamp {
                seconds: secs,
                nanos: nsecs as i32,
            }
        }
    }
}
