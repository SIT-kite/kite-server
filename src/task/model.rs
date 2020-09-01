use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct AgentInfoRequest;

#[derive(Clone, Deserialize)]
pub struct AgentInfo {
    pub name: String,
}

#[derive(Serialize)]
pub struct ElectricityBillRequest {
    pub room: String,
}

#[derive(Debug, Deserialize, PartialEq, Default)]
pub struct ElectricityBill {
    pub room_id: String,
    pub deposit_balance: f32,
    pub subsidized_balance: f32,
    pub total_balance: f32,
    pub available_power: f32,
}

#[derive(Serialize)]
pub struct ActivityListRequest {
    /// Count of activities per page.
    pub count: u16,
    /// Page index.
    pub index: u16,
}

#[derive(Serialize)]
pub struct CourseScoreRequest {
    pub account: String,
    pub credential: String,
    pub term: String,
}
/// Course score function.
#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
pub struct CourseScoreInner {
    /// Score got for daily performance
    pub regular_grade: f32,
    /// Midterm grade
    pub midterm_grade: f32,
    /// Final exam grade
    pub final_grade: f32,
    /// Total mark
    pub total_mark: f32,
    /// Make up exam score.
    pub make_up_grade: f32,
    /// Total mark after make-up exam
    pub make_up_total: f32,
}

#[derive(Debug, Deserialize)]
pub struct Activity {
    pub title: String,
    pub id: String,
    pub link: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum CourseScoreLine {
    /// Have commented the teacher
    Normal(CourseScoreInner),
    /// Comment (评教) is needed
    Uncomment,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct CourseScore {
    /// Unique ID of the course
    pub course_code: String,
    /// Course name
    pub course_name: String,
    /// credit
    pub course_credit: f32,
    /// score data.
    pub detail: CourseScoreLine,
}
