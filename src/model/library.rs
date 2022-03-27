pub mod appointment;
pub mod period;

pub use appointment::*;
pub use period::*;

use chrono::{DateTime, Local};
use num_derive::ToPrimitive;

#[derive(thiserror::Error, Debug, ToPrimitive)]
pub enum LibraryError {
    #[error("找不到记录")]
    NoSuchItem = 200,
    #[error("当日已满")]
    AlreadyFull = 201,
    #[error("禁止取消已使用的预约")]
    CanNotCancelUsed = 202,
    #[error("禁止取消历史场次")]
    CanNotCancelExpired = 203,
    #[error("禁止申请历史场次")]
    CanNotApplyExpired = 204,
    #[error("权限不足")]
    Forbidden = 205,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct Application {
    /// 预约编号
    pub id: i32,
    /// 预约场次. 格式为 `yyMMdd` + 场次 (1 上午, 2 下午, 3 晚上）
    pub period: i32,
    /// 学号/工号
    pub user: String,
    /// 场次下座位号
    pub index: i32,
    /// 预约状态
    pub status: i32,
    /// 预约时间
    pub ts: DateTime<Local>,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct Notice {
    /// 时间戳
    pub ts: DateTime<Local>,
    /// 内容
    pub html: String,
}

#[derive(sqlx::FromRow)]
pub struct Status {
    /// 预约场次
    pub period: i32,
    /// 已预约人数
    pub applied: i32,
}
