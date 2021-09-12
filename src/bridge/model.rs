use chrono::{DateTime, Local, NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInfoRequest;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInfo {
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalAuthRequest {
    pub account: String,
    pub credential: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PortalAuthResponse {
    Ok,
    Err(String),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScScoreItemRequest {
    pub account: String,
    pub passwd: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScScoreItem {
    pub activity_id: i32,
    pub amount: f32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveScScore {
    pub account: String,
    pub activity_id: i32,
    pub amount: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScActivityRequest {
    pub account: String,
    pub passwd: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScActivityItem {
    pub activity_id: i32,
    pub time: DateTime<Local>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct SaveScActivity {
    pub account: String,
    pub activity_id: i32,
    pub time: DateTime<Local>,
    pub status: String,
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScDetail {
    pub activity_id: i32,
    pub title: String,
    pub time: DateTime<Local>,
    pub status: String,
    pub amount: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct ActivityListRequest {
    /// Count of activities per page.
    pub count: u16,
    /// Page index.
    pub index: u16,
    /// Category Id
    pub category: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDetailRequest {
    /// Activity id in sc.sit.edu.cn
    pub id: i32,
}

/// Activity link, used for list recent activities.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: i32,
    pub category: i32,
}

/// Activity link, used for list recent activities.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDetail {
    /// Activity id
    pub id: i32,
    /// Category id
    pub category: i32,
    /// Activity title
    pub title: String,
    /// Activity start date time
    pub start_time: Option<NaiveDateTime>,
    /// Sign date time
    pub sign_time: Option<NaiveDateTime>,
    /// Activity end date time
    pub end_time: Option<NaiveDateTime>,
    /// Place
    pub place: Option<String>,
    /// Duration
    pub duration: Option<String>,
    /// Activity manager
    pub manager: Option<String>,
    /// Manager contact (phone)
    pub contact: Option<String>,
    /// Activity organizer
    pub organizer: Option<String>,
    /// Activity undertaker
    pub undertaker: Option<String>,
    /// Description in text[]
    pub description: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SchoolYear {
    AllYear,
    SomeYear(i32),
}

#[derive(Clone, Debug, Deserialize, serde_repr::Serialize_repr, PartialEq)]
#[repr(u8)]
pub enum Semester {
    All = 0,
    FirstTerm = 1,
    SecondTerm = 2,
    MidTerm = 3,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MajorRequest {
    pub entrance_year: SchoolYear,
    pub account: String,
    pub passwd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Major {
    /// 入学年份
    entrance_year: i32,
    /// 专业代码
    id: String,
    /// 专业名称
    name: String,
    /// 专业内部标识
    inner_id: String,
    /// 专业方向内部表示
    direction_id: String,
    /// 专业方向
    direction: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimeTableRequest {
    pub account: String,
    pub passwd: String,
    pub school_year: SchoolYear,
    pub semester: Semester,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    /// 课程名称
    pub course_name: String,
    /// 星期
    pub day: i32,
    /// 节次
    pub time_index: i32,
    /// 周次
    pub week: i32,
    /// 教室
    pub place: String,
    /// 教师
    pub teacher: Vec<String>,
    /// 校区
    pub campus: String,
    /// 学分
    pub credit: f32,
    /// 学时
    pub hours: i32,
    /// 教学班
    pub dyn_class_id: String,
    /// 课程代码
    pub course_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScoreRequest {
    pub account: String,
    pub passwd: String,
    pub school_year: SchoolYear,
    pub semester: Semester,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    /// 成绩
    score: f32,
    /// 课程
    course: String,
    /// 课程代码
    course_id: String,
    /// 班级
    class_id: String,
    /// 学年
    school_year: String,
    /// 学期
    semester: Semester,
    /// 学分
    credit: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScoreDetailRequest {
    pub account: String,
    pub password: String,
    pub school_year: SchoolYear,
    pub semester: Semester,
    pub class_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreDetail {
    // 平时成绩
    score_type: String,
    // 期末成绩
    percentage: String,
    // 总评
    score: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScScoreSummary {
    /// Total score.
    pub total: f32,
    /// Subject report.(主题报告)
    pub theme_report: f32,
    /// Social practice.(社会实践)
    pub social_practice: f32,
    /// Innovation, entrepreneurship and creativity.(创新创业创意)
    pub creativity: f32,
    /// Campus safety and civilization.(校园安全文明)
    pub safety_civilization: f32,
    /// Charity and Volunteer.(公益志愿)
    pub charity: f32,
    /// Campus culture.(校园文化)
    pub campus_culture: f32,
}
