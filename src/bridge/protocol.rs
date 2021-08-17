use std::pin::Pin;

use tokio_tower::multiplex;

use crate::bridge::model::{
    Activity, ActivityDetail, ActivityDetailRequest, ActivityListRequest, AgentInfo, AgentInfoRequest,
};

/// Response payload
#[derive(Debug, serde::Serialize)]
pub enum RequestPayload {
    None,
    AgentInfo(AgentInfoRequest),
    ActivityList(ActivityListRequest),
    ActivityDetail(ActivityDetailRequest),
}

/// Response payload
#[derive(Debug, serde::Deserialize)]
pub enum ResponsePayload {
    None,
    Credential(AgentInfo),
    ActivityList(Vec<Activity>),
    ActivityDetail(Box<ActivityDetail>),
}

/// Error code and message to response
#[derive(Debug, serde::Deserialize, thiserror::Error)]
#[error("{} ({})", msg, code)]
pub struct ErrorResponse {
    pub code: u16,
    pub msg: String,
}

pub type ResponseResult = std::result::Result<ResponsePayload, ErrorResponse>;

#[derive(Debug, serde::Serialize)]
pub struct RequestFrame {
    payload: RequestPayload,
}

#[derive(Debug, serde::Deserialize)]
struct ResponseFrame {
    payload: ResponseResult,
}

#[derive(Debug, Default)]
pub struct Tagger(slab::Slab<()>);

impl<Request: core::fmt::Debug, Response: core::fmt::Debug>
    multiplex::TagStore<Tagged<Request>, Tagged<Response>> for Tagger
{
    type Tag = u32;

    fn assign_tag(mut self: Pin<&mut Self>, r: &mut Tagged<Request>) -> Self::Tag {
        r.tag = self.0.insert(()) as u32;
        r.tag
    }
    fn finish_tag(mut self: Pin<&mut Self>, r: &Tagged<Response>) -> Self::Tag {
        self.0.remove(r.tag as usize);
        r.tag
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Tagged<T>
where
    T: core::fmt::Debug,
{
    pub v: T,
    pub tag: u32,
}

impl<T: core::fmt::Debug> From<T> for Tagged<T> {
    fn from(t: T) -> Self {
        Tagged { tag: 0, v: t }
    }
}
