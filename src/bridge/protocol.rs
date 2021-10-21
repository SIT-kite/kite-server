use std::pin::Pin;

use serde::{Deserialize, Serialize};
use tokio_tower::multiplex;

use crate::bridge::model::*;

/// Response payload
#[derive(Debug, Serialize)]
pub enum RequestPayload {
    None,
    Ping(String),
    AgentInfo(AgentInfoRequest),
    PortalAuth(PortalAuthRequest),
    ActivityList(ActivityListRequest),
    ActivityDetail(ActivityDetailRequest),
    ScScoreDetail(ScScoreItemRequest),
    ScActivityDetail(ScActivityRequest),
    ScActivityJoin(ScJoinRequest),
    MajorList(MajorRequest),
    TimeTable(TimeTableRequest),
    Score(ScoreRequest),
    ScoreDetail(ScoreDetailRequest),
    SearchLibrary(SearchLibraryRequest),
    BookHoldingInfo(BookHoldingRequest),
    CardExpense(ExpenseRequest),
}

/// Response payload
#[derive(Debug, Deserialize)]
pub enum ResponsePayload {
    None,
    Pong(String),
    Credential(AgentInfo),
    PortalAuth(PortalAuthResponse),
    ActivityList(Vec<Activity>),
    ActivityDetail(Box<ActivityDetail>),
    ScScoreDetail(Vec<ScScoreItem>),
    ScActivityDetail(Vec<ScActivityItem>),
    ScActivityJoin(String),
    MajorList(Vec<Major>),
    TimeTable(Vec<Course>),
    Score(Vec<Score>),
    ScoreDetail(Vec<ScoreDetail>),
    SearchLibrary(SearchLibraryResult),
    BookHoldingInfo(HoldingPreviews),
    CardExpense(ExpensePage),
}

/// Error code and message to response
#[derive(Debug, Deserialize, thiserror::Error)]
#[error("{} ({})", msg, code)]
pub struct ErrorResponse {
    pub code: u16,
    pub msg: String,
}

/// It is a Result enum. Being Ok(ResponsePayload) when the operation completed successfully.
/// Otherwise, an Err(ErrorResponse) representing an error occurred when executing the operation by agent.
pub type ResponseResult = std::result::Result<ResponsePayload, ErrorResponse>;

#[derive(Debug, Serialize)]
pub struct RequestFrame {
    payload: RequestPayload,
}

impl RequestFrame {
    pub fn new(payload: RequestPayload) -> Self {
        Self { payload }
    }
}

#[derive(Debug, Deserialize)]
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

#[derive(Serialize, Deserialize, Debug)]
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
