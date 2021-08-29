//! This module contains all the abstract models required by the business.

use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::error::ApiError;

/// Telephone mod
pub mod contact;
/// Course and score management.
pub mod edu;
/// Event display, sign-in and statistics
pub mod event;
/// Attachment upload, download and management.
pub mod file;
/// Freshman query.
pub mod freshman;
/// Second-hand market
pub mod mall;
/// Show some mottos.
pub mod motto;
/// Miniprogram index notice;
pub mod notice;
/// Querying electricity bill and expenses record.
pub mod pay;
/// Search mod
pub mod search;
/// User management.
pub mod user;

const DEFAULT_PAGE_INDEX: u16 = 1;
const DEFAULT_ITEM_COUNT: u16 = 20;

#[derive(Debug, Error, ToPrimitive)]
pub enum CommonError {
    #[error("请求成功")]
    Success = 0,
    #[error("接口依赖的模块出现问题, 可能遇到了bug, 请重试或联系易班工作站")]
    Internal = 1,
    #[error("请求的参数错误")]
    Parameter = 2,
    #[error("当前您所在的区域暂不提供服务")]
    AddrNotSupported = 3,
    #[error("该操作需要登录")]
    LoginNeeded = 4,
    #[error("请求的权限不足")]
    Forbidden = 5,
    #[error("需要实名认证后才能继续")]
    IdentityNeeded = 6,
}

impl From<CommonError> for ApiError {
    fn from(common_error: CommonError) -> Self {
        ApiError {
            code: common_error.to_u16().unwrap(),
            inner_msg: None,
            error_msg: Some(common_error.to_string()),
        }
    }
}

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
