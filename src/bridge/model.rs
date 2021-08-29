use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct AgentInfoRequest;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ActivityListRequest {
    /// Count of activities per page.
    pub count: u16,
    /// Page index.
    pub index: u16,
}

#[derive(Debug, Serialize)]
pub struct ActivityDetailRequest {
    /// Activity id in sc.sit.edu.cn
    pub id: String,
}

/// Activity link, used for list recent activities.
#[derive(Debug, Serialize, Deserialize)]
pub struct Activity {
    pub title: String,
    pub id: String,
    pub link: String,
}

/// Activity link, used for list recent activities.
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityDetail {
    /// Activity id
    pub id: String,
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
    /// Acitvity undertaker
    pub undertaker: Option<String>,
    /// Description in text[]
    pub description: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SchoolYear {
    AllYear,
    SomeYear(i32),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
pub struct Course {
    /// 课程名称
    pub(crate) course_name: String,
    /// 星期
    day: i32,
    /// 节次
    time_index: i32,
    /// 周次
    week: i32,
    /// 教室
    place: String,
    /// 教师
    teacher: Vec<String>,
    /// 校区
    campus: String,
    /// 学分
    credit: f32,
    /// 学时
    hours: f32,
    /// 教学班
    dyn_class_id: String,
    /// 课程代码
    course_id: String,
    /// 陪课班
    prefered_class: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScoreRequest {
    pub account: String,
    pub passwd: String,
    pub school_year: SchoolYear,
    pub semester: Semester,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
