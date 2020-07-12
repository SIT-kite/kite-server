use failure::Fail;

#[derive(Fail, Debug, ToPrimitive)]
pub enum EventError {
    #[fail(display = "重复创建活动")]
    DuplicatedEvent = 8,
    #[fail(display = "找不到这个活动")]
    NoSuchEvent = 9,
    #[fail(display = "重复申请")]
    DuplicatedApply = 14,
    #[fail(display = "重复签到")]
    AlreadySigned = 17,
}
