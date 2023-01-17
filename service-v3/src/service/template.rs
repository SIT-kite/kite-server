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
