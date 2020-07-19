//! This is the freshman module, which is a part of sit-kite project.
//! Freshman module, as a tool, allows freshmen query their dormitory, counselor
//! and classmates.
//! In the design of this module, we use word "account" to express student id,
//! name or admission ticket number, when the word "secret" used as "password".
//! Usually, secret is the six right characters of their id card number.

mod delivery;
mod familiar;
mod myself;

pub use delivery::Package;
pub use familiar::{NewMate, PeopleFamiliar};
pub use myself::FreshmanBasic;

pub use familiar::*;
pub use myself::*;

#[derive(Debug, Fail, ToPrimitive)]
pub enum FreshmanError {
    #[fail(display = "无匹配的新生数据")]
    NoSuchAccount = 18,
    #[fail(display = "账户不匹配")]
    DismatchAccount = 19,
    #[fail(display = "已绑定")]
    BoundAlready = 20,
    #[fail(display = "未登录")]
    Forbidden = 21,
    #[fail(display = "需要凭据")]
    SecretNeeded = 22,
}
