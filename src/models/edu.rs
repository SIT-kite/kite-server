pub use classroom::{convert_time_string, query_avail_classroom, transform_date};
pub use classroom::{AvailClassroom, AvailClassroomQuery};
pub use course::{get_current_term, is_valid_term};
pub use course::{CourseBase, CourseClass};
pub use major::{Major, PlannedCourse};
pub use timetable::{export_course_list_to_calendar, generate_sign};

pub use crate::models::sc::{
    get_sc_score_detail, query_activity_detail, query_activity_list, query_current_sc_activity_list,
    query_current_sc_score_list, save_sc_activity_detail, save_sc_activity_list, save_sc_score_list,
};

mod classroom;
mod course;
mod major;
mod timetable;

#[derive(Debug, thiserror::Error, ToPrimitive)]
pub enum EduError {
    #[error("签名校验失败")]
    SignFailure = 310,
}
