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

pub use convert::*;

mod convert {
    use anyhow::Context;

    use crate::model::{PageSort, PageView, Uuid};
    use crate::service::gen::template as gen;

    pub trait ToUuidMessage {
        fn uuid(self) -> gen::Uuid;
    }

    pub trait ToUuid {
        fn uuid(self) -> anyhow::Result<Uuid>;
    }

    pub trait ToPageView {
        fn page_option(self) -> PageView;
    }

    impl ToUuidMessage for Uuid {
        fn uuid(self) -> gen::Uuid {
            gen::Uuid {
                value: self.to_string(),
            }
        }
    }

    impl ToUuid for gen::Uuid {
        fn uuid(self) -> anyhow::Result<Uuid> {
            Uuid::parse_str(&self.value).with_context(|| format!("Failed to parse uuid: {}", self.value))
        }
    }

    impl ToPageView for gen::PageOption {
        fn page_option(self) -> PageView {
            let sort = if self.sort.unwrap_or(0) == 0 {
                PageSort::Asc
            } else {
                PageSort::Desc
            };
            PageView {
                size: self.size,
                index: self.index,
                sort,
            }
        }
    }
}
