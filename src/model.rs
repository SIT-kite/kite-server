use thiserror::Error;

pub mod badge;
pub mod classroom;
pub mod contact;
pub mod electricity;
pub mod freshman;
pub mod game;
pub mod library;
pub mod notice;
pub mod report;
pub mod user;
pub mod weather;
pub const CAMPUS_FENGXIAN: i32 = 1;
pub const CAMPUS_XUHUI: i32 = 2;

const DEFAULT_PAGE_INDEX: i32 = 0;
const DEFAULT_ITEM_COUNT: i32 = 20;

/// Page parameters for list pagination
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct PageView {
    /// Page index, 1 is the minimum value
    pub index: Option<i32>,
    /// Page count, 1 is the minimum value
    pub count: Option<i32>,
}

impl PageView {
    /// Create a new page view structure
    pub fn new() -> Self {
        PageView::default()
    }
    /// Get validated index
    pub fn index(&self) -> i32 {
        if let Some(index) = self.index {
            if index > 0 {
                return index;
            }
        }
        DEFAULT_PAGE_INDEX
    }
    /// Get validated item count value
    pub fn count(&self, max_count: i32) -> i32 {
        if let Some(count) = self.count {
            if count < max_count {
                return count;
            }
        }
        DEFAULT_ITEM_COUNT
    }
    /// Calculate offset
    pub fn offset(&self, max_count: i32) -> i32 {
        let index = if self.index() > 0 { self.index() - 1 } else { 0 };
        self.count(max_count) * index
    }
}
