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
