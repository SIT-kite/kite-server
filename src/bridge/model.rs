use std::collections::HashMap;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum_macros::EnumString;

use crate::error::{ApiError, Result};
use crate::models::CommonError;

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
    pub start_time: DateTime<Local>,
    /// Sign date time
    pub sign_start_time: DateTime<Local>,
    /// Activity end date time
    pub sign_end_time: DateTime<Local>,
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
    pub description: String,
    /// Image attachment.
    pub images: Vec<ScImages>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScImages {
    pub new_name: String,
    pub old_name: String,
    pub content: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SchoolYear {
    AllYear,
    SomeYear(i32),
}

pub fn trans_year_to_i32(year: String) -> Result<i32> {
    let first_year = year
        .split_once("-")
        .and_then(|(first, _)| {
            let year = first.parse::<i32>().unwrap_or_default();
            if (2015..2030).contains(&year) {
                Some(year)
            } else {
                None
            }
        })
        .ok_or_else(|| ApiError::new(CommonError::Parameter))?;

    Ok(first_year)
}

pub fn trans_to_year(year: String) -> Result<SchoolYear> {
    let first_year = trans_year_to_i32(year)?;

    Ok(SchoolYear::SomeYear(first_year))
}

#[derive(Clone, Debug, Deserialize, serde_repr::Serialize_repr, PartialEq)]
#[repr(u8)]
pub enum Semester {
    All = 0,
    FirstTerm = 1,
    SecondTerm = 2,
    MidTerm = 3,
}

pub fn trans_to_semester(number: i32) -> Semester {
    match number {
        1 => Semester::FirstTerm,
        2 => Semester::SecondTerm,
        3 => Semester::MidTerm,
        _ => Semester::All,
    }
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
    pub(crate) score: f32,
    /// 课程
    pub(crate) course: String,
    /// 课程代码
    pub(crate) course_id: String,
    /// 班级
    pub(crate) class_id: String,
    /// 学年
    pub(crate) school_year: String,
    /// 学期
    pub(crate) semester: i32,
    /// 学分
    pub(crate) credit: f32,
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
    // 成绩类型
    pub(crate) score_type: String,
    // 百分比
    pub(crate) percentage: String,
    // 成绩
    pub(crate) score: f32,
}

#[derive(sqlx::FromRow, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveScore {
    pub(crate) score: f32,
    /// 课程
    pub(crate) course: String,
    /// 课程代码
    pub(crate) course_id: String,
    /// 班级
    pub(crate) class_id: String,
    /// 学年
    pub(crate) school_year: String,
    /// 学期
    pub(crate) semester: i32,
    /// 学分
    pub(crate) credit: f32,
    /// 详情
    pub(crate) detail: Option<Value>,
    /// 是否评教
    pub(crate) is_evaluated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
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

/// 搜索方式
#[derive(Debug, Serialize, Deserialize, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum SearchWay {
    /// 按任意词查询
    Any,
    /// 标题名
    Title,
    /// 正题名：一本书的主要名称
    TitleProper,
    /// ISBN号
    Isbn,
    /// 著者
    Author,
    /// 主题词
    SubjectWord,
    /// 分类号
    ClassNo,
    /// 控制号
    CtrlNo,
    /// 订购号
    OrderNo,
    /// 出版社
    Publisher,
    /// 索书号
    CallNo,
}

/// 排序规则
#[derive(Debug, Serialize, Deserialize, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum SortWay {
    /// 匹配度
    MatchScore,
    /// 出版日期
    PublishDate,
    /// 主题词
    Subject,
    /// 标题名
    Title,
    /// 作者
    Author,
    /// 索书号
    CallNo,
    /// 标题名拼音
    Pinyin,
    /// 借阅次数
    LoanCount,
    /// 续借次数
    RenewCount,
    /// 题名权重
    TitleWeight,
    /// 正题名权重
    TitleProperWeight,
    /// 卷册号
    Volume,
}

#[derive(Debug, Serialize, Deserialize, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum SortOrder {
    /// 升序排序
    Asc,
    /// 降序排序
    Desc,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchLibraryRequest {
    /// 搜索关键字
    pub keyword: String,
    /// 搜索结果数量
    pub rows: u16,
    /// 搜索分页号
    pub page: u32,
    /// 搜索方式
    pub search_way: SearchWay,
    /// 搜索结果的排序方式
    pub sort_way: SortWay,
    /// 搜索结果的升降序方式
    pub sort_order: SortOrder,
}

/// 馆藏信息检索
#[derive(Debug, Serialize, Deserialize)]
pub struct BookHoldingRequest {
    pub book_id_list: Vec<String>,
}

/// 图书信息
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    /// 图书号
    pub book_id: String,
    /// ISBN号
    pub isbn: String,
    /// 图书标题
    pub title: String,
    /// 图书作者
    pub author: String,
    /// 出版社
    pub publisher: String,
    /// 出版日期
    pub publish_date: String,
    /// 索书号
    pub call_no: String,
}

/// 检索结果
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchLibraryResult {
    /// 检索总结果数(所有页面的结果总数)
    pub result_count: u32,
    /// 检索用时
    pub use_time: f32,
    /// 当前页号
    pub current_page: u32,
    /// 总页数
    pub total_pages: u32,
    /// 当前页面图书列表
    pub book_list: Vec<Book>,
}

/// 馆藏信息预览
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldingPreviews {
    /// 馆藏信息预览
    pub holding_previews: HashMap<String, Vec<HoldingPreview>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldingPreview {
    /// 索书号
    pub call_no: String,
    /// 所在馆代号
    pub library_code: String,
    /// 所在馆藏地点
    pub library_name: String,
    /// 所在馆藏地点代号
    pub location: String,
    /// 所在馆藏地点名
    pub location_name: String,
    /// 馆藏总数
    pub total: u32,
    /// 可借阅的数目
    pub loanable_count: u32,
    /// 书架号
    pub shelf_no: String,
    /// 条码号
    pub barcode: String,
}
