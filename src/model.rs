pub mod badge;
pub mod classroom;
pub mod contact;
pub mod electricity;
pub mod notice;
pub mod user;
pub mod weather;

pub const CAMPUS_FENGXIAN: i32 = 1;
pub const CAMPUS_XUHUI: i32 = 2;

const DEFAULT_PAGE_INDEX: u16 = 0;
const DEFAULT_ITEM_COUNT: u16 = 20;

/// Page parameters for list pagination
#[derive(Serialize, Deserialize, Default)]
pub struct PageView {
    /// Page index, 1 is the minimum value
    pub index: Option<u16>,
    /// Page count, 1 is the minimum value
    pub count: Option<u16>,
}

impl PageView {
    /// Create a new page view structure
    pub fn new() -> Self {
        PageView::default()
    }
    /// Get validated index
    pub fn index(&self) -> u16 {
        if let Some(index) = self.index {
            if index > 0 {
                return index;
            }
        }
        DEFAULT_PAGE_INDEX
    }
    /// Get validated item count value
    pub fn count(&self, max_count: u16) -> u16 {
        if let Some(count) = self.count {
            if count < max_count {
                return count;
            }
        }
        DEFAULT_ITEM_COUNT
    }
    /// Calculate offset
    pub fn offset(&self, max_count: u16) -> u16 {
        self.count(max_count) * (self.index() - 1)
    }
}
