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

pub type Uuid = uuid::Uuid;

const DEFAULT_PAGE_INDEX: i32 = 0;
const DEFAULT_ITEM_COUNT: i32 = 20;

pub enum PageSort {
    Asc,
    Desc,
}

pub struct PageView {
    pub size: i32,
    pub index: i32,
    pub sort: PageSort,
}

pub enum Gender {
    Male,
    Female,
}

impl Default for PageSort {
    fn default() -> Self {
        PageSort::Asc
    }
}

impl Default for PageView {
    fn default() -> Self {
        Self {
            size: DEFAULT_ITEM_COUNT,
            index: DEFAULT_PAGE_INDEX,
            sort: Default::default(),
        }
    }
}

impl PageView {
    /// Create a new page view structure
    pub fn new() -> Self {
        PageView::default()
    }
    /// Get validated index
    pub fn index(&self) -> i32 {
        if self.index > 0 {
            self.index
        } else {
            DEFAULT_PAGE_INDEX
        }
    }
    /// Get validated item count value
    pub fn count(&self, max_count: i32) -> i32 {
        if self.size < max_count {
            self.size
        } else {
            DEFAULT_ITEM_COUNT
        }
    }
    /// Calculate offset
    pub fn offset(&self, max_count: i32) -> i32 {
        let index = if self.index() > 0 { self.index() - 1 } else { 0 };
        self.count(max_count) * index
    }
}
